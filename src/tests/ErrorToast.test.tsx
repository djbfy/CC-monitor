import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { ErrorToast } from '../components/ErrorToast';

describe('ErrorToast', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  it('does not render when error is null', () => {
    render(<ErrorToast error={null} onDismiss={vi.fn()} />);
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('renders error message when error is set', () => {
    render(<ErrorToast error="Something went wrong" onDismiss={vi.fn()} />);
    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('calls onDismiss when close button is clicked', () => {
    const dismissSpy = vi.fn();
    render(<ErrorToast error="Test error" onDismiss={dismissSpy} />);
    fireEvent.click(screen.getByLabelText('关闭'));
    expect(dismissSpy).toHaveBeenCalled();
  });

  it('auto-dismisses after 5 seconds', () => {
    const dismissSpy = vi.fn();
    render(<ErrorToast error="Auto dismiss" onDismiss={dismissSpy} />);
    expect(dismissSpy).not.toHaveBeenCalled();
    act(() => { vi.advanceTimersByTime(5000); });
    expect(dismissSpy).toHaveBeenCalled();
  });

  it('does not auto-dismiss when error is null', () => {
    const dismissSpy = vi.fn();
    render(<ErrorToast error={null} onDismiss={dismissSpy} />);
    act(() => { vi.advanceTimersByTime(10000); });
    expect(dismissSpy).not.toHaveBeenCalled();
  });
});
