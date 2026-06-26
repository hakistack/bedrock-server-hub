import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

import type {
  BedrockServer,
  LogLine,
  ServerMetrics,
  ServerSettings,
  ServerStatus,
  StatusEvent,
  ValidationResult,
} from '$lib/types/server';
import { openUrl, openPath } from '@tauri-apps/plugin-opener';

import type { PropertyEntry, PropertyUpdate } from '$lib/types/properties';
import type { World } from '$lib/types/world';
import type {
  BackupProgress,
  BackupRecord,
  BackupSchedule,
  RestoreOptions,
} from '$lib/types/backup';
import type { Player, PlayerEvent } from '$lib/types/player';
import type {
  DownloadProgress,
  DownloadedServerPackage,
  ServerDownloadOption,
} from '$lib/types/download';
import type {
  AddonInstallReport,
  AddonPackage,
  InstalledAddon,
  WorldPacks,
} from '$lib/types/addon';
import type { NetworkStatus } from '$lib/types/network';
import type { UpdateInfo, UpdateProgress } from '$lib/types/update';

/** Official Minecraft Bedrock Dedicated Server download page. */
export const OFFICIAL_DOWNLOAD_PAGE =
  'https://www.minecraft.net/en-us/download/server/bedrock';
export const MINECRAFT_EULA = 'https://www.minecraft.net/en-us/eula';
export const MINECRAFT_PRIVACY = 'https://privacy.microsoft.com/en-us/privacystatement';

/** Open a URL in the user's default browser. */
export function openInBrowser(url: string): Promise<void> {
  return openUrl(url);
}

/** Reveal a folder/file in the OS file explorer. */
export function openInFolder(path: string): Promise<void> {
  return openPath(path);
}

/** Open a native folder picker. Returns the chosen path or null if cancelled. */
export async function pickFolder(title = 'Selecciona la carpeta del servidor'): Promise<string | null> {
  const result = await open({ directory: true, multiple: false, title });
  return typeof result === 'string' ? result : null;
}

/** Open a native file picker constrained to the given extensions. */
export async function pickFile(
  extensions: string[],
  name: string,
  title = 'Selecciona un archivo',
): Promise<string | null> {
  const result = await open({
    directory: false,
    multiple: false,
    title,
    filters: [{ name, extensions }],
  });
  return typeof result === 'string' ? result : null;
}

/** Open a native file picker allowing multiple files. */
export async function pickFiles(
  extensions: string[],
  name: string,
  title = 'Selecciona archivos',
): Promise<string[]> {
  const result = await open({
    directory: false,
    multiple: true,
    title,
    filters: [{ name, extensions }],
  });
  if (Array.isArray(result)) return result;
  return typeof result === 'string' ? [result] : [];
}

/**
 * Typed wrappers around every Tauri command. Tauri maps camelCase JS keys to
 * the snake_case Rust parameters automatically.
 */
