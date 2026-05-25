import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

type RolldownWarning = {
  code?: string;
  id?: string;
  message?: string;
};

type RolldownWarningHandler = (
  warning: RolldownWarning,
  defaultHandler: (warning: RolldownWarning) => void,
) => void;

const ignoredPureAnnotationPackages = ["/mobx/", "/mobx-react/"];
const asyncGenomeBrowserChunkWarningLimitKb = 2000;

const isIgnoredPureAnnotationWarning = (warning: RolldownWarning): boolean =>
  warning.code === "INVALID_ANNOTATION"
  && ignoredPureAnnotationPackages.some((pkg) => warning.id?.includes(pkg));

const onwarn: RolldownWarningHandler = (warning, defaultHandler) => {
  if (isIgnoredPureAnnotationWarning(warning)) {
    return;
  }
  defaultHandler(warning);
};

export default defineConfig({
  build: {
    chunkSizeWarningLimit: asyncGenomeBrowserChunkWarningLimitKb,
    rolldownOptions: {
      onwarn,
    },
    target: "es2022",
  },
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": "/src",
    },
  },
  server: {
    proxy: {
      "/health": "http://127.0.0.1:3000",
      "/jbrowse": "http://127.0.0.1:3000",
      "/openapi.json": "http://127.0.0.1:3000",
      "/sequence": "http://127.0.0.1:3000",
      "/v2": "http://127.0.0.1:3000",
    },
  },
});
