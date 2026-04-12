import { Session } from '../types';
import { SessionCard } from './SessionCard';
import './CardGrid.css';

interface CardGridProps {
  sessions: Session[];
  onAddClick: () => void;
  onRemoveSession?: (id: string) => void;
}

export function CardGrid({ sessions, onAddClick, onRemoveSession }: CardGridProps) {
  return (
    <div className="card-grid">
      {sessions.map((s) => (
        <SessionCard key={s.id} session={s} onRemove={onRemoveSession} />
      ))}
      <div className="card add-card" onClick={onAddClick} role="button" tabIndex={0}>
        <div className="card-head">
          <span className="card-name" style={{ color: 'var(--color-text-tertiary)' }}>
            + 添加终端
          </span>
        </div>
        <div className="add-inner">绑定新的 Claude Code 进程</div>
      </div>
    </div>
  );
}
