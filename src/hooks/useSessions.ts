// useSessions.ts — Phase 4: real Tauri event subscription
import { useState, useEffect, useCallback } from 'react';
import { Session } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export function useSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Initial load
    invoke<Session[]>('get_sessions')
      .then(setSessions)
      .catch((e: unknown) => setError(String(e)));

    // Real-time session updates
    let unlistenUpdate: UnlistenFn | undefined;
    let unlistenCleanup: UnlistenFn | undefined;

    listen<Session[]>('session_update', (event) => {
      setSessions((prev) => {
        const updated = event.payload;
        if (updated.length === 1) {
          // Single session update — merge into existing list
          const updatedSession = updated[0];
          const exists = prev.some((s) => s.id === updatedSession.id);
          if (exists) {
            return prev.map((s) => (s.id === updatedSession.id ? updatedSession : s));
          } else {
            return [...prev, updatedSession];
          }
        }
        // Full list update (from get_sessions)
        return updated;
      });
    }).then((fn) => {
      unlistenUpdate = fn;
    });

    // Session auto-cleanup (offline > 10min)
    listen<string>('session_cleanup', (event) => {
      setSessions((prev) => prev.filter((s) => s.id !== event.payload));
    }).then((fn) => {
      unlistenCleanup = fn;
    });

    return () => {
      unlistenUpdate?.();
      unlistenCleanup?.();
    };
  }, []);

  const refresh = useCallback(async () => {
    try {
      const result = await invoke<Session[]>('get_sessions');
      setSessions(result);
    } catch (e: unknown) {
      setError(String(e));
    }
  }, []);

  return { sessions, error, refresh };
}
