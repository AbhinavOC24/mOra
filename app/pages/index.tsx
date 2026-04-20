import { useState } from "react";
import Head from "next/head";
import Layout from "../components/Layout";
import OverviewPage from "../components/OverviewPage";
import AssertionsPage from "../components/AssertionsPage";
import NewAssertionPage from "../components/NewAssertionPage";
import VlaPage from "../components/VlaPage";
import LockPage from "../components/LockPage";
import AdminPage from "../components/AdminPage";

type Page = "overview" | "assertions" | "new-assertion" | "vla" | "lock" | "admin";

const PAGE_TITLES: Record<Page, string> = {
  overview: "Overview",
  assertions: "Assertions",
  "new-assertion": "New Assertion",
  vla: "VLA / Vote",
  lock: "veMORA Lock",
  admin: "Admin",
};

export default function Home() {
  const [activePage, setActivePage] = useState<Page>("overview");

  const navigate = (page: Page) => setActivePage(page);

  return (
    <>
      <Head>
        <title>mORA — Optimistic Oracle</title>
        <meta name="description" content="mORA is a decentralized optimistic oracle protocol on Solana, secured by veMORA holders." />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;450;500;600;700&display=swap"
          rel="stylesheet"
        />
      </Head>

      <Layout
        activePage={activePage}
        onNavigate={navigate}
        pageTitle={PAGE_TITLES[activePage]}
      >
        {activePage === "overview" && <OverviewPage />}
        {activePage === "assertions" && <AssertionsPage onNewAssertion={() => navigate("new-assertion")} />}
        {activePage === "new-assertion" && <NewAssertionPage />}
        {activePage === "vla" && <VlaPage />}
        {activePage === "lock" && <LockPage />}
        {activePage === "admin" && <AdminPage />}
      </Layout>
    </>
  );
}
