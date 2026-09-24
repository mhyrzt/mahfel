use mahfel_application::ports::{CollaborationCheckpoint, CollaborationStore};
use mahfel_domain::{
    ActorId, CollaborationDocumentId, CollaborationEvent, CollaborationEventKind, SessionId,
};
use sha2::{Digest, Sha256};
use yrs::{
    Doc, ReadTxn, StateVector, Transact, Update,
    updates::{decoder::Decode, encoder::Encode},
};

pub(super) struct DocumentState {
    pub doc: Doc,
    pub cursor: i64,
    document_id: CollaborationDocumentId,
    updates_since_checkpoint: u32,
}

impl DocumentState {
    pub(super) async fn load(
        store: &dyn CollaborationStore,
        session_id: SessionId,
        actor: ActorId,
        document_id: CollaborationDocumentId,
    ) -> Result<Self, String> {
        let mut doc = Doc::new();
        let mut cursor = 0;
        if let Some(checkpoint) = store
            .checkpoint(document_id)
            .await
            .map_err(|error| error.to_string())?
        {
            if let Some(restored) = restore_checkpoint(&checkpoint) {
                doc = restored;
                cursor = checkpoint.through_sequence;
            } else {
                tracing::warn!(%document_id, "invalid collaboration checkpoint; replaying journal");
            }
        }
        let mut state = Self {
            doc,
            cursor,
            document_id,
            updates_since_checkpoint: 0,
        };
        state.refresh(store, session_id, actor).await?;
        Ok(state)
    }

    pub(super) async fn refresh(
        &mut self,
        store: &dyn CollaborationStore,
        session_id: SessionId,
        actor: ActorId,
    ) -> Result<Vec<CollaborationEvent>, String> {
        let mut delivered = Vec::new();
        loop {
            let events = store
                .events(session_id, actor, self.cursor, 100)
                .await
                .map_err(|error| error.to_string())?;
            let count = events.len();
            for event in events {
                if event.kind == CollaborationEventKind::DocumentUpdate
                    && event.document_id == Some(self.document_id)
                    && let Some(update) = &event.update
                {
                    apply(&self.doc, update)?;
                    self.updates_since_checkpoint = self.updates_since_checkpoint.saturating_add(1);
                }
                self.cursor = event.sequence;
                delivered.push(event);
            }
            if count < 100 {
                break;
            }
        }
        if self.updates_since_checkpoint >= 100 {
            self.checkpoint(store).await?;
            self.updates_since_checkpoint = 0;
        }
        Ok(delivered)
    }

    pub(super) fn diff(&self, vector: &StateVector) -> Vec<u8> {
        self.doc.transact().encode_state_as_update_v1(vector)
    }

    pub(super) fn vector(&self) -> StateVector {
        self.doc.transact().state_vector()
    }

    async fn checkpoint(&self, store: &dyn CollaborationStore) -> Result<(), String> {
        let txn = self.doc.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        let vector = txn.state_vector().encode_v1();
        let digest = Sha256::digest(&update);
        store
            .save_checkpoint(self.document_id, self.cursor, &update, &vector, &digest)
            .await
            .map_err(|error| error.to_string())
    }
}

fn restore_checkpoint(checkpoint: &CollaborationCheckpoint) -> Option<Doc> {
    let actual: [u8; 32] = Sha256::digest(&checkpoint.update).into();
    if actual.as_slice() != checkpoint.sha256.as_slice() {
        return None;
    }
    let doc = Doc::new();
    apply(&doc, &checkpoint.update).ok()?;
    let matches = doc.transact().state_vector().encode_v1() == checkpoint.state_vector;
    matches.then_some(doc)
}

pub(super) fn apply(doc: &Doc, bytes: &[u8]) -> Result<(), String> {
    let update = Update::decode_v1(bytes).map_err(|error| error.to_string())?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::restore_checkpoint;
    use mahfel_application::ports::CollaborationCheckpoint;
    use sha2::{Digest, Sha256};
    use yrs::{Doc, ReadTxn, StateVector, Text, Transact, updates::encoder::Encode};

    #[test]
    fn interrupted_or_corrupt_checkpoint_requires_journal_replay() {
        let source = Doc::new();
        let text = source.get_or_insert_text("notes");
        text.insert(&mut source.transact_mut(), 0, "hello");
        let txn = source.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        let vector = txn.state_vector().encode_v1();
        let digest = Sha256::digest(&update).to_vec();
        let checkpoint = CollaborationCheckpoint {
            through_sequence: 5,
            update: update.clone(),
            state_vector: vector.clone(),
            sha256: digest,
        };
        let restored = restore_checkpoint(&checkpoint).expect("valid checkpoint");
        assert_eq!(restored.transact().state_vector().encode_v1(), vector);
        let truncated = CollaborationCheckpoint {
            update: update[..update.len() - 1].to_vec(),
            ..checkpoint
        };
        assert!(restore_checkpoint(&truncated).is_none());
    }
}
