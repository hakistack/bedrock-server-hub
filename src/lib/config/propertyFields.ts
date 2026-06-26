import type { PropertyFieldSpec } from '$lib/types/properties';

/**
 * The curated set of `server.properties` keys surfaced as friendly controls.
 * Only keys actually present in the file are rendered; everything else is
 * still editable through the "advanced" raw list so nothing is hidden.
 */
export const PROPERTY_FIELDS: PropertyFieldSpec[] = [
  { key: 'server-name', label: 'Nombre del servidor', control: 'text' },
  {
    key: 'gamemode',
    label: 'Modo de juego',
    control: 'select',
    options: ['survival', 'creative', 'adventure'],
  },
  {
    key: 'difficulty',
    label: 'Dificultad',
    control: 'select',
    options: ['peaceful', 'easy', 'normal', 'hard'],
  },
  { key: 'allow-cheats', label: 'Permitir trucos', control: 'boolean' },
  { key: 'max-players', label: 'Máx. jugadores', control: 'number' },
  { key: 'online-mode', label: 'Modo online (auth Xbox Live)', control: 'boolean' },
  { key: 'server-port', label: 'Puerto (IPv4)', control: 'number' },
  { key: 'server-portv6', label: 'Puerto (IPv6)', control: 'number' },
  { key: 'level-name', label: 'Mundo activo (level-name)', control: 'text' },
  { key: 'view-distance', label: 'Distancia de vista', control: 'number' },
  { key: 'tick-distance', label: 'Distancia de tick', control: 'number' },
  {
    key: 'default-player-permission-level',
    label: 'Permiso por defecto',
    control: 'select',
    options: ['visitor', 'member', 'operator'],
  },
];

export const FIELD_KEYS = new Set(PROPERTY_FIELDS.map((f) => f.key));
