use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Severity::Critical => "critical",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub kind: String,
    pub severity: Severity,
    pub node_id: String,
    pub description: String,
}
