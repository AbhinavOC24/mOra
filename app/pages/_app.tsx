import type { AppProps } from "next/app";
import { WalletProvider } from "../contexts/WalletContext";
import "../styles/globals.css";

export default function App({ Component, pageProps }: AppProps) {
  return (
    <WalletProvider>
      <Component {...pageProps} />
    </WalletProvider>
  );
}
