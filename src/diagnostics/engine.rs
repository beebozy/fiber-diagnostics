use crate::diagnostics::issue::Issue;
use crate::diagnostics::rules::node_down;

pub struct DiagnosticsEngine;

impl DiagnosticsEngine {
    pub fn evaluate(nodes: Vec<node_down::NodeStatus>) -> Vec<Issue> {
        let mut issues = Vec::new();

        for node in nodes {
            if let Some(issue) = node_down::evaluate(&node) {
                issues.push(issue);
            }
        }

        issues
    }
}
