import { useState } from "react";
import { useWallet } from "../contexts/WalletContext";

export default function NewAssertionPage() {
  const { connected } = useWallet();
  const [form, setForm] = useState({
    question: "",
    answerType: "YesNo",
    liveness: "3600",
    dispute: "3600",
    bond: "",
    reward: "",
    metadata: "",
  });

  const set = (k: string, v: string) => setForm((f) => ({ ...f, [k]: v }));

  const canSubmit = connected && form.question && form.bond && form.reward;

  return (
    <div className="page">
      <div className="page-header">
        <h1>New Assertion</h1>
        <p>Post a question on-chain to be answered by the mORA protocol.</p>
      </div>

      <div className="card">
        <div className="card-header">
          <span className="card-title">Request Details</span>
          <span className="card-meta">Step 1 of 1</span>
        </div>

        <div className="form-section">
          <div className="field">
            <label className="field-label">Question *</label>
            <textarea
              className="input"
              placeholder='e.g. "Was BTC above $50k on Apr 20, 2025?"'
              value={form.question}
              onChange={(e) => set("question", e.target.value)}
              style={{ minHeight: 80 }}
            />
            <span className="field-hint">Max 500 characters. Be precise.</span>
          </div>

          <div className="grid-2">
            <div className="field">
              <label className="field-label">Answer Type *</label>
              <select className="select" value={form.answerType} onChange={(e) => set("answerType", e.target.value)}>
                <option value="YesNo">Yes / No</option>
                <option value="Number">Number</option>
                <option value="String">String</option>
              </select>
            </div>
            <div className="field">
              <label className="field-label">Liveness Window (seconds) *</label>
              <input
                type="number"
                className="input"
                value={form.liveness}
                onChange={(e) => set("liveness", e.target.value)}
                min={3600}
                max={604800}
              />
              <span className="field-hint">Min 1h, max 7d</span>
            </div>
          </div>

          <div className="grid-2">
            <div className="field">
              <label className="field-label">Dispute Period (seconds) *</label>
              <input
                type="number"
                className="input"
                value={form.dispute}
                onChange={(e) => set("dispute", e.target.value)}
                min={3600}
                max={604800}
              />
            </div>
            <div className="field" />
          </div>

          <div className="divider" />

          <div style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-primary)", marginBottom: 4 }}>
            Bond & Reward
          </div>
          <div style={{ fontSize: 12, color: "var(--color-text-secondary)", marginBottom: 12 }}>
            Both are locked in escrow until the assertion is resolved.
          </div>

          <div className="grid-2">
            <div className="field">
              <label className="field-label">Your Bond (MORA) *</label>
              <input
                type="number"
                className="input"
                placeholder="10"
                value={form.bond}
                onChange={(e) => set("bond", e.target.value)}
                min={1}
              />
              <span className="field-hint">Refunded on resolution</span>
            </div>
            <div className="field">
              <label className="field-label">Reward Pool (MORA) *</label>
              <input
                type="number"
                className="input"
                placeholder="50"
                value={form.reward}
                onChange={(e) => set("reward", e.target.value)}
                min={1}
              />
              <span className="field-hint">Paid to proposer or arbiters</span>
            </div>
          </div>

          <div className="field">
            <label className="field-label">Metadata (optional)</label>
            <input
              type="text"
              className="input"
              placeholder='e.g. "source: CoinGecko, date: 2025-04-20"'
              value={form.metadata}
              onChange={(e) => set("metadata", e.target.value)}
            />
          </div>

          {/* Summary */}
          {(form.bond || form.reward) && (
            <div style={{
              background: "var(--color-bg)",
              border: "1px solid var(--color-border)",
              borderRadius: "var(--radius-md)",
              padding: "12px 14px",
              fontSize: 12.5,
              color: "var(--color-text-secondary)",
            }}>
              <div style={{ marginBottom: 6, fontWeight: 600, color: "var(--color-text-primary)" }}>
                Transaction Summary
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 4 }}>
                <span>Total locked in escrow</span>
                <span style={{ color: "var(--color-text-primary)", fontWeight: 500 }}>
                  {(Number(form.bond || 0) + Number(form.reward || 0)).toFixed(2)} MORA
                </span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span>Network fee (est.)</span>
                <span>~0.000005 SOL</span>
              </div>
            </div>
          )}

          <button
            className="btn btn-primary btn-lg btn-full"
            disabled={!canSubmit}
          >
            {!connected ? "Connect Wallet First" : "Submit Assertion →"}
          </button>
        </div>
      </div>
    </div>
  );
}
