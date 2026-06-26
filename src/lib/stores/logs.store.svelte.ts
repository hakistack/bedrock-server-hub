import type { LogLine } from '$lib/types/server';

const MAX_LINES = 2000;

class LogsStore {
  byServer = $state<Record<string, LogLine[]>>({});

  append(log: LogLine) {
    const current = this.byServer[log.serverId] ?? [];
    const next = current.length >= MAX_LINES ? current.slice(current.length - MAX_LINES + 1) : current.slice();
    next.push(log);
    this.byServer = { ...this.byServer, [log.serverId]: next };
  }

  get(serverId: string | null): LogLine[] {
    return serverId ? (this.byServer[serverId] ?? []) : [];
  }

  clear(serverId: string) {
    this.byServer = { ...this.byServer, [serverId]: [] };
  }
}

export const logsStore = new LogsStore();
