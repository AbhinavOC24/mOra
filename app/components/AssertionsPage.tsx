import { useState, useEffect } from "react";
import { useWallet } from "../contexts/WalletContext";

// Status display helpers
const STATUS_MAP: Record<string, string> = {
  requested: "Requested",
  proposed: "Proposed",
  disputed: "Disputed",
  resolved: "Resolved",
};

interface Assertion {
  id: string;
  question: string;
  status: string;
  answerType: string;
  proposedAnswer: string;
  bond: string;
  reward: string;
  livenessEndsAt: number;
}

interface AssertionsPageProps {
  onNewAssertion: () => void;
}

export default function AssertionsPage({ onNewAssertion }: AssertionsPageProps) {
  const { connected } = useWallet();
  const [filter, setFilter] = useState<"all" | "proposed" | "disputed" | "resolved">("all");
  const [selected, setSelected] = useState<string | null>(null);
  const [assertions, setAssertions] = useState<Assertion[]>([]);
  const [now, setNow] = useState(Math.floor(Date.now() / 1000));

  useEffect(() => {
    // Fetch data from local backend server
    fetch('http://localhost:3001/api/assertions')
      .then(res => res.json())
      .then(data => setAssertions(data))
      .catch(err => console.error("Could not fetch backend:", err));
    
    // Update current time every second for the clock
    const interval = setInterval(() => {
      setNow(Math.floor(Date.now() / 1000));
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const handleResolve = (id: string) => {
    fetch(`http://localhost:3001/api/assertions/${id}/resolve`, { method: 'POST' })
      .then(res => res.json())
      .then(data => {
        if (data.success) {
          setAssertions(prev => prev.map(a => a.id === id ? data.assertion : a));
        }
      });
  };

  const filtered = filter === "all" ? assertions : assertions.filter((a) => a.status === filter);
  const detail = selected ? assertions.find((a) => a.id === selected) : null;
  
  // Compute how much time is left for the detail view
  let timePassed = false;
  let remainingText = "Expired";
  if (detail) {
    const diff = detail.livenessEndsAt - now;
    if (diff > 0) {
      const minutes = Math.floor(diff / 60);
      const seconds = diff % 60;
      remainingText = `${minutes}m ${seconds}s remaining`;
    } else {
      timePassed = true;
    }
  }

  return (
    <div className="page-full" style={{ display: "flex", gap: 16 }}>
      {/* List panel */}
      <div style={{ flex: "0 0 440px", maxWidth: 440 }}>
        <div className="page-header" style={{ marginBottom: 16 }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <h1>Assertions</h1>
            <button className="btn btn-primary btn-sm" onClick={onNewAssertion}>+ New</button>
          </div>
          <p>All open assertions on the protocol.</p>
        </div>

        {/* Tab filters */}
        <div className="tabs" style={{ marginBottom: 14 }}>
          {(["all", "proposed", "disputed", "resolved"] as const).map((f) => (
            <button key={f} className={`tab${filter === f ? " active" : ""}`} onClick={() => setFilter(f)}>
              {f.charAt(0).toUpperCase() + f.slice(1)}
            </button>
          ))}
        </div>

        <div className="assertion-list">
          {filtered.length === 0 && (
            <div className="empty-state">
              <div className="empty-state-title">No assertions</div>
              <div className="empty-state-desc">No {filter} assertions exist yet.</div>
            </div>
          )}
          {filtered.map((a) => {
            const diff = a.livenessEndsAt - now;
            const isExpired = diff <= 0;
            return (
              <div
                key={a.id}
                className={`assertion-item${selected === a.id ? " active" : ""}`}
                onClick={() => setSelected(a.id === selected ? null : a.id)}
                style={selected === a.id ? { borderColor: "var(--color-border-hover)" } : {}}
              >
                <div className="assertion-item-id">#{a.id}</div>
                <div className="assertion-item-body">
                  <div className="assertion-item-question">{a.question}</div>
                  <div className="assertion-item-meta">
                    <span className="badge badge-default">{STATUS_MAP[a.status] ?? a.status}</span>
                    <span className="meta-text">{a.answerType}</span>
                    <span className="meta-text">{a.status === 'proposed' ? (isExpired ? 'Expired' : `${Math.floor(diff/60)}m left`) : ''}</span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Detail panel */}
      {detail ? (
        <div style={{ flex: 1, paddingTop: 0 }}>
          <div style={{ marginBottom: 16, paddingTop: 28 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 4 }}>
              <span style={{ fontSize: 12, color: "var(--color-text-tertiary)" }}>#{detail.id}</span>
              <span className="badge badge-white">{STATUS_MAP[detail.status]}</span>
              <span className="badge badge-default">{detail.answerType}</span>
            </div>
            <h2 style={{ fontSize: 16, fontWeight: 600, letterSpacing: "-0.3px" }}>{detail.question}</h2>
          </div>

          <div className="grid-2" style={{ marginBottom: 12 }}>
            <div className="stat-card">
              <div className="stat-label">Proposer Bond</div>
              <div className="stat-value" style={{ fontSize: 18 }}>{detail.bond}</div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Reward Pool</div>
              <div className="stat-value" style={{ fontSize: 18 }}>{detail.reward}</div>
            </div>
          </div>

          <div className="card" style={{ marginBottom: 12 }}>
            <div className="card-header"><span className="card-title">Proposed Answer</span></div>
            <div style={{ fontSize: 20, fontWeight: 600 }}>{detail.proposedAnswer}</div>
            <div style={{ fontSize: 13, color: "var(--color-text-secondary)", marginTop: 8, fontVariantNumeric: "tabular-nums" }}>
              Time remaining: <strong style={{ color: timePassed ? "var(--color-error)" : "var(--color-text-primary)" }}>{remainingText}</strong>
            </div>
          </div>

          {/* Actions */}
          <div className="card">
            <div className="card-header"><span className="card-title">Actions</span></div>
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
              {detail.status === "requested" && (
                <button className="btn btn-primary" disabled={!connected}>
                  Propose Answer
                </button>
              )}
              {detail.status === "proposed" && (
                <>
                  {!timePassed ? (
                    <>
                      <button className="btn btn-primary" disabled={true} title="Wait until liveness window expires">Auto-Resolve</button>
                      <button className="btn btn-secondary" disabled={!connected}>Dispute</button>
                    </>
                  ) : (
                    <>
                      <button className="btn btn-primary" disabled={!connected} onClick={() => handleResolve(detail.id)}>Auto-Resolve (Time Passed)</button>
                      <button className="btn btn-secondary" disabled={true} title="Liveness window closed">Dispute</button>
                    </>
                  )}
                </>
              )}
              {detail.status === "disputed" && (
                <button className="btn btn-primary" disabled={!connected}>Go to VLA Round →</button>
              )}
              {detail.status === "resolved" && (
                <button className="btn btn-secondary" disabled={!connected}>Claim Reward</button>
              )}
              {!connected && (
                <span style={{ fontSize: 12, color: "var(--color-text-tertiary)", marginLeft: "auto" }}>
                  Connect wallet to interact
                </span>
              )}
            </div>
          </div>
        </div>
      ) : (
        <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", paddingTop: 80 }}>
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: 13, color: "var(--color-text-tertiary)" }}>Select an assertion to view details</div>
          </div>
        </div>
      )}
    </div>
  );
}
