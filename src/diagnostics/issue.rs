#[derive(Debug, Clone)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub kind: String,
    pub severity: Severity,
    pub node_id: String,
    pub description: String,
}
