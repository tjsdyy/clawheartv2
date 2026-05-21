import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import { queryClient } from "./lib/queryClient";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { Toaster } from "./components/Toaster";
import "./lib/i18n";
import "./styles/globals.css";

// 全局兜底：把 React 之外的 unhandled 都打到 console，让 devtools 一眼可见
window.addEventListener("error", (e) => {
  console.error("[ClawHeart] window.error:", e.error || e.message);
});
window.addEventListener("unhandledrejection", (e) => {
  console.error("[ClawHeart] unhandled promise rejection:", e.reason);
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <App />
        </BrowserRouter>
        <Toaster />
      </QueryClientProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
