// App.tsx — Main window: macOS style
import { useState, useCallback, useEffect } from 'react';
import { open, ask } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useSessions } from './hooks/useSessions';
import { useLaunchSession } from './hooks/useLaunchSession';
import { ViewToggle } from './components/ViewToggle';
import { CardGrid } from './components/CardGrid';
import { ErrorToast } from './components/ErrorToast';
import './App.css';

export default function App() {
  const { sessions, error, refresh } = useSessions();
  const launchSession = useLaunchSession();
  const [localError, setLocalError] = useState<string | null>(null);

  const allError = localError || error;

  // Intercept window close and show confirmation
  useEffect(() => {
    const win = getCurrentWindow();
    const handler = () => {
      ask('确定要退出 CC Monitor 吗？', {
        title: '退出确认',
        kind: 'warning',
      }).then((confirmed) => {
        if (confirmed) {
          invoke('exit_app').catch(() => {});
        }
      });
    };
    const unlistenPromise = win.onCloseRequested(handler);
    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, []);

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
