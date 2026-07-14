use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::NodeStatus;

pub fn evaluate(node: &NodeStatus) -> Option<Issue> {
    if !node.rpc_reachable {
        return Some(Issue {
            kind: "node-down".into(),
            severity: Severity::Critical,
            node_id: node.node_id.clone(),
            description: format!("Node {} is unreachable through RPC", node.node_id),
        });
    }

    None
}
