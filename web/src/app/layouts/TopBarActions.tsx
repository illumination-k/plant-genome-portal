import type { ReactElement } from "react";
import ThemeToggle from "@/shared/ui/ThemeToggle";

const TopBarActions = (): ReactElement => (
  <div className="ml-auto flex items-center gap-2 md:ml-0">
    <ThemeToggle />
    <a className="hidden text-sm text-text-muted hover:text-text md:inline" href="/openapi.json">
      API
    </a>
  </div>
);

export default TopBarActions;
