import { SessionState } from '../types';
import './StatusDot.css';

interface StatusDotProps {
  state: SessionState;
}

export function StatusDot({ state }: StatusDotProps) {
  return <div className={`dot dot-${state}`} />;
}
