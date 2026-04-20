import React, { createContext, useContext, useState, useCallback, useMemo } from "react";
import { Connection, PublicKey, Transaction, VersionedTransaction } from "@solana/web3.js";
import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { RPC_URL, PROGRAM_ID } from "../lib/protocol";
import idl from "../lib/oo_core.json";

interface WalletCtx {
  publicKey: PublicKey | null;
  connected: boolean;
  connect: () => Promise<void>;
  disconnect: () => void;
  connection: Connection;
  program: Program | null;
}

const WalletContext = createContext<WalletCtx>({
  publicKey: null,
  connected: false,
  connect: async () => {},
  disconnect: () => {},
  connection: new Connection(RPC_URL, "confirmed"),
  program: null,
});

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [publicKey, setPublicKey] = useState<PublicKey | null>(null);
  const [wallet, setWallet] = useState<any>(null);
  
  const connection = useMemo(() => new Connection(RPC_URL, "confirmed"), []);

  const connect = useCallback(async () => {
    const win = window as any;
    const provider = win.phantom?.solana || win.solflare || win.solana;
    if (!provider) {
      alert("No Solana wallet found. Please install Phantom or Solflare.");
      return;
    }
    try {
      const resp = await provider.connect();
      const pubkey = new PublicKey(resp.publicKey.toString());
      setPublicKey(pubkey);
      setWallet(provider);
    } catch (err) {
      console.error("Wallet connect error:", err);
    }
  }, []);

  const disconnect = useCallback(() => {
    const win = window as any;
    const provider = win.phantom?.solana || win.solflare || win.solana;
    provider?.disconnect?.();
    setPublicKey(null);
    setWallet(null);
  }, []);

  const program = useMemo(() => {
    if (!publicKey || !wallet) return null;
    
    // Wrap simple provider object into Anchor Wallet interface
    const anchorWallet = {
      publicKey,
      signTransaction: (tx: any) => wallet.signTransaction(tx),
      signAllTransactions: (txs: any) => wallet.signAllTransactions(txs),
    };

    const provider = new AnchorProvider(connection, anchorWallet, AnchorProvider.defaultOptions());
    return new Program(idl as Idl, provider);
  }, [publicKey, wallet, connection]);

  return (
    <WalletContext.Provider value={{ publicKey, connected: !!publicKey, connect, disconnect, connection, program }}>
      {children}
    </WalletContext.Provider>
  );
}

export const useWallet = () => useContext(WalletContext);

