import "./styles.css";
import AppProviders from "@/app/AppProviders";
import { client } from "@/api/client/client.gen";
import { createRoot } from "react-dom/client";

const apiBaseUrl = import.meta.env.VITE_API_BASE_URL;
if (apiBaseUrl) {
  client.setConfig({ baseUrl: apiBaseUrl });
}

const rootElement = document.querySelector("#root");
if (!rootElement) {
  throw new Error("Root element not found.");
}

createRoot(rootElement).render(<AppProviders />);
