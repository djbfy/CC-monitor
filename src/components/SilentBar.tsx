import { SessionState } from '../types';
import './SilentBar.css';

interface SilentBarProps {
  state: SessionState;
  silentSecs: number;
  maxSecs?: number;
}

export function SilentBar({ state, silentSecs, maxSecs = 60 }: SilentBarProps) {
  const percent = Math.min((silentSecs / maxSecs) * 100, 100);
  return (
    <div className="silent-row">
      <div className="bar-track">
        <div
          className={`bar-fill bar-fill-${state}`}
          style={{ width: `${percent}%` }}
        />
      </div>
      <span className="bar-lbl">{silentSecs}s</span>
    </div>
  );
}
