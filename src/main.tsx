import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

/* Design System CSS */
import "./styles/variables.css";
import "./styles/base.css";
import "./styles/components.css";
import "./styles/animations.css";
import "./styles/floating-window.css";
import "./styles/settings.css";
import "./styles/onboarding.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
