export interface NavItem {
  href: string;
  label: string;
  icon: string;
}

export const NAV: NavItem[] = [
  { href: '/', label: 'Dashboard', icon: '🏠' },
  { href: '/servers', label: 'Servers', icon: '🗄️' },
  { href: '/worlds', label: 'Worlds', icon: '🌍' },
  { href: '/addons', label: 'Addons', icon: '🧩' },
  { href: '/backups', label: 'Backups', icon: '💾' },
  { href: '/console', label: 'Console', icon: '🖥️' },
  { href: '/players', label: 'Players', icon: '👥' },
  { href: '/network', label: 'Network', icon: '🛡️' },
  { href: '/settings', label: 'Settings', icon: '⚙️' },
];

export function isActive(pathname: string, href: string): boolean {
  return href === '/' ? pathname === '/' : pathname.startsWith(href);
}

export function activeLabel(pathname: string): string {
  const match = [...NAV]
    .sort((a, b) => b.href.length - a.href.length)
    .find((n) => isActive(pathname, n.href));
  return match?.label ?? '';
}
