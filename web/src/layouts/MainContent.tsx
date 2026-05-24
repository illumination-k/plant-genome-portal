import { Outlet } from "react-router";
import type { ReactElement } from "react";

const MainContent = (): ReactElement => (
  <main className="mx-auto max-w-7xl px-5 py-8">
    <Outlet />
  </main>
);

export default MainContent;
