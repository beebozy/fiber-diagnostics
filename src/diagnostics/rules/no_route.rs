use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::PaymentStatus;

pub fn evaluate(payment: &PaymentStatus) -> Option<Issue> {
    let failed = payment.status.eq_ignore_ascii_case("failed");
    let has_no_route = payment
        .failed_error
        .as_deref()
        .map(|err| {
            err.to_lowercase().contains("no route") || err.to_lowercase().contains("no_route")
        })
        .unwrap_or(false);

    if failed && has_no_route {
        return Some(Issue {
            kind: "no-route".into(),
            severity: Severity::Warning,
            node_id: payment.node_id.clone(),
            description: format!(
                "Payment {} failed because no route was available for node {}",
                payment.payment_hash, payment.node_id
            ),
        });
    }

    None
}
