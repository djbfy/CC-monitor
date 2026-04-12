import { Session } from '../types';
import './TopBar.css';

const startDrag = () => {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
    getCurrentWindow().startDragging().catch(console.error);
  }).catch(console.error);
};

const STATE_ORDER: Session['state'][] = ['running', 'confirm', 'idle', 'offline'];

const STATE_LABELS: Record<Session['state'], string> = {
  running: '运行中',
  confirm: '待确认',
  idle: '休息中',
  offline: '离线',
};

interface TopBarProps {
  sessions: Session[];
  onSessionClick: (id: string) => void;
  onBackToCard?: () => void;
  isPinned?: boolean;
  onTogglePin?: () => void;
}

export function TopBar({ sessions, onSessionClick, onBackToCard, isPinned, onTogglePin }: TopBarProps) {
  const ordered = [...sessions].sort((a, b) => {
    const ai = STATE_ORDER.indexOf(a.state);
    const bi = STATE_ORDER.indexOf(b.state);
    if (ai !== bi) return ai - bi;
    return a.id.localeCompare(b.id);
  });

  return (
    <div className="bar-mode">
      {/* Drag handle */}
      <div className="bar-drag-handle" onMouseDown={startDrag}>
        <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeLinecap="round" strokeWidth="1.4">
          <line x1="4" y1="6" x2="16" y2="6" />
          <line x1="4" y1="10" x2="16" y2="10" />
          <line x1="4" y1="14" x2="16" y2="14" />
        </svg>
      </div>

      {/* Scrollable session tiles */}
      <div className="bar-scroll-container">
        <div className="bar-tiles">
          {ordered.map((session) => (
            <div
              key={session.id}
              className={`bar-tile bar-tile--${session.state}`}
              onClick={() => onSessionClick(session.id)}
            >
              <span className={`bar-dot bar-dot--${session.state}`} />
              <span className="bar-name">{session.name}</span>
              <span className="bar-divider" />
              <span className={`bar-state-label bar-state-label--${session.state}`}>
                {STATE_LABELS[session.state]}
              </span>
            </div>
          ))}
        </div>
      </div>
      {/* Right fade — outside scroll container so it stays fixed */}
      <div className="bar-edge bar-edge--right" />

      {/* Right-side buttons */}
      {onBackToCard && (
        <div className="bar-right">
          <div
            className="bar-btn"
            onClick={onBackToCard}
            role="button"
            tabIndex={0}
            title="切换到卡片视图"
          >
            <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5">
              <rect x="2" y="2" width="6.5" height="6.5" rx="1.5" />
              <rect x="11.5" y="2" width="6.5" height="6.5" rx="1.5" />
              <rect x="2" y="11.5" width="6.5" height="6.5" rx="1.5" />
              <rect x="11.5" y="11.5" width="6.5" height="6.5" rx="1.5" />
            </svg>
          </div>
          {onTogglePin && (
            <div
              className={`bar-btn ${isPinned ? 'bar-btn--active' : ''}`}
              onClick={onTogglePin}
              role="button"
              tabIndex={0}
              title={isPinned ? '取消置顶' : '置顶'}
            >
              <svg viewBox="0 0 20 20" fill="currentColor">
                <path d="M12 2.5a1.5 1.5 0 011.5 1.5v1.793l.854.854a.5.5 0 00.353.146H15.5a1 1 0 011 1v1a1 1 0 01-1 1h-1.5l-.854.854a.5.5 0 01-.853-.354L12.5 9.707V11a1 1 0 01-1 1h-1a1 1 0 01-1-1v-1.293l-.854-.854a.5.5 0 01.353-.853H9a1 1 0 011-1h.793l.647-.647a.5.5 0 00.353-.853L11 4.5V3A1.5 1.5 0 0112.5 1.5h.5V2.5H12zM5 15l5-5 5 5v2H5v-2z"/>
              </svg>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
