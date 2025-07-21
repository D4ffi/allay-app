import React from "react";
import ReactDOM from "react-dom/client";
import Home from "./pages/Home.tsx";
import { SystemProvider } from "./contexts/SystemContext.tsx";
import { LocaleProvider } from "./contexts/LocaleContext.tsx";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <LocaleProvider>
      <SystemProvider>
        <Home />
      </SystemProvider>
    </LocaleProvider>
  </React.StrictMode>,
);
