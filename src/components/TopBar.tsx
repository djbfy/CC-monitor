import { useState, useEffect, useCallback } from 'react';
import { Session } from '../types';
import './TopBar.css';

const startDrag = () => {
  console.log('[TopBar] startDrag called');
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
    console.log('[TopBar] getCurrentWindow ready');
    getCurrentWindow().startDragging().catch((e: unknown) => {
      console.error('startDragging failed:', e);
    });
  }).catch((e: unknown) => {
    console.error('[TopBar] dynamic import failed:', e);
  });
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
  const [openDropdown, setOpenDropdown] = useState<Session['state'] | null>(null);
  console.log('[TopBar] render, sessions:', sessions.length, 'openDropdown:', openDropdown, 'window size:', window.innerWidth, 'x', window.innerHeight);

  // Close dropdown on click outside
  useEffect(() => {
    if (openDropdown === null) return;
    const handler = (e: MouseEvent) => {
      const dropdown = document.querySelector('.bar-dropdown');
      const barMode = document.querySelector('.bar-mode');
      if (dropdown && (dropdown.contains(e.target as Node) || barMode?.contains(e.target as Node))) {
        return;
      }
      setOpenDropdown(null);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [openDropdown]);

  const handleCellEnter = useCallback((state: Session['state'], list: Session[]) => {
    console.log('[TopBar] handleCellEnter', state, 'list.length:', list.length);
    if (list.length > 1) {
      setOpenDropdown(state);
    }
  }, []);

  const handleCellLeave = useCallback(() => {}, []);

  const handleCellClick = useCallback((_state: Session['state'], list: Session[]) => {
    if (list.length === 1) {
      onSessionClick(list[0].id);
    }
  }, [onSessionClick]);

  // Debug: listen on window to see if mouse events ever arrive
  useEffect(() => {
    const onMouseOver = (e: MouseEvent) => {
      console.log('[window] mouseover', (e.target as HTMLElement).className, 'bar-mode children:', document.querySelectorAll('.bar-mode > *').length);
    };
    const onMouseEnter = (e: MouseEvent) => {
      console.log('[window] mouseenter', (e.target as HTMLElement).className);
    };
    window.addEventListener('mouseover', onMouseOver);
    window.addEventListener('mouseenter', onMouseEnter);
    return () => {
      window.removeEventListener('mouseover', onMouseOver);
      window.removeEventListener('mouseenter', onMouseEnter);
    };
  }, []);

  const grouped = STATE_ORDER.reduce((acc, state) => {
    acc[state] = sessions.filter((s) => s.state === state);
    return acc;
  }, {} as Record<Session['state'], Session[]>);

  return (
    <>
      <div className="bar-mode">
        {/* Drag handle */}
        <div className="bar-drag-handle" onMouseDown={startDrag}>
          <svg viewBox="0 0 12 12" fill="currentColor">
            <circle cx="3" cy="3" r="1.2" />
            <circle cx="9" cy="3" r="1.2" />
            <circle cx="3" cy="6" r="1.2" />
            <circle cx="9" cy="6" r="1.2" />
            <circle cx="3" cy="9" r="1.2" />
            <circle cx="9" cy="9" r="1.2" />
          </svg>
        </div>

        {/* State columns */}
        {STATE_ORDER.map((state) => {
          const list = grouped[state];
          const count = list.length;
          const primary = list[0];

          return (
            <div key={state} className="bar-col">
              <div
                className={`bar-cell ${count === 0 ? 'bar-cell--empty' : ''}`}
                data-state={state}
                onMouseEnter={() => handleCellEnter(state, list)}
                onMouseLeave={handleCellLeave}
                onClick={() => handleCellClick(state, list)}
              >
                <span className={`bar-dot bar-dot--${count === 0 ? 'empty' : state}`} />
                {count === 0 ? (
                  <span className="bar-label-empty">无</span>
                ) : (
                  <>
                    <span className="bar-name">{primary?.name}</span>
                    <span className={`bar-state-label bar-state-label--${state}`}>
                      {STATE_LABELS[state]}
                    </span>
                    {count > 1 && (
                      <span className={`bar-count bar-count--${state}`}>{count}</span>
                    )}
                  </>
                )}
              </div>
            </div>
          );
        })}

        {/* Right-side buttons */}
        {onBackToCard && (
          <div className="bar-right">
            <div
              className="bar-back"
              onClick={onBackToCard}
              role="button"
              tabIndex={0}
              title="切换到卡片视图"
            >
              <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
                <rect x="0.5" y="0.5" width="4.5" height="4.5" rx="1" />
                <rect x="7" y="0.5" width="4.5" height="4.5" rx="1" />
                <rect x="0.5" y="7" width="4.5" height="4.5" rx="1" />
                <rect x="7" y="7" width="4.5" height="4.5" rx="1" />
              </svg>
            </div>
            {onTogglePin && (
              <div
                className={`bar-pin ${isPinned ? 'bar-pin--active' : ''}`}
                onClick={onTogglePin}
                role="button"
                tabIndex={0}
                title={isPinned ? '取消置顶' : '置顶'}
              >
                <svg viewBox="0 0 12 12" fill="currentColor">
                  <path d="M7 1a1 1 0 011 1v.586l1.293 1.293a.5.5 0 01-.293.853L8 4.414V9.5a.5.5 0 01-.854.354l-2-2A.5.5 0 015 7.5V4.414l-.707-.707a.5.5 0 010-.707l1-1A1 1 0 016 2h1v1.586l-1 1A.5.5 0 012 6.414V9.5a.5.5 0 01-.146.354l-2 2A.5.5 0 01-.208.896L.5 12.5l.146.051a.5.5 0 00.708-.354V10a.5.5 0 01.146-.354l2-2A.5.5 0 003.5 7.5V6.414l.707.707a.5.5 0 010 .707l-1 1V11a1 1 0 001 1v1H2.5l-.146-.051a1.5 1.5 0 010-1.898l2.792-2.792A.5.5 0 005.5 5.5V4a.5.5 0 01.146-.354l1-1V3H7z"/>
                </svg>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Dropdown */}
      {openDropdown !== null ? (
        (() => {
          const count = grouped[openDropdown].length;
          console.log('[TopBar] dropdown guard:', { openDropdown, count });
          return count > 1 ? (
            <Dropdown
              list={grouped[openDropdown]}
              state={openDropdown}
              onSelect={(id) => {
                onSessionClick(id);
                setOpenDropdown(null);
              }}
            />
          ) : null;
        })()
      ) : null}
    </>
  );
}

function Dropdown({
  list,
  state,
  onSelect,
}: {
  list: Session[];
  state: Session['state'];
  onSelect: (id: string) => void;
}) {
  const [pos, setPos] = useState({ top: 0, left: 0, width: 180 });

  useEffect(() => {
    // Find the target cell by data-state attribute
    const cell = document.querySelector(`[data-state="${state}"]`) as HTMLElement | null;
    if (!cell) { console.log('[Dropdown] no cell found for state:', state); return; }

    // Use offsetTop of the cell relative to bar-mode, plus bar-mode's offset from window top
    const barMode = document.querySelector('.bar-mode') as HTMLElement | null;
    const barTop = barMode ? barMode.offsetTop : 0;
    const barLeft = barMode ? barMode.offsetLeft : 0;

    // Cell position relative to bar-mode container
    const cellTop = cell.offsetTop;
    const cellLeft = cell.offsetLeft;
    const cellWidth = cell.offsetWidth;

    console.log('[Dropdown] cell offsetTop:', cellTop, 'offsetLeft:', cellLeft, 'offsetWidth:', cellWidth, 'barTop:', barTop, 'barLeft:', barLeft);

    setPos({
      top: barTop + cellTop + cell.offsetHeight + 4, // just below cell
      left: barLeft + cellLeft,
      width: Math.max(cellWidth, 180),
    });
  }, [state]);

  return (
    <div className="bar-dropdown" style={{ top: pos.top, left: pos.left, width: pos.width }}>
      {list.map((s) => (
        <div
          key={s.id}
          className="bar-dropdown-item"
          onClick={() => onSelect(s.id)}
        >
          <span className={`bar-dot bar-dot--${s.state}`} />
          <span className="bar-dropdown-name">{s.name}</span>
        </div>
      ))}
    </div>
  );
}
