import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
  input: "../target/openapi/api.json",
  output: "src/api/client",
  plugins: [
    {
      name: "@hey-api/client-fetch",
      throwOnError: true,
    },
    "@hey-api/typescript",
    "valibot",
    "@hey-api/sdk",
    "@tanstack/react-query",
  ],
});
