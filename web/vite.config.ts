import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  build: {
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
