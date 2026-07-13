"use client";

import { useMemo } from "react";
import { Issue } from "../lib/types";
import styles from "./StatsBar.module.css";

interface StatsBarProps {
  issues: Issue[];
}

function IconActivity() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
    </svg>
  );
}

function IconAlertCircle() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="8" x2="12" y2="12" />
      <line x1="12" y1="16" x2="12.01" y2="16" />
    </svg>
  );
}

function IconAlertTriangle() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
      <line x1="12" y1="9" x2="12" y2="13" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  );
}

function IconCheckCircle() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
      <polyline points="22 4 12 14.01 9 11.01" />
    </svg>
  );
}

export default function StatsBar({ issues }: StatsBarProps) {
  const stats = useMemo(() => {
    const total = issues.length;
    const critical = issues.filter((i) => i.severity === "Critical").length;
    const warning = issues.filter((i) => i.severity === "Warning").length;
    const allNodeIds = new Set(issues.map((i) => i.node_id));
    const downNodeIds = new Set(
      issues.filter((i) => i.kind === "node-down").map((i) => i.node_id)
    );
    const totalMonitored = Math.max(allNodeIds.size, 1);
    const healthy = totalMonitored - downNodeIds.size;
    return { total, critical, warning, healthy, totalMonitored };
  }, [issues]);

  const cards = [
    {
      label: "Total Issues",
      value: stats.total,
      icon: <IconActivity />,
      colorClass: styles.blue,
      desc: "Active diagnostics alerts",
    },
    {
      label: "Critical",
      value: stats.critical,
      icon: <IconAlertCircle />,
      colorClass: styles.red,
      desc: "Requires immediate action",
    },
    {
      label: "Warnings",
      value: stats.warning,
      icon: <IconAlertTriangle />,
      colorClass: styles.amber,
      desc: "Performance degradation risks",
    },
    {
      label: "Healthy Nodes",
      value: `${stats.healthy}/${stats.totalMonitored}`,
      icon: <IconCheckCircle />,
      colorClass: styles.green,
      desc: "Reachability check status",
    },
  ];

  return (
    <div className={styles.grid}>
      {cards.map((card, idx) => (
        <div key={idx} className={`${styles.card} ${card.colorClass}`}>
          <div className={styles.header}>
            <span className={styles.label}>{card.label}</span>
            <span className={styles.icon}>{card.icon}</span>
          </div>
          <div className={styles.content}>
            <span className={styles.value}>{card.value}</span>
            <span className={styles.desc}>{card.desc}</span>
          </div>
        </div>
      ))}
    </div>
  );
}
