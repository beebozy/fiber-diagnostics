import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import Sidebar from "@/components/Sidebar";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
});

export const metadata: Metadata = {
  title: "Fiber Diagnostics — Network Health Dashboard",
  description: "Real-time diagnostics and monitoring for Fiber Network nodes, channels, and payments.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={inter.variable}>
      <body>
        <Sidebar />
        <div style={{
          marginLeft: "var(--sidebar-width)",
          marginTop: "var(--header-height)",
          padding: "var(--space-xl)",
          minHeight: "calc(100vh - var(--header-height))",
          display: "flex",
          flexDirection: "column",
        }}>
          {children}
        </div>
      </body>
    </html>
  );
}
