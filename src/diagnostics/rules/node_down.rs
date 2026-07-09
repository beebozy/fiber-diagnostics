use crate::diagnostics::issue::{Issue, Severity};

pub struct NodeStatus {
    pub node_id: String,
    pub rpc_reachable: bool,
}

pub fn evaluate(node: &NodeStatus) -> Option<Issue> {
    if !node.rpc_reachable {
        return Some(Issue {
            kind: "NODE_DOWN".into(),
            severity: Severity::Critical,
            node_id: node.node_id.clone(),
            description: format!(
                "Node {} is unreachable through RPC",
                node.node_id
            ),
        });
    }

    None
}
