"use client";

import styles from "./Header.module.css";

interface HeaderProps {
  title: string;
  children?: React.ReactNode;
}

export default function Header({ title, children }: HeaderProps) {
  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <h2 className={styles.title}>{title}</h2>
        <div className={styles.actions}>
          {children}
        </div>
      </div>
    </header>
  );
}
