import type { ReactElement } from "react";
import SideRail from "@/layouts/SideRail";
import TopBar from "@/layouts/TopBar";
import MainContent from "@/layouts/MainContent";

const RootLayout = (): ReactElement => (
  <div className="flex min-h-screen flex-col bg-canvas text-text">
    <TopBar />
    <div className="flex flex-1">
      <SideRail />
      <MainContent />
    </div>
  </div>
);

export default RootLayout;
