import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { Action } from 'svelte/action';

export interface FileDropParams {
  /** Lowercase extensions (without dot) accepted by this zone. */
  extensions: string[];
  /** Called with the first accepted path when files are dropped on the node. */
  onDrop: (path: string) => void;
  /** Toggled while a drag hovers this node — use to style the zone. */
  onHover?: (hovering: boolean) => void;
}

function matchesExt(path: string, extensions: string[]): boolean {
  const lower = path.toLowerCase();
  return extensions.some((ext) => lower.endsWith(`.${ext.toLowerCase()}`));
}

/**
 * Svelte action: accept OS file drops on a specific element.
 *
 * Tauri's drag-drop event is window-global and carries a physical-pixel
 * position, so we hit-test it against the node's rect to support several
 * independent drop zones on the same page.
 */
export const fileDrop: Action<HTMLElement, FileDropParams> = (node, params) => {
  let current = params;
  let unlisten: (() => void) | null = null;
  let disposed = false;

  function within(physicalX: number, physicalY: number): boolean {
    const dpr = window.devicePixelRatio || 1;
    const x = physicalX / dpr;
    const y = physicalY / dpr;
    const r = node.getBoundingClientRect();
    return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
  }

  getCurrentWebview()
    .onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === 'over') {
        const hovering = within(payload.position.x, payload.position.y);
        current.onHover?.(hovering);
      } else if (payload.type === 'drop') {
        const hovering = within(payload.position.x, payload.position.y);
        current.onHover?.(false);
        if (!hovering) return;
        // Deliver every matching dropped file (supports batch drops).
        for (const p of payload.paths.filter((p) => matchesExt(p, current.extensions))) {
          current.onDrop(p);
        }
      } else {
        // 'leave' / 'enter'
        current.onHover?.(false);
      }
    })
    .then((un) => {
      if (disposed) un();
      else unlisten = un;
    });

  return {
    update(next: FileDropParams) {
      current = next;
    },
    destroy() {
      disposed = true;
      if (unlisten) unlisten();
    },
  };
};
