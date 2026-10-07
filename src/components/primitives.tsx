import type { ButtonHTMLAttributes, ReactNode } from 'react';

export type IconName = 'memory' | 'search' | 'refresh' | 'chevron' | 'history' | 'settings' | 'close' | 'back' | 'arrow' | 'mic' | 'command' | 'harness';
const paths: Record<IconName, ReactNode> = {
  memory: <><path d="M4 4h7a3 3 0 0 1 3 3v14a4 4 0 0 0-4-2H4ZM20 4h-3a3 3 0 0 0-3 3M14 21a4 4 0 0 1 4-2h2V4" /></>,
  search: <><circle cx="10.5" cy="10.5" r="5.5" /><path d="m15 15 4 4" /></>,
  refresh: <><path d="M19 8a7 7 0 0 0-12.2-1.8L5 8M5 4v4h4M5 16a7 7 0 0 0 12.2 1.8L19 16M19 20v-4h-4" /></>,
  chevron: <path d="m8 10 4 4 4-4" />, 
  history: <><path d="M4 8a9 9 0 1 1-1 8M4 3v5h5M12 7v5l3 2" /></>,
  settings: <><path d="m9 3 1-1h4l1 1 .5 2 2 1 2-.5 2 3-1.5 1.5v2L21 14l-2 3-2-.5-2 1-.5 2-1 1h-4l-1-1-.5-2-2-1-2 .5-2-3 1.5-1.5v-2L3 9l2-3 2 .5 2-1Z" /><circle cx="12" cy="11" r="3" /></>,
  close: <path d="m6 6 12 12M18 6 6 18" />,
  back: <path d="m14 5-7 7 7 7M7 12h13" />,
  arrow: <path d="M12 19V5m-6 6 6-6 6 6" />,
  mic: <><rect x="9" y="3" width="6" height="12" rx="3" /><path d="M5 11v1a7 7 0 0 0 14 0v-1M12 19v3m-3 0h6" /></>,
  command: <><path d="M8 8h8v8H8ZM8 8V5a3 3 0 1 0-3 3h3m8 0h3a3 3 0 1 0-3-3v3m0 8v3a3 3 0 1 0 3-3h-3m-8 0H5a3 3 0 1 0 3 3v-3" /></>,
  harness: <><path d="M5 4v16M19 4v16M5 12h14" /><path d="m9 7 3 5 3-5m-6 10 3-5 3 5" /></>,
};

export function Icon({ name }: { name: IconName }) {
  return <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{paths[name]}</svg>;
}

export function Button({ primary, quiet, className = '', ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { primary?: boolean; quiet?: boolean }) {
  return <button type="button" {...props} className={`button${primary ? ' primary' : ''}${quiet ? ' quiet' : ''} ${className}`} />;
}

export function IconButton({ icon, label, ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { icon: IconName; label: string }) {
  return <button type="button" {...props} className="icon-button" aria-label={label} title={label}><Icon name={icon} /></button>;
}
