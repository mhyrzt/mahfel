use crate::ports::{
    ActivityRecord, ActivityStatus, InteractionCommand, InteractionError, InteractionStore,
    Principal,
};
use mahfel_domain::{ActivityKind, ActorId, SessionId};

pub async fn execute<S: InteractionStore + ?Sized>(
    store: &S,
    session: SessionId,
    principal: &Principal,
    request_id: uuid::Uuid,
    mut command: InteractionCommand,
) -> Result<(), InteractionError> {
    if let InteractionCommand::SaveDraft { definition, .. } = &mut command {
        *definition = definition.clone().normalize()?;
    }
    store.execute(session, principal, request_id, command).await
}

#[derive(Clone, Debug)]
pub struct ActivityResults {
    pub respondents: usize,
    pub counts: Vec<usize>,
    pub correct_options: Option<Vec<u16>>,
    pub explanation: Option<String>,
    pub correct: Option<bool>,
}

#[must_use]
pub fn results(activity: &ActivityRecord, actor: ActorId) -> Option<ActivityResults> {
    let own = activity
        .votes
        .iter()
        .find(|vote| vote.actor_id == actor && !vote.selected.is_empty());
    let revealed = activity.status == ActivityStatus::Closed
        || (activity.status == ActivityStatus::Open
            && activity.definition.kind == ActivityKind::Poll
            && own.is_some());
    if !revealed {
        return None;
    }
    let quiz = activity.definition.kind == ActivityKind::Quiz;
    Some(ActivityResults {
        respondents: activity
            .votes
            .iter()
            .filter(|v| !v.selected.is_empty())
            .count(),
        counts: (0..activity.definition.options.len())
            .map(|i| {
                activity
                    .votes
                    .iter()
                    .filter(|v| v.selected.iter().any(|n| usize::from(*n) == i))
                    .count()
            })
            .collect(),
        correct_options: quiz.then(|| activity.definition.correct_options.clone()),
        explanation: if quiz {
            activity.definition.explanation.clone()
        } else {
            None
        },
        correct: if quiz {
            own.map(|v| activity.definition.is_correct(&v.selected))
        } else {
            None
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::ActivityVote;
    use mahfel_domain::ActivityDefinition;
    #[test]
    fn open_quiz_never_reveals_key_counts_or_correctness_even_to_its_voter() {
        let actor = ActorId::new();
        let mut a = ActivityRecord {
            id: uuid::Uuid::new_v4(),
            definition: ActivityDefinition {
                question: "Pick".into(),
                options: vec!["A".into(), "B".into()],
                kind: ActivityKind::Quiz,
                multiple: true,
                anonymous: true,
                correct_options: vec![0, 1],
                explanation: Some("Both".into()),
            },
            status: ActivityStatus::Open,
            votes: vec![ActivityVote {
                actor_id: actor,
                name: "A".into(),
                selected: vec![0],
                version: 1,
            }],
        };
        assert!(results(&a, actor).is_none());
        a.status = ActivityStatus::Closed;
        let r = results(&a, actor).expect("closed quiz results");
        assert_eq!(r.correct, Some(false));
        assert_eq!(r.correct_options, Some(vec![0, 1]));
        a.definition.kind = ActivityKind::Poll;
        a.status = ActivityStatus::Open;
        assert!(results(&a, ActorId::new()).is_none());
        assert!(results(&a, actor).is_some());
    }
}
