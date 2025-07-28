import React from "react";
import ReactDOM from "react-dom/client";
import Home from "./pages/Home.tsx";
import { SystemProvider } from "./contexts/SystemContext.tsx";
import { LocaleProvider } from "./contexts/LocaleContext.tsx";
import { RconProvider } from "./contexts/RconContext.tsx";
import { ServerStateProvider } from "./contexts/ServerStateContext.tsx";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <LocaleProvider>
      <SystemProvider>
        <ServerStateProvider>
          <RconProvider>
            <Home />
          </RconProvider>
        </ServerStateProvider>
      </SystemProvider>
    </LocaleProvider>
  </React.StrictMode>,
);
