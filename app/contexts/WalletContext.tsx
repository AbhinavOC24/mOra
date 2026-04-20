import React, { createContext, useContext, useState, useCallback } from "react";
import { Connection, PublicKey } from "@solana/web3.js";
import { RPC_URL } from "../lib/protocol";

interface WalletCtx {
  publicKey: PublicKey | null;
  connected: boolean;
  connect: () => Promise<void>;
  disconnect: () => void;
  connection: Connection;
}

const WalletContext = createContext<WalletCtx>({
  publicKey: null,
  connected: false,
  connect: async () => {},
  disconnect: () => {},
  connection: new Connection(RPC_URL, "confirmed"),
});

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [publicKey, setPublicKey] = useState<PublicKey | null>(null);
  const connection = new Connection(RPC_URL, "confirmed");

  const connect = useCallback(async () => {
    const win = window as any;
    const provider = win.phantom?.solana || win.solflare || win.solana;
    if (!provider) {
      alert("No Solana wallet found. Please install Phantom or Solflare.");
      return;
    }
    try {
      const resp = await provider.connect();
      setPublicKey(new PublicKey(resp.publicKey.toString()));
    } catch (err) {
      console.error("Wallet connect error:", err);
    }
  }, []);

  const disconnect = useCallback(() => {
    const win = window as any;
    const provider = win.phantom?.solana || win.solflare || win.solana;
    provider?.disconnect?.();
    setPublicKey(null);
  }, []);

  return (
    <WalletContext.Provider value={{ publicKey, connected: !!publicKey, connect, disconnect, connection }}>
      {children}
    </WalletContext.Provider>
  );
}

export const useWallet = () => useContext(WalletContext);
