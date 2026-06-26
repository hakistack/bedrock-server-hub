export interface World {
  name: string;
  displayName: string | null;
  path: string;
  isActive: boolean;
  sizeBytes: number;
  modifiedAt: string | null;
  hasBehaviorPacks: boolean;
  hasResourcePacks: boolean;
}
