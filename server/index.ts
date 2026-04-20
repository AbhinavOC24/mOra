import express from 'express';
import cors from 'cors';

const app = express();
app.use(cors());
app.use(express.json());

// Helper to get time relative to now for dynamic demo data
const getRelativeTime = (secondsOff: number) => {
  return Math.floor(Date.now() / 1000) + secondsOff;
};

// In-memory data store for the hackathon demo
let assertions = [
  {
    id: "1001",
    question: "Will SOL close above $200 on Apr 30?",
    status: "proposed",
    answerType: "YesNo",
    proposedAnswer: "Yes",
    bond: "10 MORA",
    reward: "50 MORA",
    // Ends in 5 minutes from server start
    livenessEndsAt: getRelativeTime(300), 
  },
  {
    id: "1002",
    question: "What was BTC price at UTC 00:00 Apr 20?",
    status: "disputed",
    answerType: "Number",
    proposedAnswer: "64000",
    bond: "20 MORA",
    reward: "100 MORA",
    // Passed 2 hours ago
    livenessEndsAt: getRelativeTime(-7200), 
  },
  {
    id: "1003",
    question: "Did the Solana network have >99.9% uptime in Q1 2025?",
    status: "resolved",
    answerType: "YesNo",
    proposedAnswer: "Yes",
    bond: "15 MORA",
    reward: "75 MORA",
    // Passed long ago
    livenessEndsAt: getRelativeTime(-86400), 
  },
];

app.get('/api/assertions', (req, res) => {
  res.json(assertions);
});

app.post('/api/assertions/:id/resolve', (req, res) => {
  const { id } = req.params;
  const index = assertions.findIndex(a => a.id === id);
  if (index !== -1) {
    const assertion = assertions[index];
    if (assertion.status === 'proposed' && assertion.livenessEndsAt < Math.floor(Date.now() / 1000)) {
       assertions[index].status = 'resolved';
       res.json({ success: true, assertion: assertions[index] });
    } else {
       res.status(400).json({ error: 'Cannot resolve yet' });
    }
  } else {
    res.status(404).json({ error: 'Not found' });
  }
});

const PORT = 3001;
app.listen(PORT, () => {
  console.log(`Backend server running on http://localhost:${PORT}`);
});
