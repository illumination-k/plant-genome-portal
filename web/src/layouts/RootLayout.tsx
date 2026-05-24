import type { ReactElement } from "react";
import SiteHeader from "@/layouts/SiteHeader";
import MainContent from "@/layouts/MainContent";

const RootLayout = (): ReactElement => (
  <div className="min-h-screen bg-stone-50 text-zinc-950">
    <SiteHeader />
    <MainContent />
  </div>
);

export default RootLayout;
