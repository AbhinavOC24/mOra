// Overview page — protocol dashboard
export default function OverviewPage() {
  const stats = [
    { label: "Total Assertions", value: "—", sub: "On-chain" },
    { label: "veMORA Locked", value: "—", sub: "MORA tokens" },
    { label: "Active VLA Rounds", value: "—", sub: "Disputes" },
    { label: "Program", value: "Devnet", sub: "E7TZ…Pnat" },
  ];

  return (
    <div className="page">
      <div className="page-header">
        <h1>Protocol Overview</h1>
        <p>Real-time view of the mORA optimistic oracle on Solana devnet.</p>
      </div>

      <div className="stats-grid">
        {stats.map((s) => (
          <div className="stat-card" key={s.label}>
            <div className="stat-label">{s.label}</div>
            <div className="stat-value">{s.value}</div>
            <div className="stat-sub">{s.sub}</div>
          </div>
        ))}
      </div>

      <div className="card">
        <div className="card-header">
          <span className="card-title">How It Works</span>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          {[
            { step: "01", title: "Request", desc: "Post a question on-chain with a MORA bond and reward." },
            { step: "02", title: "Propose", desc: "A proposer submits an answer within the liveness window and stakes a bond." },
            { step: "03", title: "Challenge", desc: "Any party can dispute the proposal by posting their own bond." },
            { step: "04", title: "Arbitrate", desc: "veMORA holders participate in a commit-reveal vote to determine the canonical answer." },
            { step: "05", title: "Resolve", desc: "Bonds are slashed and rewards distributed pro-rata to correct arbiters." },
          ].map((item) => (
            <div key={item.step} style={{ display: "flex", gap: 14, alignItems: "flex-start" }}>
              <div style={{
                width: 28, height: 28, flexShrink: 0,
                border: "1px solid var(--color-border)",
                borderRadius: 6,
                display: "flex", alignItems: "center", justifyContent: "center",
                fontSize: 10, fontWeight: 700, color: "var(--color-text-tertiary)",
                letterSpacing: "0.05em"
              }}>
                {item.step}
              </div>
              <div>
                <div style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-primary)", marginBottom: 2 }}>
                  {item.title}
                </div>
                <div style={{ fontSize: 12.5, color: "var(--color-text-secondary)" }}>
                  {item.desc}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div style={{ marginTop: 12, padding: "12px 0", borderTop: "1px solid var(--color-border)" }}>
        <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
          {["Anchor 0.31.1", "Solana 2.x", "SPL Token-2022", "veMORA Model", "Commit-Reveal VLA"].map((tag) => (
            <span key={tag} className="badge badge-default">{tag}</span>
          ))}
        </div>
      </div>
    </div>
  );
}
