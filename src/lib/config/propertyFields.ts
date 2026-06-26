import type { PropertyFieldSpec } from '$lib/types/properties';

/** Section order for the grouped Settings UI. */
export const PROPERTY_GROUPS = ['General', 'Gameplay', 'Players', 'Network', 'Advanced'] as const;

/**
 * Curated `server.properties` keys surfaced as friendly controls, grouped into
 * sections with short descriptions. Only keys present in the file are rendered;
 * everything else stays editable through the "advanced" raw list.
 */
export const PROPERTY_FIELDS: PropertyFieldSpec[] = [
  {
    key: 'server-name',
    label: 'Nombre del servidor',
    control: 'text',
    group: 'General',
    help: 'Cómo aparece tu servidor en la lista de Minecraft.',
  },
  {
    key: 'level-name',
    label: 'Mundo activo',
    control: 'text',
    group: 'General',
    help: 'Carpeta del mundo dentro de worlds/ que carga el servidor.',
  },
  {
    key: 'gamemode',
    label: 'Modo de juego',
    control: 'select',
    group: 'Gameplay',
    options: ['survival', 'creative', 'adventure'],
    help: 'Modo por defecto para jugadores nuevos.',
  },
  {
    key: 'difficulty',
    label: 'Dificultad',
    control: 'select',
    group: 'Gameplay',
    options: ['peaceful', 'easy', 'normal', 'hard'],
  },
  {
    key: 'allow-cheats',
    label: 'Permitir trucos',
    control: 'boolean',
    group: 'Gameplay',
    help: 'Habilita comandos como /gamemode o /give en el mundo.',
  },
  {
    key: 'max-players',
    label: 'Máx. jugadores',
    control: 'number',
    group: 'Players',
    help: 'Cuántos jugadores pueden conectarse a la vez.',
  },
  {
    key: 'default-player-permission-level',
    label: 'Permiso por defecto',
    control: 'select',
    group: 'Players',
    options: ['visitor', 'member', 'operator'],
    help: 'Nivel de permisos que recibe un jugador nuevo.',
  },
  {
    key: 'server-port',
    label: 'Puerto (IPv4)',
    control: 'number',
    group: 'Network',
    help: 'Puerto UDP en el que escucha el servidor (por defecto 19132).',
  },
  {
    key: 'server-portv6',
    label: 'Puerto (IPv6)',
    control: 'number',
    group: 'Network',
  },
  {
    key: 'online-mode',
    label: 'Modo online (Xbox Live)',
    control: 'boolean',
    group: 'Network',
    help: 'Exige autenticación con cuenta de Xbox Live para entrar.',
  },
  {
    key: 'view-distance',
    label: 'Distancia de vista',
    control: 'number',
    group: 'Advanced',
    help: 'Chunks que el servidor envía. Más alto = más CPU/red.',
  },
  {
    key: 'tick-distance',
    label: 'Distancia de tick',
    control: 'number',
    group: 'Advanced',
    help: 'Radio de chunks que se simulan activamente (4–12).',
  },
];

export const FIELD_KEYS = new Set(PROPERTY_FIELDS.map((f) => f.key));
