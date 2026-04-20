import { useState } from "react";
import { useWallet } from "../contexts/WalletContext";

export default function AdminPage() {
  const { connected } = useWallet();
  const [form, setForm] = useState({
    minBond: "10",
    arbiterBps: "500",
    burnVault: "",
    minArbiterVp: "1000000",
    commitWindow: "3600",
    vlaEnabled: true,
    moraMint: "",
    minLock: "86400",
    maxLock: "63072000",
  });

  const set = (k: string, v: string | boolean) => setForm((f) => ({ ...f, [k]: v }));

  return (
    <div className="page">
      <div className="page-header">
        <h1>Admin</h1>
        <p>Initialize and configure protocol parameters. Requires admin keypair.</p>
      </div>

      <div style={{ display: "flex", gap: 10, marginBottom: 16 }}>
        <span className="badge badge-white">Admin Only</span>
        <span style={{ fontSize: 12, color: "var(--color-text-tertiary)", alignSelf: "center" }}>
          These instructions can only be called by the configured admin wallet.
        </span>
      </div>

      {/* Global Config */}
      <div className="card" style={{ marginBottom: 12 }}>
        <div className="card-header">
          <span className="card-title">Global Config</span>
          <span className="card-meta">initialize_global_config</span>
        </div>
        <div className="form-section">
          <div className="grid-2">
            <div className="field">
              <label className="field-label">Min Proposer Bond (MORA)</label>
              <input type="number" className="input" value={form.minBond} onChange={(e) => set("minBond", e.target.value)} />
            </div>
            <div className="field">
              <label className="field-label">Arbiter Reward (bps)</label>
              <input type="number" className="input" value={form.arbiterBps} onChange={(e) => set("arbiterBps", e.target.value)} />
              <span className="field-hint">{(Number(form.arbiterBps) / 100).toFixed(2)}%</span>
            </div>
          </div>
          <div className="field">
            <label className="field-label">Burn Vault Address</label>
            <input type="text" className="input mono" placeholder="Pubkey" value={form.burnVault} onChange={(e) => set("burnVault", e.target.value)} />
          </div>
          <div className="grid-2">
            <div className="field">
              <label className="field-label">Min Arbiter Voting Power</label>
              <input type="number" className="input" value={form.minArbiterVp} onChange={(e) => set("minArbiterVp", e.target.value)} />
            </div>
            <div className="field">
              <label className="field-label">VLA Commit Window (sec)</label>
              <input type="number" className="input" value={form.commitWindow} onChange={(e) => set("commitWindow", e.target.value)} />
            </div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <input
              type="checkbox"
              id="vlaEnabled"
              checked={form.vlaEnabled}
              onChange={(e) => set("vlaEnabled", e.target.checked)}
              style={{ width: 14, height: 14, accentColor: "#fff", cursor: "pointer" }}
            />
            <label htmlFor="vlaEnabled" style={{ fontSize: 13, cursor: "pointer" }}>VLA Enabled</label>
          </div>
          <button className="btn btn-primary" disabled={!connected}>
            Initialize Global Config →
          </button>
        </div>
      </div>

      {/* Locking Config */}
      <div className="card">
        <div className="card-header">
          <span className="card-title">Locking Config</span>
          <span className="card-meta">initialize_locking_config</span>
        </div>
        <div className="form-section">
          <div className="field">
            <label className="field-label">MORA Mint Address</label>
            <input type="text" className="input mono" placeholder="SPL Token mint" value={form.moraMint} onChange={(e) => set("moraMint", e.target.value)} />
            <span className="field-hint">Any custom SPL token (Token-2022 compatible)</span>
          </div>
          <div className="grid-2">
            <div className="field">
              <label className="field-label">Min Lock Duration (sec)</label>
              <input type="number" className="input" value={form.minLock} onChange={(e) => set("minLock", e.target.value)} />
              <span className="field-hint">{(Number(form.minLock) / 86400).toFixed(1)} days</span>
            </div>
            <div className="field">
              <label className="field-label">Max Lock Duration (sec)</label>
              <input type="number" className="input" value={form.maxLock} onChange={(e) => set("maxLock", e.target.value)} />
              <span className="field-hint">{(Number(form.maxLock) / 86400).toFixed(0)} days</span>
            </div>
          </div>
          <button className="btn btn-primary" disabled={!connected}>
            Initialize Locking Config →
          </button>
        </div>
      </div>
    </div>
  );
}
