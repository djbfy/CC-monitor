// App.tsx — Main window: macOS style
import { useState, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useSessions } from './hooks/useSessions';
import { useLaunchSession } from './hooks/useLaunchSession';
import { ViewToggle } from './components/ViewToggle';
import { CardGrid } from './components/CardGrid';
import { ErrorToast } from './components/ErrorToast';
import './App.css';

const STATE_LABELS = ['运行中', '待确认', '休息中', '离线'] as const;
const STATE_KEYS = ['running', 'confirm', 'idle', 'offline'] as const;

export default function App() {
  const { sessions, error, refresh } = useSessions();
  const launchSession = useLaunchSession();
  const [localError, setLocalError] = useState<string | null>(null);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);

  const allError = localError || error;

  const handleAddClick = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择 Claude Code 工作目录',
      });
      if (!selected) return;

      const result = await launchSession(selected as string);
      if (!result.ok) {
        setLocalError(result.error);
        return;
      }
      refresh();
    } catch (e: unknown) {
      setLocalError(String(e));
    }
  }, [launchSession, refresh]);

  const handleRemoveSession = useCallback(async (id: string) => {
    try {
      await invoke('stop_session', { id });
    } catch (e: unknown) {
      setLocalError(String(e));
    }
  }, []);

  const handleRefresh = useCallback(async () => {
    try {
      await invoke('refresh_sessions');
    } catch (e: unknown) {
      setLocalError(String(e));
    }
  }, []);

  const dismissError = useCallback(() => {
    setLocalError(null);
  }, []);

  const handleSwitchToBar = useCallback(() => {
    invoke('set_view_mode', { mode: 'bar' }).catch(console.error);
  }, []);

  // Group sessions by state
  const grouped = STATE_KEYS.reduce((acc, key) => {
    acc[key] = sessions.filter((s) => s.state === key);
    return acc;
  }, {} as Record<string, typeof sessions>);

  const activeGroup = selectedGroup;

  return (
    <div className="root">
      {/* macOS titlebar */}
      <div className="toolbar">
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div className="window-dots">
            <div className="dot-red" />
            <div className="dot-yellow" />
            <div className="dot-green" />
          </div>
          <span className="app-title">CC Monitor</span>
        </div>
        <div className="toolbar-right">
          <button className="refresh-btn" onClick={handleRefresh} title="刷新列表">
            ↻
          </button>
          <ViewToggle mode="card" onChange={handleSwitchToBar} />
        </div>
      </div>

      {/* Main content area */}
      <div className="content-area">
        <CardGrid sessions={sessions} onAddClick={handleAddClick} onRemoveSession={handleRemoveSession} />
      </div>

      <ErrorToast error={allError} onDismiss={dismissError} />
    </div>
  );
}
