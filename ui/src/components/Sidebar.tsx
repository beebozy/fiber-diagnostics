"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import styles from "./Sidebar.module.css";

function IconDashboard() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
    </svg>
  );
}

function IconAlerts() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
      <path d="M13.73 21a2 2 0 01-3.46 0" />
    </svg>
  );
}

function IconNodes() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
      <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
      <line x1="6" y1="6" x2="6.01" y2="6" />
      <line x1="6" y1="18" x2="6.01" y2="18" />
    </svg>
  );
}

function LogoIcon() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="url(#logoGrad)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <defs>
        <linearGradient id="logoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#06b6d4" />
          <stop offset="100%" stopColor="#3b82f6" />
        </linearGradient>
      </defs>
      <polygon points="12 2 2 7 12 12 22 7 12 2" />
      <polyline points="2 17 12 22 22 17" />
      <polyline points="2 12 12 17 22 12" />
    </svg>
  );
}

export default function Sidebar() {
  const pathname = usePathname();

  const navItems = [
    { label: "Dashboard", href: "/", icon: <IconDashboard /> },
    { label: "Issues", href: "#", icon: <IconAlerts />, disabled: true },
    { label: "Nodes", href: "#", icon: <IconNodes />, disabled: true },
  ];

  return (
    <aside className={styles.sidebar}>
      <div className={styles.logoContainer}>
        <span className={styles.logoIcon}><LogoIcon /></span>
        <h1 className={styles.logoText}>Fiber Diagnostics</h1>
      </div>

      <nav className={styles.nav}>
        {navItems.map((item, index) => {
          const isActive = pathname === item.href && !item.disabled;
          return (
            <Link
              key={index}
              href={item.href}
              className={`${styles.navItem} ${isActive ? styles.active : ""} ${
                item.disabled ? styles.disabled : ""
              }`}
              onClick={(e) => {
                if (item.disabled) {
                  e.preventDefault();
                }
              }}
            >
              <span className={styles.navIcon}>{item.icon}</span>
              <span className={styles.navLabel}>{item.label}</span>
              {item.disabled && <span className={styles.badge}>Soon</span>}
            </Link>
          );
        })}
      </nav>

      <div className={styles.footer}>
        <div className={styles.versionContainer}>
          <span className={styles.versionLabel}>System Status</span>
          <span className={styles.versionValue}>v0.1.0</span>
        </div>
      </div>
    </aside>
  );
}
