import { Session } from '../types';
import { StatusDot } from './StatusDot';
import { SilentBar } from './SilentBar';
import { TerminalPreview } from './TerminalPreview';
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
  prevLine?: string;
  onRemove?: (id: string) => void;
}

function formatDuration(ms: number): string {
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  return `${hrs}h`;
}

export function SessionCard({ session, prevLine, onRemove }: SessionCardProps) {
  const metaActive = session.state === 'running' ? '活跃' : session.state === 'confirm' ? '等待' : session.state === 'idle' ? '静默' : null;
  const metaValue = session.state !== 'offline'
    ? metaActive === '静默' || metaActive === '等待'
      ? `${session.silentSecs}s`
      : formatDuration(Date.now() - session.startedAt)
    : null;

  return (
    <div className={`card ${session.state}`}>
      <div className="card-head">
        <span className="card-name">{session.name}</span>
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
      <SilentBar state={session.state} silentSecs={session.silentSecs} />
      <TerminalPreview
        lastLine={session.lastLine || '  (无输出)'}
        prevLine={prevLine}
        state={session.state}
      />
      <div className="meta">
        <span className="mi">CPU <span className="mv">{session.cpuPercent.toFixed(1)}%</span></span>
        {metaValue && (
          <span className="mi">
            {metaActive} <span className="mv">{metaValue}</span>
          </span>
        )}
        {session.state === 'offline' && (
          <span className="mi">进程 <span className="mv">不存在</span></span>
        )}
      </div>
    </div>
  );
}
