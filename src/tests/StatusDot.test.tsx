import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { StatusDot } from '../components/StatusDot';

describe('StatusDot', () => {
  it('renders running state with animation class', () => {
    const { container } = render(<StatusDot state="running" />);
    const dot = container.querySelector('.dot');
    expect(dot?.className).toContain('dot-running');
    expect(dot?.className).toContain('dot');
  });

  it('renders confirm state with alert animation', () => {
    const { container } = render(<StatusDot state="confirm" />);
    const dot = container.querySelector('.dot');
    expect(dot?.className).toContain('dot-confirm');
  });

  it('renders idle state without pulse animation', () => {
    const { container } = render(<StatusDot state="idle" />);
    const dot = container.querySelector('.dot');
    expect(dot?.className).toContain('dot-idle');
    expect(dot?.className).not.toContain('pulse');
  });

  it('renders offline state without pulse animation', () => {
    const { container } = render(<StatusDot state="offline" />);
    const dot = container.querySelector('.dot');
    expect(dot?.className).toContain('dot-offline');
  });

  it('renders all four states with distinct classes', () => {
    const states = ['running', 'confirm', 'idle', 'offline'] as const;
    states.forEach((state) => {
      const { container } = render(<StatusDot state={state} />);
      const dot = container.querySelector(`.dot-${state}`);
      expect(dot).toBeTruthy();
    });
  });
});
