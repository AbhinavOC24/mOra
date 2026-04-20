import { useState } from "react";
import { useWallet } from "../contexts/WalletContext";
import { getAssertionPda, getGlobalPda, PROGRAM_ID, MORA_MINT } from "../lib/protocol";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";

export default function NewAssertionPage() {
  const { connected, program, publicKey } = useWallet();
  const [loading, setLoading] = useState(false);
  const [form, setForm] = useState({
    question: "",
    answerType: "YesNo",
    liveness: "3600",
    dispute: "3600",
    bond: "10",
    reward: "50",
    metadata: "",
  });

  const set = (k: string, v: string) => setForm((f) => ({ ...f, [k]: v }));

  const canSubmit = connected && form.question && form.bond && form.reward && !loading;

  const handleSubmit = async () => {
    if (!program || !publicKey) return;

    setLoading(true);
    try {
      // For this hackathon, we use a random u64 as the assertionId
      const assertionId = new anchor.BN(Math.floor(Math.random() * 1e12));
      const assertionPda = getAssertionPda(BigInt(assertionId.toString()), PROGRAM_ID);
      
      const [assertionEscrow] = PublicKey.findProgramAddressSync(
        [Buffer.from("assertion_escrow"), assertionId.toArrayLike(Buffer, "le", 8)],
        PROGRAM_ID
      );

      const requesterMoraAta = getAssociatedTokenAddressSync(MORA_MINT, publicKey);

      const tx = await (program.rpc as any).requestAssertion(
        assertionId,
        form.question,
        { [form.answerType.charAt(0).toLowerCase() + form.answerType.slice(1)]: {} },
        new anchor.BN(form.liveness),
        new anchor.BN(form.dispute),
        form.metadata || null,
        new anchor.BN(Number(form.bond) * 1e9),
        new anchor.BN(Number(form.reward) * 1e9),
        {
          accounts: {
            requester: publicKey,
            assertionRequest: assertionPda,
            assertionEscrow,
            requesterMoraAta,
            moraMint: MORA_MINT,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          },
        }
      );

      console.log("Assertion requested! Tx:", tx);
      alert("Assertion successfully posted on-chain!");
      
      // Optional: Proactively notify backend to sync faster
      fetch('http://localhost:3001/api/indexer/sync', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: assertionPda.toBase58(), skipSync: true }) // Hint to backend
      }).catch(e => console.warn("Backend notification failed", e));

    } catch (err: any) {
      console.error("Submission failed:", err);
      alert("On-chain transaction failed: " + (err.message || err));
    } finally {
      setLoading(false);
    }
  };

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
          {/* ... existing fields ... */}
          <div className="field">
            <label className="field-label">Question *</label>
            <textarea
              className="input"
              placeholder='e.g. "Was BTC above $50k on Apr 20, 2025?"'
              value={form.question}
              onChange={(e) => set("question", e.target.value)}
              style={{ minHeight: 80 }}
            />
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
              <input type="number" className="input" value={form.liveness} onChange={(e) => set("liveness", e.target.value)} />
            </div>
          </div>

          <div className="grid-2">
            <div className="field">
              <label className="field-label">Dispute Period (seconds) *</label>
              <input type="number" className="input" value={form.dispute} onChange={(e) => set("dispute", e.target.value)} />
            </div>
          </div>

          <div className="grid-2">
            <div className="field">
              <label className="field-label">Your Bond (MORA) *</label>
              <input type="number" className="input" value={form.bond} onChange={(e) => set("bond", e.target.value)} />
            </div>
            <div className="field">
              <label className="field-label">Reward Pool (MORA) *</label>
              <input type="number" className="input" value={form.reward} onChange={(e) => set("reward", e.target.value)} />
            </div>
          </div>

          <button
            className="btn btn-primary btn-lg btn-full"
            disabled={!canSubmit}
            onClick={handleSubmit}
          >
            {loading ? "Waiting for Transaction..." : !connected ? "Connect Wallet First" : "Submit Assertion →"}
          </button>
        </div>
      </div>
    </div>
  );
}