export const api = {
  validateServerFolder: (path: string) =>
    invoke<ValidationResult>('validate_server_folder', { path }),

  importServer: (path: string, name?: string) =>
    invoke<BedrockServer>('import_server', { path, name: name ?? null }),

  listServers: () => invoke<BedrockServer[]>('list_servers'),

  getServer: (serverId: string) => invoke<BedrockServer>('get_server', { serverId }),

  renameServer: (serverId: string, name: string) =>
    invoke<BedrockServer>('rename_server', { serverId, name }),

  removeServer: (serverId: string) => invoke<void>('remove_server', { serverId }),

  startServer: (serverId: string) => invoke<void>('start_server', { serverId }),

  stopServer: (serverId: string) => invoke<void>('stop_server', { serverId }),

  restartServer: (serverId: string) => invoke<void>('restart_server', { serverId }),

  getServerStatus: (serverId: string) =>
    invoke<ServerStatus>('get_server_status', { serverId }),

  getServerSettings: (serverId: string) =>
    invoke<ServerSettings>('get_server_settings', { serverId }),

  setAutoRestart: (serverId: string, enabled: boolean) =>
    invoke<void>('set_auto_restart', { serverId, enabled }),

  getOnlinePlayers: (serverId: string) =>
    invoke<Player[]>('get_online_players', { serverId }),

  sendServerCommand: (serverId: string, command: string) =>
    invoke<void>('send_server_command', { serverId, command }),

  readProperties: (serverId: string) =>
    invoke<PropertyEntry[]>('read_properties', { serverId }),

  updateProperties: (serverId: string, updates: PropertyUpdate[]) =>
    invoke<PropertyEntry[]>('update_properties', { serverId, updates }),

  // --- Worlds (phase 2) ---
  listWorlds: (serverId: string) => invoke<World[]>('list_worlds', { serverId }),

  importWorld: (serverId: string, mcworldPath: string, makeActive: boolean) =>
    invoke<World>('import_world', { serverId, mcworldPath, makeActive }),

  activateWorld: (serverId: string, worldName: string) =>
    invoke<void>('activate_world', { serverId, worldName }),

  // --- Backups (phase 2) ---
  listBackups: (serverId: string) => invoke<BackupRecord[]>('list_backups', { serverId }),

  createBackup: (serverId: string, worldName?: string) =>
    invoke<BackupRecord>('create_backup', { serverId, worldName: worldName ?? null }),

  restoreBackup: (backupId: string, options: RestoreOptions) =>
    invoke<BackupRecord>('restore_backup', { backupId, options }),

  deleteBackup: (backupId: string) => invoke<void>('delete_backup', { backupId }),

  getBackupSchedule: (serverId: string) =>
    invoke<BackupSchedule>('get_backup_schedule', { serverId }),

  setBackupSchedule: (serverId: string, schedule: BackupSchedule) =>
    invoke<void>('set_backup_schedule', { serverId, schedule }),

  // --- Official server downloader (Create Server wizard) ---
  getOfficialDownloadOptions: () =>
    invoke<ServerDownloadOption[]>('get_official_server_download_options'),

  resolveManualDownloadUrl: (url: string) =>
    invoke<ServerDownloadOption>('resolve_manual_download_url', { url }),

  downloadBedrockServer: (
    option: ServerDownloadOption,
    downloadId: string,
    acceptedEula: boolean,
  ) =>
    invoke<DownloadedServerPackage>('download_bedrock_server', {
      option,
      downloadId,
      acceptedEula,
    }),

  installDownloadedServer: (
    pkg: DownloadedServerPackage,
    targetDirectory: string,
    serverName: string,
  ) =>
    invoke<BedrockServer>('install_downloaded_server', {
      package: pkg,
      targetDirectory,
      serverName,
    }),

  cancelDownload: (downloadId: string) => invoke<void>('cancel_download', { downloadId }),

  // --- Addons (phase 3) ---
  previewAddon: (sourcePath: string) =>
    invoke<AddonPackage>('preview_addon', { sourcePath }),

  installAddon: (
    serverId: string,
    worldName: string,
    sourcePath: string,
    selectedUuids?: string[],
  ) =>
    invoke<AddonInstallReport>('install_addon', {
      serverId,
      worldName,
      sourcePath,
      selectedUuids: selectedUuids ?? null,
    }),

  installAddons: (
    serverId: string,
    worldName: string,
    items: { sourcePath: string; selectedUuids: string[] }[],
  ) => invoke<AddonInstallReport>('install_addons', { serverId, worldName, items }),

  listInstalledAddons: (serverId: string) =>
    invoke<InstalledAddon[]>('list_installed_addons', { serverId }),

  uninstallAddon: (serverId: string, worldName: string, uuid: string) =>
    invoke<boolean>('uninstall_addon', { serverId, worldName, uuid }),

  listWorldPacks: (serverId: string, worldName: string) =>
    invoke<WorldPacks>('list_world_packs', { serverId, worldName }),

  reorderWorldPacks: (
    serverId: string,
    worldName: string,
    packType: 'behavior' | 'resource',
    orderedUuids: string[],
  ) =>
    invoke<WorldPacks>('reorder_world_packs', { serverId, worldName, packType, orderedUuids }),

  // --- Network / firewall ---
  getNetworkStatus: (serverId: string) =>
    invoke<NetworkStatus>('get_network_status', { serverId }),

  addFirewallRules: (serverId: string) =>
    invoke<NetworkStatus>('add_firewall_rules', { serverId }),

  assignFreePort: (serverId: string) =>
    invoke<NetworkStatus>('assign_free_port', { serverId }),

  // --- Self-updater ---
  checkForUpdate: () => invoke<UpdateInfo>('check_for_update'),

  downloadAndInstallUpdate: (downloadUrl: string) =>
    invoke<void>('download_and_install_update', { downloadUrl }),
};

/** Subscribe to self-update download progress. */
export function onUpdateProgress(handler: (p: UpdateProgress) => void): Promise<UnlistenFn> {
  return listen<UpdateProgress>('update://progress', (event) => handler(event.payload));
}

/** Subscribe to backup zip progress events. */
export function onBackupProgress(
  handler: (p: BackupProgress) => void,
): Promise<UnlistenFn> {
  return listen<BackupProgress>('backup://progress', (event) => handler(event.payload));
}

/** Subscribe to download progress events. */
export function onDownloadProgress(
  handler: (p: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<DownloadProgress>('download://progress', (event) => handler(event.payload));
}

/** Subscribe to live log lines from any server process. */
export function onServerLog(handler: (log: LogLine) => void): Promise<UnlistenFn> {
  return listen<LogLine>('server://log', (event) => handler(event.payload));
}

/** Subscribe to server status transitions. */
export function onServerStatus(handler: (status: StatusEvent) => void): Promise<UnlistenFn> {
  return listen<StatusEvent>('server://status', (event) => handler(event.payload));
}

/** Subscribe to server CPU/RAM metrics samples. */
export function onServerMetrics(handler: (m: ServerMetrics) => void): Promise<UnlistenFn> {
  return listen<ServerMetrics>('server://metrics', (event) => handler(event.payload));
}

/** Subscribe to player connect/disconnect events. */
export function onPlayerEvent(handler: (e: PlayerEvent) => void): Promise<UnlistenFn> {
  return listen<PlayerEvent>('server://player', (event) => handler(event.payload));
}
