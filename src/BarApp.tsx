// BarApp.tsx — Lightweight app for the bar window
import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { TopBar } from './components/TopBar';
import { Session } from './types';

export default function BarApp() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [isPinned, setIsPinned] = useState(false);

  useEffect(() => {
    invoke<Session[]>('get_sessions').then(setSessions).catch(console.error);
    getCurrentWindow().isAlwaysOnTop().then(setIsPinned).catch(() => {});

    let unlistenUpdate: Promise<() => void>;
    let unlistenCleanup: Promise<() => void>;

    unlistenUpdate = listen<Session[]>('session_update', (event) => {
      setSessions(event.payload);
    });

    unlistenCleanup = listen<string>('session_cleanup', (event) => {
      setSessions((prev) => prev.filter((s) => s.id !== event.payload));
    });

    return () => {
      unlistenUpdate.then((fn) => fn());
      unlistenCleanup.then((fn) => fn());
    };
  }, []);

  const handleSessionClick = useCallback(async (_id: string) => {
    try {
      await invoke('set_view_mode', { mode: 'card' });
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const mainWindow = await WebviewWindow.getByLabel('main');
      if (mainWindow) await mainWindow.setFocus();
    } catch (e) {
      console.error('Failed to focus main window:', e);
    }
  }, []);

  const handleBackToCard = useCallback(async () => {
    try {
      await invoke('set_view_mode', { mode: 'card' });
    } catch (e) {
      console.error('Failed to switch to card view:', e);
    }
  }, []);

  const handleTogglePin = useCallback(async () => {
    try {
      const newState = !isPinned;
      await getCurrentWindow().setAlwaysOnTop(newState);
      setIsPinned(newState);
    } catch (e) {
      console.error('Failed to toggle always-on-top:', e);
    }
  }, [isPinned]);


  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        background: '#0d0d12',
        overflow: 'visible',
      }}
    >
      <TopBar
        sessions={sessions}
        onSessionClick={handleSessionClick}
        onBackToCard={handleBackToCard}
        isPinned={isPinned}
        onTogglePin={handleTogglePin}
      />
    </div>
  );
}
