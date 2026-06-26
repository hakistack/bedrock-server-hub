export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion: string;
  notes: string;
  downloadUrl: string | null;
  assetSize: number | null;
  supported: boolean;
}

export interface UpdateProgress {
  downloadedBytes: number;
  totalBytes: number | null;
  percentage: number | null;
}
