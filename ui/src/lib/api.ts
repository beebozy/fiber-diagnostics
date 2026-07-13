"use client";

import { useState, useEffect } from "react";
import { IssuesResponse } from "./types";
import { FIXTURE_RESPONSE } from "./fixtures";

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
