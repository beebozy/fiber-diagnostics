export type Severity = "Critical" | "Warning" | "Info";

export interface Issue {
  kind: string;
  severity: Severity;
  node_id: string;
  description: string;
}

export interface IssuesResponse {
  generated_at: string;
  count: number;
  issues: Issue[];
}

export const ISSUE_KINDS = [
  "node-down",
  "peer-offline",
  "channel-not-ready",
  "insufficient-balance",
  "invoice-expired",
  "no-route",
  "fee-too-low",
  "asset-mismatch",
] as const;

export type IssueKind = (typeof ISSUE_KINDS)[number];

export const ISSUE_KIND_LABELS: Record<string, string> = {
  "node-down": "Node Down",
  "peer-offline": "Peer Offline",
  "channel-not-ready": "Channel Not Ready",
  "insufficient-balance": "Insufficient Balance",
  "invoice-expired": "Invoice Expired",
  "no-route": "No Route",
  "fee-too-low": "Fee Too Low",
  "asset-mismatch": "Asset Mismatch",
};

export const SEVERITY_ORDER: Record<Severity, number> = {
  Critical: 0,
  Warning: 1,
  Info: 2,
};

export interface NetworkStats {
  generated_at: string;
  graph_nodes: number;
  graph_channels: number;
  monitored_nodes: number;
  nodes_online: number;
  nodes_offline: number;
  total_peers: number;
  total_channels: number;
  active_issues: number;
}

