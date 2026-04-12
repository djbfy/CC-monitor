import ReactDOM from "react-dom/client";
import App from "./App";
import BarApp from "./BarApp";
import "./styles/tokens.css";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

// Detect which Tauri window is rendering.
// In Tauri v2, __TAURI_INTERNALS__.metadata.currentWebview.label is set before JS runs.
// For dev mode without Tauri (e.g. Vite dev server), we need a different approach.
function getWindowLabel(): string | undefined {
  const label = (window as any).__TAURI_INTERNALS__?.metadata?.currentWebview?.label;
  if (label) return label;
  const name = (window as any).name;
  if (name) return name;
  const tauriLabel = (window as any).__TAURI_WINDOW_LABEL__;
  if (tauriLabel) return tauriLabel;
  // Fallback for dev: read from URL param ?window=bar
  const params = new URLSearchParams(window.location.search);
  return params.get('window') ?? undefined;
}

const windowLabel = getWindowLabel();

if (windowLabel === "bar") {
  root.render(<BarApp />);
} else {
  root.render(<App />);
}
