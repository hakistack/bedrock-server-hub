export type ServerStatus =
  | 'offline'
  | 'starting'
  | 'online'
  | 'stopping'
  | 'crashed';

export interface BedrockServer {
  id: string;
  name: string;
  path: string;
  executablePath: string;
  propertiesPath: string;
  worldsPath: string;
  createdAt: string;
  updatedAt: string;
  serverVersion?: string | null;
  installSource?: string | null;
  platform?: string | null;
  channel?: string | null;
  createdFromDownload?: boolean;
}

export interface ValidationResult {
  isValid: boolean;
  executablePath: string | null;
  propertiesPath: string | null;
  worldsPath: string | null;
  issues: string[];
}

export interface LogLine {
  serverId: string;
  stream: 'stdout' | 'stderr';
  line: string;
}

export interface StatusEvent {
  serverId: string;
  status: ServerStatus;
}

/** Serialized shape of every error returned by a Tauri command. */
export interface AppError {
  code: string;
  message: string;
}

export interface ServerSettings {
  autoRestart: boolean;
}

export interface ServerMetrics {
  serverId: string;
  cpu: number;
  memoryBytes: number;
}
