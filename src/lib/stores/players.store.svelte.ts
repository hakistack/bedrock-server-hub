import type { Player, PlayerEvent } from '$lib/types/player';

const MAX_EVENTS = 100;

class PlayersStore {
  online = $state<Record<string, Player[]>>({});
  events = $state<Record<string, PlayerEvent[]>>({});

  setOnline(serverId: string, players: Player[]) {
    this.online = { ...this.online, [serverId]: players };
  }

  applyEvent(e: PlayerEvent) {
    // Update online list.
    const current = this.online[e.serverId] ?? [];
    let next: Player[];
    if (e.event === 'connected') {
      next = current.some((p) => p.xuid === e.xuid)
        ? current
        : [...current, { name: e.name, xuid: e.xuid, connectedAt: e.at }];
    } else {
      next = current.filter((p) => p.xuid !== e.xuid);
    }
    this.online = { ...this.online, [e.serverId]: next };

    // Append to activity feed (most recent first).
    const feed = this.events[e.serverId] ?? [];
    const trimmed = [e, ...feed].slice(0, MAX_EVENTS);
    this.events = { ...this.events, [e.serverId]: trimmed };
  }

  onlineOf(serverId: string | null): Player[] {
    return serverId ? (this.online[serverId] ?? []) : [];
  }

  eventsOf(serverId: string | null): PlayerEvent[] {
    return serverId ? (this.events[serverId] ?? []) : [];
  }
}

export const playersStore = new PlayersStore();
