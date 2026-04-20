import React, { useState } from "react";
import { useWallet } from "../contexts/WalletContext";

type Page =
  | "overview"
  | "assertions"
  | "new-assertion"
  | "vla"
  | "lock"
  | "admin";

interface LayoutProps {
  children: React.ReactNode;
  activePage: Page;
  onNavigate: (page: Page) => void;
  pageTitle: string;
}

function truncate(addr: string) {
  return addr.slice(0, 4) + "…" + addr.slice(-4);
}

const NAV_ITEMS: { id: Page; label: string; icon: string; section?: string }[] = [
  { id: "overview", label: "Overview", icon: "◻", section: "Protocol" },
  { id: "assertions", label: "Assertions", icon: "◈", section: "Protocol" },
  { id: "new-assertion", label: "New Assertion", icon: "+", section: "Protocol" },
  { id: "vla", label: "VLA / Vote", icon: "◉", section: "Arbitration" },
  { id: "lock", label: "veMORA Lock", icon: "⬡", section: "Arbitration" },
  { id: "admin", label: "Admin", icon: "○", section: "Settings" },
];

export default function Layout({ children, activePage, onNavigate, pageTitle }: LayoutProps) {
  const { publicKey, connected, connect, disconnect } = useWallet();

  const grouped = NAV_ITEMS.reduce<Record<string, typeof NAV_ITEMS>>((acc, item) => {
    const sec = item.section ?? "General";
    if (!acc[sec]) acc[sec] = [];
    acc[sec].push(item);
    return acc;
  }, {});

  return (
    <div className="layout">
      {/* Sidebar */}
      <aside className="sidebar">
        <div className="sidebar-logo">
          <div className="sidebar-logo-mark">mΩ</div>
          <span className="sidebar-logo-text">mORA</span>
          <span className="sidebar-logo-badge">alpha</span>
        </div>
        <nav className="sidebar-nav">
          {Object.entries(grouped).map(([section, items]) => (
            <React.Fragment key={section}>
              <div className="sidebar-section-label">{section}</div>
              {items.map((item) => (
                <button
                  key={item.id}
                  className={`nav-item${activePage === item.id ? " active" : ""}`}
                  onClick={() => onNavigate(item.id)}
                >
                  <span className="nav-item-icon" style={{ fontFamily: "monospace", fontSize: "12px" }}>
                    {item.icon}
                  </span>
                  {item.label}
                </button>
              ))}
            </React.Fragment>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div style={{ fontSize: 11, color: "var(--color-text-tertiary)", marginBottom: 6 }}>
            Devnet
          </div>
          <div style={{ fontSize: 11, color: "var(--color-text-tertiary)", fontFamily: "monospace" }}>
            E7TZ…Pnat
          </div>
        </div>
      </aside>

      {/* Main */}
      <div className="main">
        <header className="topbar">
          <span className="topbar-title">{pageTitle}</span>
          <div className="topbar-actions">
            {connected ? (
              <button
                className="wallet-btn connected"
                onClick={disconnect}
              >
                <span className="wallet-dot" />
                {truncate(publicKey!.toString())}
              </button>
            ) : (
              <button className="wallet-btn" onClick={connect}>
                Connect Wallet
              </button>
            )}
          </div>
        </header>
        {children}
      </div>
    </div>
  );
}
