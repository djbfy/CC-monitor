import { invoke } from '@tauri-apps/api/core';
import './ViewToggle.css';

type ViewMode = 'card' | 'bar';

interface ViewToggleProps {
  mode: ViewMode;
  onChange: (mode: ViewMode) => void;
}

export function ViewToggle({ mode, onChange }: ViewToggleProps) {
  const handleChange = async (newMode: ViewMode) => {
    onChange(newMode);
    try {
      await invoke('set_view_mode', { mode: newMode });
    } catch (e) {
      console.error('set_view_mode failed:', e);
    }
  };

  return (
    <div className="toggle-wrap">
      <span className="toggle-label">视图</span>
      <div className="toggle">
        <button
          className={`toggle-btn ${mode === 'card' ? 'active' : ''}`}
          onClick={() => handleChange('card')}
        >
          <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
            <rect x="0.5" y="0.5" width="4.5" height="4.5" rx="1" />
            <rect x="7" y="0.5" width="4.5" height="4.5" rx="1" />
            <rect x="0.5" y="7" width="4.5" height="4.5" rx="1" />
            <rect x="7" y="7" width="4.5" height="4.5" rx="1" />
          </svg>
          卡片
        </button>
        <button
          className={`toggle-btn ${mode === 'bar' ? 'active' : ''}`}
          onClick={() => handleChange('bar')}
        >
          <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
            <rect x="0.5" y="1.5" width="11" height="3" rx="1" />
            <rect x="0.5" y="7.5" width="11" height="3" rx="1" />
          </svg>
          顶栏
        </button>
      </div>
    </div>
  );
}
