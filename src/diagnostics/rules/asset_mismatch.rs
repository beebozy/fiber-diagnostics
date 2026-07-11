use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::{InvoiceStatus, PaymentStatus};
use std::collections::HashMap;

pub fn evaluate(payments: &[PaymentStatus], invoices: &[InvoiceStatus]) -> Vec<Issue> {
    let invoice_map: HashMap<&str, &InvoiceStatus> = invoices
        .iter()
        .map(|invoice| (invoice.payment_hash.as_str(), invoice))
        .collect();

    let mut issues = Vec::new();
    for payment in payments {
        if let Some(invoice) = invoice_map.get(payment.payment_hash.as_str()) {
            if let (Some(asset_type), Some(currency)) = (&payment.asset_type, &invoice.currency) {
                if !asset_type.eq_ignore_ascii_case(currency) {
                    issues.push(Issue {
                        kind: "asset-mismatch".into(),
                        severity: Severity::Warning,
                        node_id: payment.node_id.clone(),
                        description: format!(
                            "Payment {} asset type '{}' does not match invoice currency '{}' for node {}",
                            payment.payment_hash, asset_type, currency, payment.node_id
                        ),
                    });
                }
            }
        }
    }

    issues
}
