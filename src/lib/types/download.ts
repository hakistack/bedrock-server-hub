export type ServerPlatform = 'Windows' | 'Linux';
export type ServerChannel = 'Stable' | 'Preview';
export type DownloadStatus =
  | 'Starting'
  | 'Downloading'
  | 'Completed'
  | 'Failed'
  | 'Cancelled';

export interface ServerDownloadOption {
  id: string;
  label: string;
  platform: ServerPlatform;
  channel: ServerChannel;
  url: string;
  version: string | null;
}

export interface DownloadProgress {
  downloadId: string;
  bytesDownloaded: number;
  totalBytes?: number | null;
  percentage?: number | null;
  status: DownloadStatus;
}

export interface DownloadedServerPackage {
  downloadId: string;
  path: string;
  sizeBytes: number;
  option: ServerDownloadOption;
}
