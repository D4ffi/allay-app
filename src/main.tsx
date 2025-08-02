import React from "react";
import ReactDOM from "react-dom/client";
import { SystemProvider } from "./contexts/SystemContext.tsx";
import { LocaleProvider } from "./contexts/LocaleContext.tsx";
import { RconProvider } from "./contexts/RconContext.tsx";
import { ServerStateProvider } from "./contexts/ServerStateContext.tsx";
import { ThemeProvider } from "./contexts/ThemeContext.tsx";
import "./Global.css";
import {StyleGuide} from "./pages/StyleGuide.tsx";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <LocaleProvider>
        <SystemProvider>
          <ServerStateProvider>
            <RconProvider>
              <StyleGuide />
            </RconProvider>
          </ServerStateProvider>
        </SystemProvider>
      </LocaleProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
