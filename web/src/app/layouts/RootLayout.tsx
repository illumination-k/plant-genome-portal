import type { ReactElement } from "react";
import SideRail from "@/app/layouts/SideRail";
import TopBar from "@/app/layouts/TopBar";
import MainContent from "@/app/layouts/MainContent";
import GlobalShortcuts from "@/app/command-palette/GlobalShortcuts";

const RootLayout = (): ReactElement => (
  <div className="flex min-h-screen flex-col bg-canvas text-text">
    <TopBar />
    <div className="flex flex-1">
      <SideRail />
      <MainContent />
    </div>
    <GlobalShortcuts />
  </div>
);

export default RootLayout;
