use crate::diagnostics::issue::{Issue, Severity};
use crate::diagnostics::repository::ChannelStatus;

pub fn evaluate(channel: &ChannelStatus) -> Option<Issue> {
    let state = channel.state_name.to_uppercase();
    if !channel.enabled || state != "CHANNELREADY" {
        return Some(Issue {
            kind: "channel-not-ready".into(),
            severity: Severity::Warning,
            node_id: channel.node_id.clone(),
            description: format!(
                "Channel {} is not ready: state={}, enabled={}",
                channel.channel_id, channel.state_name, channel.enabled
            ),
        });
    }

    None
}
