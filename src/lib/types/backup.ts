export type BackupReason =
  | 'addon_install'
  | 'world_import'
  | 'properties_edit'
  | 'manual'
  | 'pre_restore';

export interface BackupRecord {
  id: string;
  serverId: string;
  worldName: string | null;
  reason: BackupReason | string;
  path: string;
  createdAt: string;
}

export interface BackupProgress {
  backupId: string;
  /** "starting" | "zipping" | "completed" */
  phase: string;
  done: number;
  total: number;
}

export interface RestoreOptions {
  restoreWorld: boolean;
  restoreProperties: boolean;
}

export interface BackupSchedule {
  enabled: boolean;
  /** "interval" | "daily" */
  mode: string;
  intervalMinutes: number;
  dailyTime: string;
  retention: number;
}
