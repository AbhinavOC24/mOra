import { useState } from "react";
import { useWallet } from "../contexts/WalletContext";

// Mock disputed assertion for demo
const MOCK_VLA = {
  id: "1002",
  question: "What was BTC price at UTC 00:00 Apr 20?",
  proposedAnswer: "64000",
  commitEndsAt: Date.now() / 1000 + 3600,
  revealEndsAt: Date.now() / 1000 + 7200,
  totalPowerProposer: 12400,
  totalPowerDisputer: 8200,
  status: "CommitPhase",
};

export default function VlaPage() {
  const { connected } = useWallet();
  const [activeTab, setActiveTab] = useState<"vote" | "finalize">("vote");
  const [phase, setPhase] = useState<"commit" | "reveal" | "done">("commit");
  const [side, setSide] = useState<"proposer" | "disputer" | null>(null);
  const [salt, setSalt] = useState("");
  const [committed, setCommitted] = useState(false);

  const totalPower = MOCK_VLA.totalPowerProposer + MOCK_VLA.totalPowerDisputer;
  const proposerPct = totalPower ? (MOCK_VLA.totalPowerProposer / totalPower) * 100 : 0;
  const disputerPct = totalPower ? (MOCK_VLA.totalPowerDisputer / totalPower) * 100 : 0;

  const timeLeft = (ts: number) => {
    const diff = ts - Date.now() / 1000;
    if (diff <= 0) return "Ended";
    const h = Math.floor(diff / 3600);
    const m = Math.floor((diff % 3600) / 60);
    return `${h}h ${m}m`;
  };

  return (
    <div className="page">
      <div className="page-header">
        <h1>VLA Round</h1>
        <p>Value Locked Arbitration — commit-reveal voting by veMORA holders.</p>
      </div>

      {/* Active round card */}
      <div className="card" style={{ marginBottom: 12 }}>
        <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 16 }}>
          <div>
            <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 6 }}>
              <span style={{ fontSize: 11, color: "var(--color-text-tertiary)" }}>#{MOCK_VLA.id}</span>
              <span className="badge badge-white">{MOCK_VLA.status}</span>
            </div>
            <div style={{ fontSize: 15, fontWeight: 600, letterSpacing: "-0.2px" }}>
              {MOCK_VLA.question}
            </div>
            <div style={{ fontSize: 12.5, color: "var(--color-text-secondary)", marginTop: 4 }}>
              Proposed answer: <strong style={{ color: "var(--color-text-primary)" }}>{MOCK_VLA.proposedAnswer}</strong>
            </div>
          </div>
          <div style={{ flexShrink: 0, textAlign: "right" }}>
            <div style={{ fontSize: 11, color: "var(--color-text-tertiary)", marginBottom: 2 }}>Commit window</div>
            <div style={{ fontSize: 14, fontWeight: 600 }}>{timeLeft(MOCK_VLA.commitEndsAt)}</div>
          </div>
        </div>
      </div>

      {/* Vote tally */}
      <div className="card" style={{ marginBottom: 12 }}>
        <div className="card-header">
          <span className="card-title">Vote Tally</span>
          <span className="card-meta">{totalPower.toLocaleString()} veMORA total</span>
        </div>
        <div className="vote-bars">
          <div className="vote-bar-row">
            <div className="vote-bar-label">
              <span>Proposer Win</span>
              <span>{MOCK_VLA.totalPowerProposer.toLocaleString()} vp ({proposerPct.toFixed(1)}%)</span>
            </div>
            <div className="vote-bar-track">
              <div className="vote-bar-fill" style={{ width: `${proposerPct}%` }} />
            </div>
          </div>
          <div className="vote-bar-row">
            <div className="vote-bar-label">
              <span>Disputer Win</span>
              <span>{MOCK_VLA.totalPowerDisputer.toLocaleString()} vp ({disputerPct.toFixed(1)}%)</span>
            </div>
            <div className="vote-bar-track">
              <div className="vote-bar-fill" style={{ width: `${disputerPct}%` }} />
            </div>
          </div>
        </div>
        <div style={{
          marginTop: 14,
          padding: "10px 14px",
          background: "var(--color-bg)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--radius-sm)",
          fontSize: 12,
          color: "var(--color-text-secondary)"
        }}>
          Note: Votes are hidden during commit phase — only VP totals revealed after "Reveal Phase".
        </div>
      </div>

      {/* Voting panel */}
      <div className="tabs">
        <button className={`tab${activeTab === "vote" ? " active" : ""}`} onClick={() => setActiveTab("vote")}>
          Cast Vote
        </button>
        <button className={`tab${activeTab === "finalize" ? " active" : ""}`} onClick={() => setActiveTab("finalize")}>
          Finalize
        </button>
      </div>

      {activeTab === "vote" && (
        <div className="card">
          {phase === "commit" && (
            <div className="form-section">
              <div className="card-header" style={{ marginBottom: 0 }}>
                <span className="card-title">Commit Phase</span>
                <span className="card-meta">{timeLeft(MOCK_VLA.commitEndsAt)} remaining</span>
              </div>
              <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
                Choose your side secretly. A hash of your vote will be submitted on-chain.
              </div>

              <div className="field">
                <label className="field-label">Your Vote</label>
                <div style={{ display: "flex", gap: 8 }}>
                  <button
                    className={`btn${side === "proposer" ? " btn-primary" : " btn-secondary"}`}
                    style={{ flex: 1 }}
                    onClick={() => setSide("proposer")}
                  >
                    ✓ Proposer is Correct
                  </button>
                  <button
                    className={`btn${side === "disputer" ? " btn-primary" : " btn-secondary"}`}
                    style={{ flex: 1 }}
                    onClick={() => setSide("disputer")}
                  >
                    ✗ Disputer is Correct
                  </button>
                </div>
              </div>

              <div className="field">
                <label className="field-label">Secret Salt</label>
                <input
                  type="text"
                  className="input mono"
                  placeholder="Random number — save this to reveal later"
                  value={salt}
                  onChange={(e) => setSalt(e.target.value)}
                />
                <span className="field-hint">⚠ You must reveal this exact salt to have your vote counted.</span>
              </div>

              {side && salt && (
                <div style={{
                  background: "var(--color-bg)",
                  border: "1px solid var(--color-border)",
                  borderRadius: "var(--radius-sm)",
                  padding: "10px 12px",
                  fontSize: 12,
                }}>
                  <div style={{ color: "var(--color-text-secondary)", marginBottom: 4 }}>Commit hash preview</div>
                  <div className="mono" style={{ wordBreak: "break-all", color: "var(--color-text-primary)" }}>
                    sha256({side === "proposer" ? "0x00" : "0x01"} || {salt}) = ████████████████████
                  </div>
                </div>
              )}

              <button
                className="btn btn-primary btn-lg btn-full"
                disabled={!connected || !side || !salt}
                onClick={() => { setCommitted(true); setPhase("reveal"); }}
              >
                {connected ? "Submit Commit →" : "Connect Wallet First"}
              </button>
            </div>
          )}

          {phase === "reveal" && (
            <div className="form-section">
              <div className="card-header" style={{ marginBottom: 0 }}>
                <span className="card-title">Reveal Phase</span>
                <span className="card-meta">{timeLeft(MOCK_VLA.revealEndsAt)} remaining</span>
              </div>
              <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
                Reveal your vote using the same side and salt you committed.
              </div>
              <div style={{
                background: "var(--color-bg)",
                border: "1px solid var(--color-border)",
                borderRadius: "var(--radius-sm)",
                padding: 12,
                fontSize: 12.5,
              }}>
                <div style={{ marginBottom: 8 }}>
                  <span style={{ color: "var(--color-text-secondary)" }}>Committed side: </span>
                  <span style={{ fontWeight: 600, textTransform: "capitalize" }}>{side}</span>
                </div>
                <div>
                  <span style={{ color: "var(--color-text-secondary)" }}>Your salt: </span>
                  <span className="mono">{salt}</span>
                </div>
              </div>
              <button
                className="btn btn-primary btn-lg btn-full"
                disabled={!connected}
              >
                Reveal Vote →
              </button>
            </div>
          )}
        </div>
      )}

      {activeTab === "finalize" && (
        <div className="card">
          <div className="form-section">
            <div className="card-header" style={{ marginBottom: 0 }}>
              <span className="card-title">Finalize Round</span>
            </div>
            <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
              Available after the reveal window closes. Checks quorum and determines the winner.
            </div>
            <div style={{
              background: "var(--color-bg)",
              border: "1px solid var(--color-border)",
              borderRadius: "var(--radius-sm)",
              padding: "12px 14px",
              fontSize: 12.5,
            }}>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
                <span style={{ color: "var(--color-text-secondary)" }}>Current leader</span>
                <span style={{ fontWeight: 600 }}>Proposer ({proposerPct.toFixed(1)}%)</span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
                <span style={{ color: "var(--color-text-secondary)" }}>Min quorum</span>
                <span>protocol-defined</span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span style={{ color: "var(--color-text-secondary)" }}>Reveal window closes</span>
                <span>{timeLeft(MOCK_VLA.revealEndsAt)}</span>
              </div>
            </div>
            <button className="btn btn-primary btn-lg btn-full" disabled={!connected}>
              Finalize VLA Round →
            </button>
            <button className="btn btn-secondary btn-full" disabled={!connected}>
              Claim Arbiter Reward
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
