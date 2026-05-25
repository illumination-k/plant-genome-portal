import { StrictMode } from "react";
import type { ReactElement } from "react";
import QueryProvider from "@/app/QueryProvider";

const AppProviders = (): ReactElement => (
  <StrictMode>
    <QueryProvider />
  </StrictMode>
);

export default AppProviders;
