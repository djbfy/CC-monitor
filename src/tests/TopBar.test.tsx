import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TopBar } from '../components/TopBar';
import { Session } from '../types';

const makeSession = (overrides: Partial<Session>): Session => ({
  id: '1',
  name: 'test-session',
  workDir: 'D:\\test',
  pid: 12345,
  state: 'running',
  silentSecs: 0,
  cpuPercent: 10,
  lastLine: 'Running',
  startedAt: Date.now(),
  ...overrides,
});

describe('TopBar', () => {
  it('only renders running and confirm sessions', () => {
    const sessions = [
      makeSession({ id: '1', state: 'running', name: 'run-1' }),
      makeSession({ id: '2', state: 'confirm', name: 'confirm-1' }),
      makeSession({ id: '3', state: 'idle', name: 'idle-1' }),
      makeSession({ id: '4', state: 'offline', name: 'offline-1' }),
    ];
    render(<TopBar sessions={sessions} onSessionClick={vi.fn()} />);
    expect(screen.getByText('run-1')).toBeInTheDocument();
    expect(screen.getByText('confirm-1')).toBeInTheDocument();
    expect(screen.queryByText('idle-1')).toBeNull();
    expect(screen.queryByText('offline-1')).toBeNull();
  });

  it('shows idle count badge when idle sessions exist', () => {
    const sessions = [
      makeSession({ id: '1', state: 'running' }),
      makeSession({ id: '2', state: 'idle' }),
      makeSession({ id: '3', state: 'idle' }),
    ];
    render(<TopBar sessions={sessions} onSessionClick={vi.fn()} />);
    expect(screen.getByText(/休息 2/)).toBeInTheDocument();
  });

  it('shows offline count badge when offline sessions exist', () => {
    const sessions = [
      makeSession({ id: '1', state: 'offline' }),
      makeSession({ id: '2', state: 'offline' }),
    ];
    render(<TopBar sessions={sessions} onSessionClick={vi.fn()} />);
    expect(screen.getByText(/离线 2/)).toBeInTheDocument();
  });

  it('shows neither badge when all sessions are running/confirm', () => {
    const sessions = [
      makeSession({ id: '1', state: 'running' }),
      makeSession({ id: '2', state: 'confirm' }),
    ];
    render(<TopBar sessions={sessions} onSessionClick={vi.fn()} />);
    expect(screen.queryByText(/休息/)).toBeNull();
    expect(screen.queryByText(/离线/)).toBeNull();
  });

  it('calls onSessionClick with correct id when session is clicked', () => {
    const clickSpy = vi.fn();
    const sessions = [makeSession({ id: 'clickable', state: 'running', name: 'target' })];
    render(<TopBar sessions={sessions} onSessionClick={clickSpy} />);
    screen.getByText('target').click();
    expect(clickSpy).toHaveBeenCalledWith('clickable');
  });
});
