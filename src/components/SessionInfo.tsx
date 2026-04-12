import { useMemo } from 'react';
import { Session } from '../types';
import './SessionInfo.css';

interface SessionInfoProps {
  session: Session;
}

function formatTimeAgo(ts: number): string {
  const diff = Date.now() - ts;
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return `${secs}s前启动`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m前启动`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h前启动`;
  const days = Math.floor(hrs / 24);
  return `${days}d前启动`;
}

export function SessionInfo({ session }: SessionInfoProps) {
  const timeLabel = useMemo(() => formatTimeAgo(session.startedAt), [session.startedAt]);

  return (
    <div className="session-info">
      <div className="si-row">
        <svg className="si-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.4">
          <path d="M3 4.5A1.5 1.5 0 014.5 3H9l2 2H4.5A1.5 1.5 0 003 6.5v9A1.5 1.5 0 004.5 17H15.5A1.5 1.5 0 0017 15.5v-6" />
          <path d="M12 3v3.5" />
          <path d="M9 6.5l3-3 3 3" />
        </svg>
        <span className="si-path" title={session.workDir}>{session.workDir}</span>
      </div>
      <div className="si-row">
        <svg className="si-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.4">
          <circle cx="10" cy="10" r="8" />
          <path d="M10 6v4l3 3" />
        </svg>
        <span className="si-time">{timeLabel}</span>
      </div>
    </div>
  );
}
