import { useState } from "react";
import { useWallet } from "../contexts/WalletContext";
import { calcVotingPower, MAX_LOCK_DURATION } from "../lib/protocol";

const MAX_DURATION = BigInt(MAX_LOCK_DURATION);

export default function LockPage() {
  const { connected } = useWallet();
  const [activeTab, setActiveTab] = useState<"lock" | "manage">("lock");
  const [amount, setAmount] = useState("");
  const [duration, setDuration] = useState(63072000); // 2 years

  const now = BigInt(Math.floor(Date.now() / 1000));
  const lockEnd = now + BigInt(duration);

  // Compute preview voting power
  const previewVp =
    amount
      ? calcVotingPower(
          BigInt(Math.floor(Number(amount) * 1e9)),
          lockEnd,
          now,
          MAX_DURATION
        )
      : 0n;

  const vpDisplay = amount
    ? (Number(previewVp) / 1e9).toFixed(4)
    : "—";

  const pct = (duration / MAX_LOCK_DURATION) * 100;

  const durations = [
    { label: "3 months", secs: 7776000 },
    { label: "6 months", secs: 15552000 },
    { label: "1 year", secs: 31536000 },
    { label: "2 years", secs: 63072000 },
  ];

  return (
    <div className="page">
      <div className="page-header">
        <h1>veMORA Lock</h1>
        <p>Lock MORA to gain voting power for VLA arbitration.</p>
      </div>

      <div className="tabs">
        <button className={`tab${activeTab === "lock" ? " active" : ""}`} onClick={() => setActiveTab("lock")}>
          Create Lock
        </button>
        <button className={`tab${activeTab === "manage" ? " active" : ""}`} onClick={() => setActiveTab("manage")}>
          Manage Position
        </button>
      </div>

      {activeTab === "lock" && (
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          {/* Voting power preview */}
          <div className="grid-2">
            <div className="lock-gauge">
              <div className="lock-gauge-label">Preview veMORA</div>
              <div className="lock-gauge-value">{vpDisplay}</div>
              <div className="lock-gauge-sub">Voting power at lock</div>
            </div>
            <div className="lock-gauge">
              <div className="lock-gauge-label">Lock Efficiency</div>
              <div className="lock-gauge-value">{pct.toFixed(0)}%</div>
              <div className="lock-gauge-sub">of max (2 years)</div>
            </div>
          </div>

          {/* Decay bar */}
          <div className="card">
            <div className="card-header">
              <span className="card-title">Power Decay</span>
              <span className="card-meta">Linear model · slope-bias</span>
            </div>
            <div style={{ fontSize: 12, color: "var(--color-text-secondary)", marginBottom: 12 }}>
              vp = A × (t<sub>e</sub> − t) / T<sub>max</sub>
            </div>
            <div className="vote-bars">
              {[0, 25, 50, 75, 100].map((pct, i) => (
                <div key={pct} className="vote-bar-row">
                  <div className="vote-bar-label">
                    <span>t + {["0", "25%", "50%", "75%", "100%"][i]} of lock</span>
                    <span>{(100 - pct).toFixed(0)}% power</span>
                  </div>
                  <div className="vote-bar-track">
                    <div className="vote-bar-fill" style={{ width: `${100 - pct}%` }} />
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Form */}
          <div className="card">
            <div className="card-header"><span className="card-title">Lock Parameters</span></div>
            <div className="form-section">
              <div className="field">
                <label className="field-label">Amount (MORA)</label>
                <input
                  type="number"
                  className="input"
                  placeholder="e.g. 1000"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  min={1}
                />
              </div>

              <div className="field">
                <label className="field-label">Lock Duration</label>
                <div style={{ display: "flex", gap: 6, marginBottom: 8 }}>
                  {durations.map((d) => (
                    <button
                      key={d.label}
                      className={`btn btn-sm${duration === d.secs ? " btn-primary" : " btn-secondary"}`}
                      onClick={() => setDuration(d.secs)}
                    >
                      {d.label}
                    </button>
                  ))}
                </div>
                <div className="progress-track">
                  <div className="progress-fill" style={{ width: `${pct}%` }} />
                </div>
              </div>

              <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12, color: "var(--color-text-secondary)", padding: "8px 0" }}>
                <span>Unlock Date</span>
                <span style={{ color: "var(--color-text-primary)" }}>
                  {new Date(Date.now() + duration * 1000).toLocaleDateString()}
                </span>
              </div>

              <button className="btn btn-primary btn-lg btn-full" disabled={!connected || !amount}>
                {connected ? "Lock MORA →" : "Connect Wallet First"}
              </button>
            </div>
          </div>
        </div>
      )}

      {activeTab === "manage" && (
        <div className="card">
          {connected ? (
            <div style={{ textAlign: "center", padding: "32px 0" }}>
              <div style={{ fontSize: 13, color: "var(--color-text-secondary)", marginBottom: 16 }}>
                No active lock position found.
              </div>
              <button className="btn btn-secondary" onClick={() => setActiveTab("lock")}>
                Create a Lock →
              </button>
            </div>
          ) : (
            <div className="empty-state">
              <div className="empty-state-title">Connect your wallet</div>
              <div className="empty-state-desc">Connect to see your lock position.</div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
