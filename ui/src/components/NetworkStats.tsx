"use client";

import { useStats } from "../lib/api";
import styles from "./NetworkStats.module.css";

function IconServer() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
      <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
      <line x1="6" y1="6" x2="6.01" y2="6" />
      <line x1="6" y1="18" x2="6.01" y2="18" />
    </svg>
  );
}

function IconUsers() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" />
      <circle cx="9" cy="7" r="4" />
      <path d="M23 21v-2a4 4 0 00-3-3.87" />
      <path d="M16 3.13a4 4 0 010 7.75" />
    </svg>
  );
}

function IconGitBranch() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="6" y1="3" x2="6" y2="15" />
      <circle cx="18" cy="6" r="3" />
      <circle cx="6" cy="18" r="3" />
      <path d="M18 9a9 9 0 01-9 9" />
    </svg>
  );
}

function IconGlobe() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z" />
    </svg>
  );
}

export default function NetworkStats() {
  const { data, loading, error } = useStats();

  if (loading) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.shimmer} />
      </div>
    );
  }

  if (error || !data) {
    return null;
  }

  return (
    <div className={styles.statsBar}>
      <div className={styles.statItem}>
        <span className={styles.icon}><IconServer /></span>
        <span className={styles.label}>Monitored Nodes:</span>
        <span className={styles.value}>
          {data.nodes_online}/{data.monitored_nodes} online
        </span>
      </div>

      <div className={styles.divider} />

      <div className={styles.statItem}>
        <span className={styles.icon}><IconUsers /></span>
        <span className={styles.label}>Connected Peers:</span>
        <span className={styles.value}>{data.total_peers}</span>
      </div>

      <div className={styles.divider} />

      <div className={styles.statItem}>
        <span className={styles.icon}><IconGitBranch /></span>
        <span className={styles.label}>Active Channels:</span>
        <span className={styles.value}>{data.total_channels}</span>
      </div>

      <div className={styles.divider} />

      <div className={styles.statItem}>
        <span className={styles.icon}><IconGlobe /></span>
        <span className={styles.label}>Network Graph:</span>
        <span className={styles.value}>
          {data.graph_nodes} nodes, {data.graph_channels} channels
        </span>
      </div>
    </div>
  );
}
