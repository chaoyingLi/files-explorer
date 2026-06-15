// Native Drag & Drop — cross-application file transfer
//
// Drag IN: Tauri onDragDropEvent captures native file paths from Explorer / other apps
// Drag OUT: text/uri-list + clipboard CF_HDROP for dragging files to desktop / other apps
//
// WARNING: This composable duplicates the native drop handling already in App.vue's
// onMounted. Do NOT use this composable while App.vue's handler is active, or drops
// will be processed twice. This file is retained as a standalone alternative for
// scenarios where you want drop handling in a child component instead of App.vue.

import { ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export interface DropPayload {
  paths: string[]; // Absolute file paths
  position: { x: number; y: number };
}

export function useNativeDrop(onDrop: (payload: DropPayload) => void) {
  const isDragOver = ref(false);

  async function init() {
    const win = getCurrentWebviewWindow();

    // Tauri native file drop from OS (Explorer, other apps)
    const unlisten = await win.onDragDropEvent((event) => {
      const e = event.payload;

      if (e.type === "over") {
        isDragOver.value = true;
      } else if (e.type === "leave") {
        isDragOver.value = false;
      } else if (e.type === "drop") {
        isDragOver.value = false;
        // Extract file paths from native drop
        const paths: string[] = e.paths
          .map((p: any) => {
            // path can be string or object with path property
            return typeof p === "string" ? p : p.path || "";
          })
          .filter(Boolean);

        if (paths.length > 0) {
          onDrop({ paths, position: { x: e.position.x, y: e.position.y } });
        }
      }
    });

    return unlisten;
  }

  return { isDragOver, init };
}
