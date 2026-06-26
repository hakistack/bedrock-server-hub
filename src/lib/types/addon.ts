export type PackType = 'behavior' | 'resource' | 'skin' | 'unknown';

export interface AddonPack {
  name: string;
  description: string | null;
  uuid: string;
  version: number[];
  packType: PackType;
  sourceExtractedPath: string;
}

export interface AddonPackage {
  id: string;
  sourcePath: string;
  displayName: string;
  packs: AddonPack[];
}

export interface InstalledPack {
  name: string;
  uuid: string;
  version: number[];
  packType: PackType;
  /** "installed" | "updated" | "skipped" | "unsupported" */
  status: string;
  message: string | null;
}

export interface AddonInstallReport {
  worldName: string;
  results: InstalledPack[];
  warnings: string[];
}

export interface InstalledAddon {
  id: string;
  serverId: string;
  worldName: string;
  name: string;
  uuid: string;
  version: string;
  packType: string;
  installedAt: string;
}
