"use client";

import { useIssues } from "@/lib/api";
import Header from "@/components/Header";
import StatusIndicator from "@/components/StatusIndicator";
import StatsBar from "@/components/StatsBar";
import IssuesList from "@/components/IssuesList";
import styles from "./page.module.css";

export default function Home() {
  const { data, loading, error, lastUpdated, isUsingFixtures } = useIssues();

  return (
    <div className={styles.wrapper}>
      {/* Top Navigation / Action Header */}
      <Header title="Network Status Dashboard">
        <StatusIndicator 
          lastUpdated={lastUpdated} 
          loading={loading} 
          isUsingFixtures={isUsingFixtures} 
        />
      </Header>

      {/* Main Panel Content */}
      {error ? (
        <div className={styles.errorState}>
          <div className={styles.errorIcon}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </div>
          <h3 className={styles.errorTitle}>Diagnostics Error</h3>
          <p className={styles.errorText}>{error}</p>
        </div>
      ) : loading && !data ? (
        <div className={styles.skeletonContainer}>
          {/* Skeleton cards matching StatsBar */}
          <div className={styles.skeletonGrid}>
            {[...Array(4)].map((_, i) => (
              <div key={i} className={`${styles.skeletonCard} ${styles.shimmer}`} />
            ))}
          </div>
          {/* Skeleton list matching IssuesList */}
          <div className={styles.skeletonListHeader} />
          <div className={styles.skeletonList}>
            {[...Array(3)].map((_, i) => (
              <div key={i} className={`${styles.skeletonListItem} ${styles.shimmer}`} />
            ))}
          </div>
        </div>
      ) : (
        <div className={styles.fadeContent}>
          {data && (
            <>
              {/* Stat Summary Metrics */}
              <StatsBar issues={data.issues} />

              {/* Filterable Issues Alert List */}
              <IssuesList issues={data.issues} />
            </>
          )}
        </div>
      )}
    </div>
  );
}
