import { IssuesResponse } from "./types";

export const FIXTURE_RESPONSE: IssuesResponse = {
  generated_at: new Date().toISOString(),
  count: 8,
  issues: [
    {
      kind: "node-down",
      severity: "Critical",
      node_id: "node_bad_1",
      description: "Node node_bad_1 is unreachable through RPC (Connection refused)",
    },
    {
      kind: "peer-offline",
      severity: "Warning",
      node_id: "node1",
      description: "Peer 02a3f9... is offline for node node1",
    },
    {
      kind: "channel-not-ready",
      severity: "Warning",
      node_id: "node1",
      description: "Channel 0xdb4a9b... is not ready: state=NegotiatingFunding, enabled=true",
    },
    {
      kind: "insufficient-balance",
      severity: "Warning",
      node_id: "node2",
      description: "Channel 0x8f22bc... has low local balance: local=50000 remote=950000 for node node2",
    },
    {
      kind: "invoice-expired",
      severity: "Warning",
      node_id: "node1",
      description: "Invoice 0x77ae81... has expired for node node1",
    },
    {
      kind: "no-route",
      severity: "Warning",
      node_id: "node2",
      description: "Payment 0x11bb2c... failed because no route was available for node node2",
    },
    {
      kind: "fee-too-low",
      severity: "Warning",
      node_id: "node2",
      description: "Payment 0xa9bc11... failed because the fee was too low for node node2",
    },
    {
      kind: "asset-mismatch",
      severity: "Warning",
      node_id: "node1",
      description: "Payment 0xcc9a22... asset type 'BTC' does not match invoice currency 'CKB' for node node1",
    },
  ],
};
