use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::PeerStatus;

pub fn evaluate(peer: &PeerStatus) -> Option<Issue> {
    if !peer.connected {
        return Some(Issue {
            kind: "peer-offline".into(),
            severity: Severity::Warning,
            node_id: peer.node_id.clone(),
            description: format!(
                "Peer {} is offline for node {}",
                peer.peer_pubkey, peer.node_id
            ),
        });
    }

    None
}
