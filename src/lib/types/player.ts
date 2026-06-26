export interface Player {
  name: string;
  xuid: string;
  connectedAt: string;
}

export interface PlayerEvent {
  serverId: string;
  /** "connected" | "disconnected" */
  event: string;
  name: string;
  xuid: string;
  at: string;
}
