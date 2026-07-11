use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::PaymentStatus;

pub fn evaluate(payment: &PaymentStatus) -> Option<Issue> {
    let failed = payment.status.eq_ignore_ascii_case("failed");
    let fee_too_low = payment
        .failed_error
        .as_deref()
        .map(|err| {
            let err = err.to_lowercase();
            err.contains("fee too low")
                || err.contains("insufficient fee")
                || err.contains("fee too small")
        })
        .unwrap_or(false);

    if failed && fee_too_low {
        return Some(Issue {
            kind: "fee-too-low".into(),
            severity: Severity::Warning,
            node_id: payment.node_id.clone(),
            description: format!(
                "Payment {} failed because the fee was too low for node {}",
                payment.payment_hash, payment.node_id
            ),
        });
    }

    None
}
