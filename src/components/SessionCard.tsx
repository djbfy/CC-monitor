import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Session } from '../types';
import { StatusDot } from './StatusDot';
import { SessionInfo } from './SessionInfo';
import './SessionCard.css';

const STATE_LABELS: Record<Session['state'], string> = {
  running: '运行中',
  confirm: '待确认',
  idle: '休息中',
  offline: '离线',
};

const STATE_CLASS: Record<Session['state'], string> = {
  running: 'st-run',
  confirm: 'st-confirm',
  idle: 'st-idle',
  offline: 'st-off',
};

interface SessionCardProps {
  session: Session;
  onRemove?: (id: string) => void;
}

function formatDuration(ms: number): string {
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h`;
  const days = Math.floor(hrs / 24);
  return `${days}d`;
}

export function SessionCard({ session, onRemove }: SessionCardProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');
  const [now, setNow] = useState(Date.now());
  const inputRef = useRef<HTMLInputElement>(null);

  // Live counter — tick every second for running sessions
  useEffect(() => {
    if (session.state === 'offline') return;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [session.state]);

  const activeTime = session.state !== 'offline' ? formatDuration(now - session.startedAt) : null;

  const startEditing = () => {
    setDraft(session.name);
    setEditing(true);
    setTimeout(() => inputRef.current?.select(), 0);
  };

  const commitRename = async () => {
    const trimmed = draft.trim();
    if (!trimmed || trimmed === session.name) {
      setEditing(false);
      return;
    }
    try {
      await invoke('rename_session', { id: session.id, name: trimmed });
      setEditing(false);
    } catch (e) {
      console.error('[SessionCard] rename failed:', e);
    }
  };

  const cancelRename = () => {
    setDraft(session.name);
    setEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') commitRename();
    else if (e.key === 'Escape') cancelRename();
  };

  return (
    <div className={`card ${session.state}`}>
      <div className="card-head">
        {editing ? (
          <input
            ref={inputRef}
            className="card-name-input"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commitRename}
            onKeyDown={handleKeyDown}
            autoFocus
          />
        ) : (
          <span
            className="card-name"
            onDoubleClick={startEditing}
            title="双击修改名称"
          >
            {session.name}
          </span>
        )}
        <span className="card-pid">
          {session.pid !== null ? `PID ${session.pid}` : '—'}
        </span>
        {onRemove && (
          <button
            className="card-remove"
            onClick={() => onRemove(session.id)}
            title="移除监控"
          >
            ×
          </button>
        )}
      </div>
      <div className="status-row">
        <StatusDot state={session.state} />
        <span className={`st ${STATE_CLASS[session.state]}`}>
          {STATE_LABELS[session.state]}
        </span>
      </div>
      <SessionInfo session={session} />
      <div className="meta">
        <span className="mi">CPU <span className="mv">{session.cpuPercent.toFixed(1)}%</span></span>
        {activeTime ? (
          <span className="mi">活跃 <span className="mv">{activeTime}</span></span>
        ) : (
          <span className="mi">进程 <span className="mv">不存在</span></span>
        )}
      </div>
    </div>
  );
}
