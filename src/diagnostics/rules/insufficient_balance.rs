use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::ChannelStatus;

fn parse_amount(value: &str) -> Option<i128> {
    let value = value.trim();
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return i128::from_str_radix(hex, 16).ok();
    }

    value.parse::<i128>().ok()
}

pub fn evaluate(channel: &ChannelStatus) -> Option<Issue> {
    let local_balance = channel.local_balance_raw.as_str();
    let remote_balance = channel.remote_balance_raw.as_str();

    if let (Some(local), Some(remote)) = (parse_amount(local_balance), parse_amount(remote_balance))
    {
        if local == 0 || local < remote / 10 {
            return Some(Issue {
                kind: "insufficient-balance".into(),
                severity: Severity::Warning,
                node_id: channel.node_id.clone(),
                description: format!(
                    "Channel {} has low local balance: local={} remote={} for node {}",
                    channel.channel_id, local, remote, channel.node_id
                ),
            });
        }
    }

    None
}
