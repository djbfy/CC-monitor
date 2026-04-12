import { SessionState } from '../types';
import './TerminalPreview.css';

interface TerminalPreviewProps {
  lastLine: string;
  prevLine?: string;
  state: SessionState;
}

export function TerminalPreview({ lastLine, prevLine, state }: TerminalPreviewProps) {
  const isConfirm = state === 'confirm';

  return (
    <div className="term">
      {prevLine && <div className="tl">{prevLine}</div>}
      <div className={`tl ${isConfirm ? 'w' : ''} ${isConfirm ? 'c' : ''}`}>
        {lastLine}
        {isConfirm && <span className="cursor">█</span>}
      </div>
    </div>
  );
}
