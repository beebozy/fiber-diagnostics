use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::InvoiceStatus;
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

fn parse_expiry(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }

    let raw_trimmed = raw.trim();
    let secs = if let Some(hex) = raw_trimmed
        .strip_prefix("0x")
        .or_else(|| raw_trimmed.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16).ok()
    } else {
        raw_trimmed.parse::<i64>().ok()
    };

    secs.and_then(|s| Utc.timestamp_opt(s, 0).single())
}

pub fn evaluate(invoice: &InvoiceStatus, payment_node_ids: &HashMap<&str, &str>) -> Option<Issue> {
    let status_expired = invoice
        .invoice_status
        .as_deref()
        .map(|status| status.eq_ignore_ascii_case("expired"))
        .unwrap_or(false);

    let expired_by_time = invoice
        .expiry_time_raw
        .as_deref()
        .and_then(parse_expiry)
        .map(|expiry| expiry < Utc::now())
        .unwrap_or(false);

    if status_expired || expired_by_time {
        let node_id = payment_node_ids
            .get(invoice.payment_hash.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        return Some(Issue {
            kind: "invoice-expired".into(),
            severity: Severity::Warning,
            node_id: node_id.clone(),
            description: format!(
                "Invoice {} has expired for node {}",
                invoice.payment_hash, node_id
            ),
        });
    }

    None
}
