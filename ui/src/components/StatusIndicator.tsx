"use client";

import { useState, useEffect } from "react";
import styles from "./StatusIndicator.module.css";

interface StatusIndicatorProps {
  lastUpdated: Date | null;
  loading: boolean;
  isUsingFixtures?: boolean;
}

export default function StatusIndicator({
  lastUpdated,
  loading,
  isUsingFixtures = false,
}: StatusIndicatorProps) {
  const [secondsAgo, setSecondsAgo] = useState<number | null>(null);

  useEffect(() => {
    if (!lastUpdated) {
      setSecondsAgo(null);
      return;
    }

    const updateTimer = () => {
      const diff = Math.floor((new Date().getTime() - lastUpdated.getTime()) / 1000);
      setSecondsAgo(diff >= 0 ? diff : 0);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [lastUpdated]);

  let statusText = "Connecting...";
  let dotClass = styles.dotConnecting;
  let labelClass = styles.labelConnecting;

  if (loading && !lastUpdated) {
    statusText = "Syncing...";
    dotClass = styles.dotLoading;
  } else if (lastUpdated !== null) {
    const timeText =
      secondsAgo === null
        ? "just now"
        : secondsAgo < 5
        ? "just now"
        : secondsAgo < 60
        ? `${secondsAgo}s ago`
        : `${Math.floor(secondsAgo / 60)}m ago`;

    if (isUsingFixtures) {
      statusText = `Fixture Data (${timeText})`;
      dotClass = styles.dotFixture;
      labelClass = styles.labelFixture;
    } else {
      statusText = `Live Sync (${timeText})`;
      dotClass = styles.dotLive;
      labelClass = styles.labelLive;
    }
  }

  return (
    <div className={styles.container}>
      <div className={styles.dotContainer}>
        <div className={`${styles.dot} ${dotClass}`} />
        <div className={`${styles.ring} ${dotClass}`} />
      </div>
      <span className={`${styles.label} ${labelClass}`}>{statusText}</span>
    </div>
  );
}
