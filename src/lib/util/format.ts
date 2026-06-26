/** Human-readable byte size, e.g. 1536 → "1.5 KB". */
export function humanSize(bytes: number): string {
  if (bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

/** Format an ISO timestamp using the local locale. */
export function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString();
}

const REASON_LABELS: Record<string, string> = {
  addon_install: 'Instalación de addon',
  world_import: 'Importación de mundo',
  properties_edit: 'Edición de propiedades',
  manual: 'Manual',
  pre_restore: 'Previo a restaurar',
};

export function backupReasonLabel(reason: string): string {
  return REASON_LABELS[reason] ?? reason;
}
