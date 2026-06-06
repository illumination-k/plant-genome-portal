// eslint-disable-next-line import/no-unassigned-import -- jest-dom registers its matchers via side effect
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

/*
 * Testing Library does not auto-clean between tests under Vitest unless global
 * cleanup hooks are wired up, so do it explicitly to keep DOM state from one
 * component test leaking into the next.
 */
afterEach(() => {
  cleanup();
});
