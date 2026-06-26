import type { ServerMetrics } from '$lib/types/server';

const MAX_POINTS = 150;

class MetricsStore {
  byServer = $state<Record<string, ServerMetrics[]>>({});

  append(m: ServerMetrics) {
    const current = this.byServer[m.serverId] ?? [];
    const next = current.length >= MAX_POINTS ? current.slice(current.length - MAX_POINTS + 1) : current.slice();
    next.push(m);
    this.byServer = { ...this.byServer, [m.serverId]: next };
  }

  get(serverId: string | null): ServerMetrics[] {
    return serverId ? (this.byServer[serverId] ?? []) : [];
  }
}

export const metricsStore = new MetricsStore();
