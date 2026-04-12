import { useEffect } from 'react';
import './ErrorToast.css';

interface ErrorToastProps {
  error: string | null;
  onDismiss: () => void;
}

export function ErrorToast({ error, onDismiss }: ErrorToastProps) {
  useEffect(() => {
    if (!error) return;
    const timer = setTimeout(onDismiss, 5000);
    return () => clearTimeout(timer);
  }, [error, onDismiss]);

  if (!error) return null;

  return (
    <div className="error-toast" role="alert">
      <span className="error-msg">{error}</span>
      <button className="error-close" onClick={onDismiss} aria-label="关闭">
        ✕
      </button>
    </div>
  );
}
