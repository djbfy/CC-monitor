export type SessionState = 'running' | 'confirm' | 'idle' | 'offline'

export interface Session {
  id: string
  name: string
  workDir: string
  pid: number | null
  state: SessionState
  silentSecs: number
  cpuPercent: number
  lastLine: string
  startedAt: number
}

export type CcResult<T> = { ok: true; data: T } | { ok: false; error: string }
