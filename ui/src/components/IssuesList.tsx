"use client";

import { useState } from "react";
import { Issue, ISSUE_KINDS, ISSUE_KIND_LABELS, Severity } from "../lib/types";
import IssueCard from "./IssueCard";
import styles from "./IssuesList.module.css";

interface IssuesListProps {
  issues: Issue[];
}

function IconSearch() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
  );
}

function IconCheck() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
      <polyline points="22 4 12 14.01 9 11.01" />
    </svg>
  );
}

export default function IssuesList({ issues }: IssuesListProps) {
  const [selectedSeverity, setSelectedSeverity] = useState<Severity | "All">("All");
  const [selectedKind, setSelectedKind] = useState<string>("All");
  const [searchQuery, setSearchQuery] = useState("");

  const severities: (Severity | "All")[] = ["All", "Critical", "Warning", "Info"];

  const filteredIssues = issues.filter((issue) => {
    const matchesSeverity =
      selectedSeverity === "All" || issue.severity === selectedSeverity;
    const matchesKind = selectedKind === "All" || issue.kind === selectedKind;
    const searchLower = searchQuery.toLowerCase();
    const matchesSearch =
      issue.node_id.toLowerCase().includes(searchLower) ||
      issue.description.toLowerCase().includes(searchLower) ||
      (ISSUE_KIND_LABELS[issue.kind] || issue.kind).toLowerCase().includes(searchLower);
    return matchesSeverity && matchesKind && matchesSearch;
  });

  return (
    <div className={styles.container}>
      <div className={styles.filterSection}>
        <div className={styles.titleRow}>
          <h3 className={styles.sectionTitle}>Active Alerts</h3>
          <span className={styles.countBadge}>
            {filteredIssues.length} {filteredIssues.length === 1 ? "alert" : "alerts"}
          </span>
        </div>

        <div className={styles.controls}>
          <div className={styles.searchContainer}>
            <span className={styles.searchIcon}><IconSearch /></span>
            <input
              type="text"
              placeholder="Search by node, error, or alert type..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                className={styles.clearSearch}
              >
                ×
              </button>
            )}
          </div>

          <div className={styles.filtersGroup}>
            <div className={styles.severitySelector}>
              {severities.map((sev) => {
                const isActive = selectedSeverity === sev;
                let activeClass = styles.activeAll;
                if (sev === "Critical") activeClass = styles.activeCritical;
                if (sev === "Warning") activeClass = styles.activeWarning;
                if (sev === "Info") activeClass = styles.activeInfo;
                return (
                  <button
                    key={sev}
                    onClick={() => setSelectedSeverity(sev)}
                    className={`${styles.pill} ${isActive ? activeClass : ""}`}
                  >
                    {sev}
                  </button>
                );
              })}
            </div>

            <div className={styles.selectContainer}>
              <select
                value={selectedKind}
                onChange={(e) => setSelectedKind(e.target.value)}
                className={styles.select}
              >
                <option value="All">All Types</option>
                {ISSUE_KINDS.map((kind) => (
                  <option key={kind} value={kind}>
                    {ISSUE_KIND_LABELS[kind] || kind}
                  </option>
                ))}
              </select>
              <span className={styles.selectArrow}>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="6 9 12 15 18 9" />
                </svg>
              </span>
            </div>
          </div>
        </div>
      </div>

      {filteredIssues.length > 0 ? (
        <div className={styles.grid}>
          {filteredIssues.map((issue, index) => (
            <IssueCard
              key={`${issue.node_id}-${issue.kind}-${index}`}
              issue={issue}
              index={index}
            />
          ))}
        </div>
      ) : (
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>
            <IconCheck />
          </div>
          <h4 className={styles.emptyTitle}>All Systems Nominal</h4>
          <p className={styles.emptyText}>
            {issues.length === 0
              ? "No diagnostics alerts detected in this network scan."
              : "No alerts match the current filter criteria."}
          </p>
        </div>
      )}
    </div>
  );
}
