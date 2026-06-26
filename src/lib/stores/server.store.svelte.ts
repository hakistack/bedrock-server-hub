import { api } from '$lib/api/commands';
import type { BedrockServer, ServerStatus } from '$lib/types/server';

const LAST_SELECTED_KEY = 'bsm.selectedServerId';

class ServerStore {
  servers = $state<BedrockServer[]>([]);
  selectedId = $state<string | null>(null);
  statuses = $state<Record<string, ServerStatus>>({});
  loading = $state(false);
  loaded = $state(false);

  get selected(): BedrockServer | null {
    return this.servers.find((s) => s.id === this.selectedId) ?? null;
  }

  get selectedStatus(): ServerStatus {
    return this.statusOf(this.selectedId);
  }

  statusOf(id: string | null): ServerStatus {
    return id ? (this.statuses[id] ?? 'offline') : 'offline';
  }

  async refresh() {
    this.loading = true;
    try {
      this.servers = await api.listServers();

      // Restore / reconcile the selected server.
      const remembered =
        typeof localStorage !== 'undefined' ? localStorage.getItem(LAST_SELECTED_KEY) : null;
      const valid = (id: string | null) => !!id && this.servers.some((s) => s.id === id);

      if (!valid(this.selectedId)) {
        this.selectedId = valid(remembered) ? remembered : (this.servers[0]?.id ?? null);
      }

      await Promise.all(
        this.servers.map(async (s) => {
          this.statuses[s.id] = await api.getServerStatus(s.id);
        }),
      );
    } finally {
      this.loading = false;
      this.loaded = true;
    }
  }

  select(id: string) {
    this.selectedId = id;
    if (typeof localStorage !== 'undefined') localStorage.setItem(LAST_SELECTED_KEY, id);
  }

  setStatus(id: string, status: ServerStatus) {
    this.statuses = { ...this.statuses, [id]: status };
  }

  upsert(server: BedrockServer) {
    const idx = this.servers.findIndex((s) => s.id === server.id);
    if (idx >= 0) this.servers[idx] = server;
    else this.servers = [...this.servers, server];
  }

  remove(id: string) {
    this.servers = this.servers.filter((s) => s.id !== id);
    if (this.selectedId === id) this.selectedId = this.servers[0]?.id ?? null;
  }
}

export const serverStore = new ServerStore();
