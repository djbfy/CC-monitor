import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SilentBar } from '../components/SilentBar';

describe('SilentBar', () => {
  it('maps silentSecs to correct bar fill width', () => {
    const { container: c1 } = render(<SilentBar state="running" silentSecs={5} maxSecs={60} />);
    const fill1 = c1.querySelector('.bar-fill') as HTMLElement;
    expect(parseFloat(fill1.style.width)).toBeCloseTo(8.33, 1);

    const { container: c2 } = render(<SilentBar state="running" silentSecs={30} maxSecs={60} />);
    const fill2 = c2.querySelector('.bar-fill') as HTMLElement;
    expect(fill2.style.width).toBe('50%');
  });

  it('caps width at 100% when silentSecs exceeds maxSecs', () => {
    const { container } = render(<SilentBar state="idle" silentSecs={90} maxSecs={60} />);
    const fill = container.querySelector('.bar-fill') as HTMLElement;
    expect(fill.style.width).toBe('100%');
  });

  it('shows silentSecs in bar-lbl', () => {
    render(<SilentBar state="running" silentSecs={12} maxSecs={60} />);
    expect(screen.getByText('12s')).toBeInTheDocument();
  });

  it('uses correct color class for each state', () => {
    const states = ['running', 'confirm', 'idle', 'offline'] as const;
    const expectedClasses = ['bar-fill-running', 'bar-fill-confirm', 'bar-fill-idle', 'bar-fill-offline'];
    states.forEach((state, i) => {
      const { container } = render(<SilentBar state={state} silentSecs={10} maxSecs={60} />);
      const fill = container.querySelector('.bar-fill');
      expect(fill?.className).toContain(expectedClasses[i]);
    });
  });
});
