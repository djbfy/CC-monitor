// IslandApp.tsx — Dynamic Island UI
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow, LogicalPosition, primaryMonitor } from '@tauri-apps/api/window';
import { Session } from './types';

// Icon SVGs
const SettingsIcon = () => (
  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="3"/>
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
  </svg>
);

const ExitIcon = () => (
  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
    <polyline points="16 17 21 12 16 7"/>
    <line x1="21" y1="12" x2="9" y2="12"/>
  </svg>
);

type Tab = 'sessions' | 'config';

export default function IslandApp() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>('sessions');
  const [exitConfirm, setExitConfirm] = useState(false);
  const islandRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Initial load
    invoke<Session[]>('get_sessions')
      .then(setSessions)
      .catch(console.error);

    let unlistenUpdate: UnlistenFn | undefined;
    let unlistenCleanup: UnlistenFn | undefined;

    listen<Session[]>('session_update', (event) => {
      setSessions(event.payload);
    }).then((fn) => { unlistenUpdate = fn; });

    listen<string>('session_cleanup', (event) => {
      setSessions((prev) => prev.filter((s) => s.id !== event.payload));
    }).then((fn) => { unlistenCleanup = fn; });

    return () => {
      unlistenUpdate?.();
      unlistenCleanup?.();
    };
  }, []);

  // Compute state
  const confirmSessions = sessions.filter(s => s.state === 'confirm');
  const runningSessions = sessions.filter(s => s.state === 'running');
  const idleSessions = sessions.filter(s => s.state === 'idle');
  const hasConfirm = confirmSessions.length > 0;

  // Position: top-center of screen
  const handleSetPosition = async () => {
    try {
      const win = getCurrentWindow();
      const monitor = await primaryMonitor();
      if (!monitor) return;
      const mSize = monitor.size;
      const x = Math.floor((mSize.width - 300) / 2);
      await win.setPosition(new LogicalPosition(x, 40));
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    handleSetPosition();
  }, []);

  const handleAllow = async (sessionId: string) => {
    try {
      await invoke('respond_confirm', { id: sessionId, action: 'approve' });
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeny = async (sessionId: string) => {
    try {
      await invoke('respond_confirm', { id: sessionId, action: 'deny' });
    } catch (e) {
      console.error(e);
    }
  };

  const handleExit = () => {
    setExitConfirm(true);
  };

  const confirmExit = async () => {
    try {
      await invoke('exit_app');
    } catch (e) {
      console.error(e);
    }
  };

  const handleSwitchToCard = () => {
    invoke('set_view_mode', { mode: 'card' }).catch(console.error);
  };

  // Collapsed pill
  if (!expanded && !exitConfirm) {
    return (
      <div
        ref={islandRef}
        className="island-pill"
        onMouseEnter={() => setExpanded(true)}
        style={{
          position: 'fixed',
          top: 40,
          left: '50%',
          transform: 'translateX(-50%)',
          background: '#111',
          borderRadius: 999,
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          padding: '0 16px',
          height: 50,
          cursor: 'pointer',
          border: '0.5px solid rgba(255,255,255,0.08)',
          userSelect: 'none',
          transition: 'all 0.2s ease',
        }}
      >
        <div
          className="i-dot"
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: hasConfirm ? '#f59e0b' : runningSessions.length > 0 ? '#22c55e' : '#444',
            boxShadow: runningSessions.length > 0 ? '0 0 5px #22c55e88' : undefined,
            animation: hasConfirm ? 'pulse 1s ease-in-out infinite' : undefined,
          }}
        />
        <span style={{ fontSize: 13, fontWeight: 700, color: '#fff' }}>
          {sessions.length}
        </span>
        <span style={{ fontSize: 11, color: '#666' }}>sessions</span>
        {hasConfirm && (
          <span style={{
            fontSize: 10,
            fontWeight: 700,
            padding: '2px 8px',
            borderRadius: 99,
            background: 'rgba(245,158,11,.18)',
            color: '#fbbf24',
          }}>
            {confirmSessions.length} Confirm
          </span>
        )}
        {!hasConfirm && runningSessions.length > 0 && (
          <span style={{
            fontSize: 10,
            fontWeight: 700,
            padding: '2px 8px',
            borderRadius: 99,
            background: 'rgba(34,197,94,.15)',
            color: '#4ade80',
          }}>
            Running
          </span>
        )}
        {!hasConfirm && runningSessions.length === 0 && sessions.length === 0 && (
          <span style={{
            fontSize: 10,
            fontWeight: 700,
            padding: '2px 8px',
            borderRadius: 99,
            background: 'rgba(255,255,255,.08)',
            color: '#888',
          }}>
            Idle
          </span>
        )}

        <style>{`
          @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.4; }
          }
        `}</style>
      </div>
    );
  }

  // Exit confirm
  if (exitConfirm) {
    return (
      <div
        style={{
          position: 'fixed',
          top: 40,
          left: '50%',
          transform: 'translateX(-50%)',
          background: '#111',
          borderRadius: 22,
          width: 300,
          border: '0.5px solid rgba(255,255,255,0.08)',
          overflow: 'hidden',
        }}
      >
        <div style={{
          display: 'flex',
          alignItems: 'center',
          padding: '12px 14px 10px',
          borderBottom: '0.5px solid rgba(255,255,255,.07)',
        }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: '#f87171', flex: 1 }}>Confirm Exit?</div>
        </div>
        <div style={{ padding: 14 }}>
          <div style={{ fontSize: 12, color: '#aaa', lineHeight: 1.6, marginBottom: 14 }}>
            Exiting will close the HTTP service and all session states.
            <br />
            Currently there are <span style={{ color: '#fbbf24', fontWeight: 700 }}>{sessions.length}</span> active sessions.
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              onClick={() => setExitConfirm(false)}
              style={{
                flex: 1,
                padding: 7,
                borderRadius: 8,
                fontSize: 12,
                fontWeight: 700,
                background: 'rgba(239,68,68,.12)',
                color: '#f87171',
                border: '0.5px solid rgba(239,68,68,.3)',
                cursor: 'pointer',
              }}
            >
              Cancel
            </button>
            <button
              onClick={confirmExit}
              style={{
                flex: 1,
                padding: 7,
                borderRadius: 8,
                fontSize: 12,
                fontWeight: 700,
                background: '#ef4444',
                color: '#fff',
                border: 'none',
                cursor: 'pointer',
              }}
            >
              Confirm Exit
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Expanded island
  return (
    <div
      style={{
        position: 'fixed',
        top: 40,
        left: '50%',
        transform: 'translateX(-50%)',
        background: '#111',
        borderRadius: 22,
        width: 340,
        border: '0.5px solid rgba(255,255,255,0.08)',
        overflow: 'hidden',
        transition: 'all 0.2s ease',
      }}
      onMouseLeave={() => setExpanded(false)}
    >
      {/* Top bar */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '12px 14px 10px',
        borderBottom: '0.5px solid rgba(255,255,255,.07)',
      }}>
        <div
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: hasConfirm ? '#f59e0b' : '#22c55e',
            animation: hasConfirm ? 'pulse 1s ease-in-out infinite' : undefined,
            flexShrink: 0,
          }}
        />
        <div style={{ fontSize: 13, fontWeight: 700, color: '#e5e5e5', flex: 1 }}>CC Monitor</div>

        {/* Settings button */}
        <button
          onClick={() => setActiveTab(activeTab === 'config' ? 'sessions' : 'config')}
          style={{
            width: 26,
            height: 26,
            borderRadius: 8,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            border: 'none',
            background: activeTab === 'config' ? 'rgba(255,255,255,.08)' : 'transparent',
          }}
          title="Settings"
        >
          <SettingsIcon />
        </button>

        {/* Exit button */}
        <button
          onClick={handleExit}
          style={{
            width: 26,
            height: 26,
            borderRadius: 8,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            border: 'none',
            background: 'transparent',
          }}
          title="Exit"
        >
          <ExitIcon />
        </button>
      </div>

      {/* Body */}
      <div style={{ padding: 10 }}>
        {/* Tab bar */}
        <div style={{
          display: 'flex',
          background: 'rgba(255,255,255,.05)',
          borderRadius: 10,
          padding: 3,
          marginBottom: 10,
        }}>
          {(['sessions', 'config'] as Tab[]).map((tab) => (
            <div
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                flex: 1,
                textAlign: 'center',
                fontSize: 11,
                fontWeight: 600,
                color: activeTab === tab ? '#e5e5e5' : '#666',
                padding: '5px 0',
                borderRadius: 8,
                cursor: 'pointer',
                background: activeTab === tab ? 'rgba(255,255,255,.1)' : 'transparent',
                transition: 'all 0.15s',
                textTransform: tab === 'sessions' ? undefined : undefined,
              }}
            >
              {tab === 'sessions' ? 'Sessions' : 'Config'}
            </div>
          ))}
        </div>

        {activeTab === 'sessions' && (
          <div>
            {/* Confirm session — priority */}
            {confirmSessions.map((session) => (
              <div
                key={session.id}
                style={{
                  background: 'rgba(245,158,11,.07)',
                  border: '0.5px solid rgba(245,158,11,.35)',
                  borderRadius: 14,
                  padding: 11,
                  marginBottom: 8,
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6 }}>
                  <div style={{ width: 7, height: 7, borderRadius: '50%', background: '#f59e0b', animation: 'pulse 1s ease-in-out infinite', flexShrink: 0 }} />
                  <div style={{ fontSize: 9, fontWeight: 800, color: '#fbbf24', background: 'rgba(245,158,11,.18)', padding: '2px 6px', borderRadius: 99 }}>Confirm</div>
                  <div style={{ marginLeft: 'auto', fontSize: 9, color: '#f59e0b' }}>Needs confirm</div>
                </div>
                <div style={{ fontSize: 12, fontWeight: 700, color: '#e5e5e5', marginBottom: 3 }}>{session.name}</div>
                <div style={{ fontSize: 11, color: '#999', fontFamily: 'monospace', lineHeight: 1.5, marginBottom: 10 }}>{session.lastLine || 'Waiting...'}</div>
                <div style={{ display: 'flex', gap: 7 }}>
                  <button
                    onClick={() => handleAllow(session.id)}
                    style={{
                      flex: 1,
                      padding: 7,
                      borderRadius: 8,
                      fontSize: 12,
                      fontWeight: 700,
                      background: '#22c55e',
                      color: '#000',
                      border: 'none',
                      cursor: 'pointer',
                    }}
                  >
                    Allow
                  </button>
                  <button
                    onClick={() => handleDeny(session.id)}
                    style={{
                      flex: 1,
                      padding: 7,
                      borderRadius: 8,
                      fontSize: 12,
                      fontWeight: 700,
                      background: 'rgba(239,68,68,.12)',
                      color: '#f87171',
                      border: '0.5px solid rgba(239,68,68,.3)',
                      cursor: 'pointer',
                    }}
                  >
                    Deny
                  </button>
                </div>
              </div>
            ))}

            {/* Running sessions */}
            {runningSessions.map((session) => (
              <div
                key={session.id}
                onClick={handleSwitchToCard}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 9,
                  padding: 9,
                  borderRadius: 11,
                  marginBottom: 6,
                  cursor: 'pointer',
                }}
              >
                <div style={{ width: 7, height: 7, borderRadius: '50%', background: '#22c55e', animation: 'pg 1.8s ease-in-out infinite', flexShrink: 0 }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 12, fontWeight: 600, color: '#ddd', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{session.name}</div>
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: 3, flexShrink: 0 }}>
                  <span style={{ fontSize: 9, fontWeight: 700, padding: '2px 7px', borderRadius: 99, background: 'rgba(34,197,94,.13)', color: '#4ade80' }}>Running</span>
                </div>
              </div>
            ))}

            {/* Idle sessions */}
            {idleSessions.map((session) => (
              <div
                key={session.id}
                onClick={handleSwitchToCard}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 9,
                  padding: 9,
                  borderRadius: 11,
                  marginBottom: 6,
                  cursor: 'pointer',
                  opacity: 0.55,
                }}
              >
                <div style={{ width: 7, height: 7, borderRadius: '50%', background: '#444', flexShrink: 0 }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 12, fontWeight: 600, color: '#aaa', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{session.name}</div>
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: 3, flexShrink: 0 }}>
                  <span style={{ fontSize: 9, fontWeight: 700, padding: '2px 7px', borderRadius: 99, background: 'rgba(255,255,255,.07)', color: '#666' }}>Idle</span>
                </div>
              </div>
            ))}

            {sessions.length === 0 && (
              <div style={{ textAlign: 'center', padding: 20, color: '#555', fontSize: 12 }}>
                No sessions
              </div>
            )}
          </div>
        )}

        {activeTab === 'config' && (
          <div style={{ padding: 0 }}>
            <div style={{ marginBottom: 14 }}>
              <div style={{ fontSize: 10, fontWeight: 700, color: '#555', letterSpacing: '.08em', textTransform: 'uppercase', marginBottom: 8 }}>Behavior</div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: 8, borderBottom: '0.5px solid rgba(255,255,255,.04)' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 12, color: '#bbb' }}>Confirm timeout auto-allow</div>
                </div>
                <div style={{ width: 34, height: 19, background: 'rgba(255,255,255,.12)', borderRadius: 99, position: 'relative', cursor: 'pointer', flexShrink: 0, transition: 'background .2s' }} />
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: 8, borderBottom: '0.5px solid rgba(255,255,255,.04)' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 12, color: '#bbb' }}>Idle timeout remove</div>
                </div>
                <div style={{ width: 34, height: 19, background: '#22c55e', borderRadius: 99, position: 'relative', cursor: 'pointer', flexShrink: 0 }} />
              </div>
            </div>

            <div style={{ marginBottom: 14 }}>
              <div style={{ fontSize: 10, fontWeight: 700, color: '#555', letterSpacing: '.08em', textTransform: 'uppercase', marginBottom: 8 }}>View</div>
              <div
                onClick={handleSwitchToCard}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  padding: 8,
                  borderRadius: 10,
                  background: 'rgba(255,255,255,.05)',
                  cursor: 'pointer',
                }}
              >
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 12, color: '#bbb' }}>Switch to card view</div>
                </div>
                <span style={{ fontSize: 10, color: '#555' }}>→</span>
              </div>
            </div>

            <div>
              <div style={{ fontSize: 10, fontWeight: 700, color: '#555', letterSpacing: '.08em', textTransform: 'uppercase', marginBottom: 8 }}>Exit</div>
              <div
                onClick={handleExit}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  padding: 8,
                  borderRadius: 10,
                  background: 'rgba(239,68,68,.06)',
                  border: '0.5px solid rgba(239,68,68,.2)',
                  cursor: 'pointer',
                }}
              >
                <ExitIcon />
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 12, color: '#f87171', fontWeight: 600 }}>Exit app</div>
                  <div style={{ fontSize: 10, color: '#9a4040', marginTop: 1 }}>Stop HTTP service, clear all sessions</div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
        @keyframes pg {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.35; }
        }
        button:hover {
          filter: brightness(1.1);
        }
        button:active {
          filter: brightness(0.9);
        }
      `}</style>
    </div>
  );
}
