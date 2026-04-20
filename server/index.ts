import express from 'express';
import cors from 'cors';
import { PrismaClient } from '@prisma/client';
import { PrismaBetterSqlite3 } from '@prisma/adapter-better-sqlite3';
import { Connection, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, Idl } from '@coral-xyz/anchor';
import idl from './oo_core.json';

const app = express();
const adapter = new PrismaBetterSqlite3({ url: "file:./prisma/dev.db" });
const prisma = new PrismaClient({ adapter });

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const provider = new AnchorProvider(connection, {} as any, AnchorProvider.defaultOptions());
const program = new Program(idl as Idl, provider);

app.use(cors());
app.use(express.json());

// Background indexer syncs Solana accounts to Prisma
async function runIndexer() {
  console.log("[Indexer] Syncing with Solana devnet...");
  try {
    const accounts = await (program.account as any).assertionRequest.all();
    console.log(`[Indexer] Found ${accounts.length} assertion accounts.`);

    for (const { publicKey, account } of accounts) {
      const data = account as any;
      
      // Map AnswerValue enum to string
      const formatAnswer = (val: any) => {
        if (!val) return null;
        if (val.yesNo !== undefined) return val.yesNo ? "Yes" : "No";
        if (val.number !== undefined) return val.number.toString();
        if (val.string !== undefined) return val.string;
        return JSON.stringify(val);
      };

      await prisma.assertion.upsert({
        where: { id: publicKey.toBase58() },
        update: {
          assertionId: data.assertionId.toString(),
          requester: data.requester.toBase58(),
          question: data.question,
          requesterBond: data.requesterBondAmount.toString(),
          reward: data.rewardAmount.toString(),
          answerType: Object.keys(data.answerType)[0],
          livenessPeriod: Number(data.livenessPeriod),
          disputePeriod: Number(data.disputePeriod),
          status: Object.keys(data.status)[0],
          requestedAt: Number(data.requestedAt),
          resolvedAt: data.resolvedAt ? Number(data.resolvedAt) : null,
          proposer: data.proposer ? data.proposer.toBase58() : null,
          proposedAt: data.proposedAt ? Number(data.proposedAt) : null,
          proposerBond: data.proposerBond ? data.proposerBond.toString() : null,
          disputer: data.disputer ? data.disputer.toBase58() : null,
          disputerBond: data.disputerBond ? data.disputerBond.toString() : null,
          proposedAnswer: formatAnswer(data.proposedAnswer),
          finalAnswer: formatAnswer(data.finalAnswer),
          metadataUrl: data.metadata || null,
          updatedAt: new Date()
        },
        create: {
          id: publicKey.toBase58(),
          assertionId: data.assertionId.toString(),
          requester: data.requester.toBase58(),
          question: data.question,
          requesterBond: data.requesterBondAmount.toString(),
          reward: data.rewardAmount.toString(),
          answerType: Object.keys(data.answerType)[0],
          livenessPeriod: Number(data.livenessPeriod),
          disputePeriod: Number(data.disputePeriod),
          status: Object.keys(data.status)[0],
          requestedAt: Number(data.requestedAt),
          resolvedAt: data.resolvedAt ? Number(data.resolvedAt) : null,
          proposer: data.proposer ? data.proposer.toBase58() : null,
          proposedAt: data.proposedAt ? Number(data.proposedAt) : null,
          proposerBond: data.proposerBond ? data.proposerBond.toString() : null,
          disputer: data.disputer ? data.disputer.toBase58() : null,
          disputerBond: data.disputerBond ? data.disputerBond.toString() : null,
          proposedAnswer: formatAnswer(data.proposedAnswer),
          finalAnswer: formatAnswer(data.finalAnswer),
          metadataUrl: data.metadata || null,
          updatedAt: new Date()
        }
      });
    }
    console.log("[Indexer] Sync complete.");
  } catch (error: any) {
    console.error("[Indexer] Sync failed:", error);
  }
}

// Initial sync on start
runIndexer();
setInterval(runIndexer, 30000); // Poll every 30 seconds


// API Endpoints reading from Prisma DB
app.get('/api/assertions', async (req, res) => {
  try {
    const assertions = await prisma.assertion.findMany({
      orderBy: { requestedAt: 'desc' }
    });

    // Format them for the frontend
    const formatted = assertions.map(a => {
      let livenessEndsAt = 0;
      if (a.status === 'proposed' && a.proposedAt) {
        livenessEndsAt = a.proposedAt + a.livenessPeriod;
      } else if (a.status === 'proposed') {
        // Fallback for mocked data
        livenessEndsAt = Math.floor(Date.now() / 1000) + 300;
      }

      return {
        id: a.assertionId,
        pubkey: a.id,
        question: a.question,
        status: a.status.toLowerCase(),
        answerType: a.answerType,
        proposedAnswer: a.proposedAnswer || "",
        bond: (Number(a.proposerBond || a.requesterBond || 10000000000) / 1e9).toFixed(2) + " MORA",
        reward: (Number(a.reward || 50000000000) / 1e9).toFixed(2) + " MORA",
        livenessEndsAt,
      };
    });

    res.json(formatted);
  } catch (error) {
    console.error(error);
    res.status(500).json({ error: "DB Error" });
  }
});

// Endpoint for frontend to push indexing updates (Hackathon fallback for WebSocket)
app.post('/api/indexer/sync', async (req, res) => {
  try {
    const data = req.body;
    await prisma.assertion.upsert({
      where: { id: data.id },
      update: { ...data, updatedAt: new Date() },
      create: { ...data, updatedAt: new Date() }
    });
    res.json({ success: true });
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

app.post('/api/assertions/:id/resolve', async (req, res) => {
  const { id } = req.params;
  try {
    const assertion = await prisma.assertion.findUnique({ where: { assertionId: id } });
    if (assertion) {
      await prisma.assertion.update({
        where: { assertionId: id },
        data: { status: 'Resolved', resolvedAt: Math.floor(Date.now() / 1000) }
      });

      res.json({ success: true });
    } else {
      res.status(404).json({ error: 'Not found' });
    }
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

const PORT = 3001;
app.listen(PORT, () => {
  console.log(`Indexer & API server running on http://localhost:${PORT}`);
});
