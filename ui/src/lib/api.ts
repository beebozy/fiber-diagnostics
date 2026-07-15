"use client";

import { useState, useEffect } from "react";
import { IssuesResponse, NetworkStats, MonitoredNode } from "./types";
import { FIXTURE_RESPONSE, FIXTURE_STATS } from "./fixtures";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "";

export async function fetchIssues(kind?: string, severity?: string): Promise<IssuesResponse> {
  const params = new URLSearchParams();
  if (kind) params.append("kind", kind);
  if (severity) params.append("severity", severity);

  const url = `${API_BASE}/api/issues${params.toString() ? `?${params.toString()}` : ""}`;
  
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`Failed to fetch issues: ${res.statusText}`);
  }
  return res.json();
}

export function useIssues(kind?: string, severity?: string) {
  const [data, setData] = useState<IssuesResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [isUsingFixtures, setIsUsingFixtures] = useState(false);

  useEffect(() => {
    let active = true;

    async function loadData() {
      try {
        const issuesData = await fetchIssues(kind, severity);
        if (active) {
          setData(issuesData);
          setError(null);
          setLastUpdated(new Date());
          setIsUsingFixtures(false);
          setLoading(false);
        }
      } catch (err: any) {
        console.warn("API unavailable, falling back to fixtures:", err.message);
        if (active) {
          // Fallback to fixture data
          // Update the generated_at to represent "now" for accurate time display
          const mockedResponse = {
            ...FIXTURE_RESPONSE,
            generated_at: new Date().toISOString(),
          };
          
          // Apply manual client-side filtering to fixtures if needed
          let filteredIssues = [...mockedResponse.issues];
          if (kind) {
            filteredIssues = filteredIssues.filter(i => i.kind === kind);
          }
          if (severity) {
            filteredIssues = filteredIssues.filter(i => i.severity.toLowerCase() === severity.toLowerCase());
          }

          setData({
            generated_at: mockedResponse.generated_at,
            count: filteredIssues.length,
            issues: filteredIssues,
          });
          setError(null);
          setLastUpdated(new Date());
          setIsUsingFixtures(true);
          setLoading(false);
        }
      }
    }

    loadData();
    const interval = setInterval(loadData, 5000);

    return () => {
      active = false;
      clearInterval(interval);
    };
  }, [kind, severity]);

  return { data, loading, error, lastUpdated, isUsingFixtures };
}

export async function fetchStats(): Promise<NetworkStats> {
  const url = `${API_BASE}/api/stats`;
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`Failed to fetch stats: ${res.statusText}`);
  }
  return res.json();
}

export function useStats() {
  const [data, setData] = useState<NetworkStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isUsingFixtures, setIsUsingFixtures] = useState(false);

  useEffect(() => {
    let active = true;

    async function loadData() {
      try {
        const statsData = await fetchStats();
        if (active) {
          setData(statsData);
          setError(null);
          setIsUsingFixtures(false);
          setLoading(false);
        }
      } catch (err: any) {
        if (active) {
          setData({
            ...FIXTURE_STATS,
            generated_at: new Date().toISOString(),
          });
          setError(null);
          setIsUsingFixtures(true);
          setLoading(false);
        }
      }
    }

    loadData();
    const interval = setInterval(loadData, 5000);

    return () => {
      active = false;
      clearInterval(interval);
    };
  }, []);

  return { data, loading, error, isUsingFixtures };
}

const MOCK_NODES: MonitoredNode[] = [
  { id: "node1", name: "Local Node A", rpc_url: "http://127.0.0.1:8227", enabled: true },
  { id: "node2", name: "Local Node B", rpc_url: "http://127.0.0.1:8237", enabled: true },
];

export async function fetchNodes(): Promise<MonitoredNode[]> {
  const url = `${API_BASE}/api/nodes`;
  try {
    const res = await fetch(url);
    if (!res.ok) {
      throw new Error(`Failed to fetch nodes: ${res.statusText}`);
    }
    return await res.json();
  } catch (err) {
    console.warn("API unavailable, falling back to mock nodes:", err);
    return MOCK_NODES;
  }
}

export async function addNode(id: string, name: string, rpc_url: string): Promise<MonitoredNode> {
  const url = `${API_BASE}/api/nodes`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ id, name, rpc_url }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error || `Failed to add node: ${res.statusText}`);
  }
  return res.json();
}

export async function deleteNode(id: string): Promise<void> {
  const url = `${API_BASE}/api/nodes/${id}`;
  const res = await fetch(url, {
    method: "DELETE",
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error || `Failed to delete node: ${res.statusText}`);
  }
}

