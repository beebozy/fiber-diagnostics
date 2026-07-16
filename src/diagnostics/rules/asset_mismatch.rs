//! Category 8: Asset Mismatch -- compares the UDT the invoice requires
//! against the UDT actually funding the channel the payment was routed
//! over.
//!
//! CONFIRMED against a real get_invoice response (2026-07): the invoice's
//! UDT lives in data.attrs as a {"udt_script": "0x<hex>"} entry -- snake_case
//! key, and the value is NOT a JSON object, it's a CKB Molecule-encoded
//! Script (binary table: code_hash:Byte32, hash_type:byte, args:Bytes).
//! decode_molecule_script() below decodes it into the same
//! {code_hash, hash_type, args} JSON shape that channel_status_current's
//! funding_udt_type_script_json already uses (from list_channels' plain
//! JSON Script object), so the two sides can be compared like-for-like.

use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::{ChannelStatus, InvoiceStatus, PaymentStatus};
use std::collections::HashMap;

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decodes a Molecule-encoded Script table into {code_hash, hash_type, args}
/// JSON, matching the shape channel_status_current already stores.
/// Byte-verified against a real captured invoice on 2026-07-11.
fn decode_molecule_script(hex: &str) -> Option<serde_json::Value> {
    let bytes = hex_decode(hex)?;
    if bytes.len() < 16 {
        return None;
    }
    let off0 = read_u32_le(&bytes, 4)? as usize;
    let off1 = read_u32_le(&bytes, 8)? as usize;
    let off2 = read_u32_le(&bytes, 12)? as usize;

    let code_hash = bytes.get(off0..off1)?;
    if code_hash.len() != 32 {
        return None;
    }
    let hash_type_byte = *bytes.get(off1)?;
    let hash_type = match hash_type_byte {
        0 => "data",
        1 => "type",
        2 => "data1",
        4 => "data2",
        _ => return None,
    };
    let args_field = bytes.get(off2..)?;
    let args_len = read_u32_le(args_field, 0)? as usize;
    let args = args_field.get(4..4 + args_len)?;

    Some(serde_json::json!({
        "code_hash": format!("0x{}", hex_encode(code_hash)),
        "hash_type": hash_type,
        "args": format!("0x{}", hex_encode(args)),
    }))
}

/// Pulls the first hop's channel_outpoint out of a payment's stored
/// routers JSON (Vec<SessionRoute> -> SessionRoute.nodes -> SessionRouteNode.channel_outpoint).
fn first_hop_channel_outpoint(router_json: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(router_json).ok()?;
    parsed
        .as_array()?
        .first()?
        .get("nodes")?
        .as_array()?
        .first()?
        .get("channel_outpoint")?
        .as_str()
        .map(|s| s.to_string())
}

/// Finds the udt_script attribute in a parsed CkbInvoice's data.attrs and
/// decodes it. CONFIRMED shape (2026-07): {"udt_script": "0x<molecule hex>"}.
fn extract_invoice_udt(parsed_invoice_json: &str) -> Option<serde_json::Value> {
    let parsed: serde_json::Value = serde_json::from_str(parsed_invoice_json).ok()?;
    let attrs = parsed.get("data")?.get("attrs")?.as_array()?;
    for attr in attrs {
        if let Some(hex) = attr.get("udt_script").and_then(|v| v.as_str()) {
            return decode_molecule_script(hex);
        }
    }
    None
}

pub fn evaluate(
    payments: &[PaymentStatus],
    invoices: &[InvoiceStatus],
    channels: &[ChannelStatus],
) -> Vec<Issue> {
    let invoice_map: HashMap<&str, &InvoiceStatus> = invoices
        .iter()
        .map(|invoice| (invoice.payment_hash.as_str(), invoice))
        .collect();

    let channel_by_outpoint: HashMap<&str, &ChannelStatus> = channels
        .iter()
        .filter_map(|c| c.channel_outpoint.as_deref().map(|op| (op, c)))
        .collect();

    let mut issues = Vec::new();

    for payment in payments {
        let Some(invoice) = invoice_map.get(payment.payment_hash.as_str()) else {
            continue;
        };
        let Some(parsed_invoice_json) = &invoice.parsed_invoice_json else {
            continue;
        };
        let Some(invoice_udt) = extract_invoice_udt(parsed_invoice_json) else {
            continue;
        };

        let Some(router_json) = &payment.router_json else {
            continue;
        };
        let Some(outpoint) = first_hop_channel_outpoint(router_json) else {
            continue;
        };
        let Some(channel) = channel_by_outpoint.get(outpoint.as_str()) else {
            continue;
        };

        match &channel.funding_udt_type_script_json {
            None => {
                issues.push(Issue {
                    kind: "asset-mismatch".into(),
                    severity: Severity::Warning,
                    node_id: payment.node_id.clone(),
                    description: format!(
                        "Payment {} routed over a native-CKB channel but invoice requires a UDT for node {}",
                        payment.payment_hash, payment.node_id
                    ),
                });
            }
            Some(channel_udt_json) => {
                let channel_udt: Option<serde_json::Value> =
                    serde_json::from_str(channel_udt_json).ok();
                if channel_udt.as_ref() != Some(&invoice_udt) {
                    issues.push(Issue {
                        kind: "asset-mismatch".into(),
                        severity: Severity::Warning,
                        node_id: payment.node_id.clone(),
                        description: format!(
                            "Payment {} routed over a channel funded with a different UDT than the invoice requires for node {}",
                            payment.payment_hash, payment.node_id
                        ),
                    });
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_real_captured_udt_script() {
        let hex = "0x550000001000000030000000310000001142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a0120000000878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b";
        let decoded = decode_molecule_script(hex).expect("should decode");
        assert_eq!(
            decoded["code_hash"],
            "0x1142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a"
        );
        assert_eq!(decoded["hash_type"], "type");
        assert_eq!(
            decoded["args"],
            "0x878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b"
        );
    }
}
