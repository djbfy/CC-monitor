import ReactDOM from "react-dom/client";
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from "./App";
import BarApp from "./BarApp";
import "./styles/tokens.css";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

window.onerror = (msg, src, line, col, err) => {
  console.error('[JS ERROR]', msg, 'at', src, 'line', line, 'col', col, err);
  const id = 'js-error-overlay';
  let el = document.getElementById(id);
  if (!el) {
    el = document.createElement('div');
    el.id = id;
    el.style = 'position:fixed;inset:0;z-index:9999;background:#1e1e24;color:#f87171;padding:20px;font-family:monospace;font-size:12px;overflow:auto';
    document.body.appendChild(el);
  }
  el.innerHTML = `<pre><b>JS Error:</b> ${msg}\n<b>File:</b> ${src}:${line}:${col}\n<b>Stack:</b> ${err?.stack || err?.message || ''}</pre>`;
};

async function init() {
  try {
    const win = getCurrentWindow();
    const label = win.label;
    if (label === "bar") {
      root.render(<BarApp />);
    } else {
      root.render(<App />);
    }
  } catch (e) {
    console.error('[init] getCurrentWindow() failed, falling back:', e);
    const params = new URLSearchParams(window.location.search);
    root.render(params.get('window') === 'bar' ? <BarApp /> : <App />);
  }
}

init();
