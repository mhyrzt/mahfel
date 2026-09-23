use mahfel_domain::{NodeId, SessionId};

use crate::ports::MediaNodeRecord;

/// Weighted rendezvous ordering. Free room slots carry the weight, so a node
/// with twice the available capacity wins about twice as many new sessions.
pub(super) fn ordered_candidates(session: SessionId, nodes: &[MediaNodeRecord]) -> Vec<NodeId> {
    let mut candidates: Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            let free = u64::from(node.max_rooms).saturating_sub(node.active_rooms);
            if node.draining || node.active_workers == 0 || free == 0 {
                return None;
            }
            let hash = stable_hash(session, &node.node_id);
            let unit = (hash as f64 + 1.0) / (u64::MAX as f64 + 2.0);
            let score = -unit.ln() / free as f64;
            Some((score, node.node_id.clone()))
        })
        .collect();
    candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
    candidates.into_iter().map(|(_, node)| node).collect()
}

fn stable_hash(session: SessionId, node: &NodeId) -> u64 {
    // FNV-1a is a deterministic distribution aid, not a security boundary.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in session
        .as_uuid()
        .as_bytes()
        .iter()
        .chain(node.as_str().as_bytes())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, max_rooms: u32, active_rooms: u64, draining: bool) -> MediaNodeRecord {
        MediaNodeRecord {
            node_id: NodeId::from_deployment(id).expect("valid node id"),
            grpc_url: format!("http://{id}:50051"),
            max_rooms,
            active_workers: 1,
            active_rooms,
            transports: 0,
            draining,
        }
    }

    #[test]
    fn excludes_unavailable_nodes_and_spreads_by_free_slots() {
        let nodes = [
            node("wide", 8, 0, false),
            node("narrow", 2, 0, false),
            node("full", 1, 1, false),
            node("draining", 100, 0, true),
        ];
        let mut wide = 0;
        let mut narrow = 0;
        for index in 0..1000_u128 {
            let session = SessionId::from_uuid(uuid::Uuid::from_u128(index));
            let candidates = ordered_candidates(session, &nodes);
            assert_eq!(candidates.len(), 2);
            if candidates[0].as_str() == "wide" {
                wide += 1;
            } else {
                narrow += 1;
            }
        }
        assert!(
            wide > narrow * 2,
            "larger free capacity should win more sessions"
        );
        assert!(narrow > 0, "smaller healthy nodes must still receive work");
    }
}
