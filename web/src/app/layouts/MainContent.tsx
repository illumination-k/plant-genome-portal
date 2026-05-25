import { Outlet } from "react-router";
import type { ReactElement } from "react";

const MainContent = (): ReactElement => (
  <main className="min-w-0 flex-1">
    <div className="mx-auto max-w-[1440px] px-6 py-8 md:px-8">
      <Outlet />
    </div>
  </main>
);

export default MainContent;
