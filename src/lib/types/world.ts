export interface World {
  name: string;
  displayName: string | null;
  path: string;
  isActive: boolean;
  sizeBytes: number;
  hasBehaviorPacks: boolean;
  hasResourcePacks: boolean;
}
