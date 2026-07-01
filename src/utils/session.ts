// ── Session persistence: save & restore app state across restarts ──

import type { FileEntry } from "@/types";
import type { ColumnState } from "@/stores/viewStore";

// ── Persisted types (without runtime-only data) ──

export interface PersistedTab {
  path: string;
  title: string;
}

export interface PersistedPane {
  type: "pane";
  tabs: PersistedTab[];
  activeTabIndex: number;
}

export interface PersistedSplit {
  type: "split";
  direction: "horizontal" | "vertical";
  children: PersistedNode[];
  sizes: number[];
}

export type PersistedNode = PersistedPane | PersistedSplit;

export interface SessionSnapshot {
  version: 1;
  savedAt: number;
  viewMode: "details" | "list" | "grid" | "tree" | "column";
  propertiesOpen: boolean;
  layout: PersistedNode | null;
  navigationHistory: string[];
  navigationIndex: number;
  focusPaneIndexPath: number[];
}

// ── Runtime types for deserialization ──

export interface Tab {
  id: string;
  path: string;
  title: string;
  files: FileEntry[];
  selectedFiles: string[];
}

export interface LayoutPane {
  type: "pane";
  id: string;
  tabs: Tab[];
  activeTabId: string;
}

export interface LayoutSplit {
  type: "split";
  id: string;
  direction: "horizontal" | "vertical";
  children: LayoutNode[];
  sizes: number[];
}

export type LayoutNode = LayoutPane | LayoutSplit;

// ── Keys ──
const STORAGE_KEY = "app-session";

// ── Save ──

export function saveSession(snapshot: SessionSnapshot): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  } catch (e) {
    console.error("Failed to save session:", e);
  }
}

// ── Load ──

export function loadSession(): SessionSnapshot | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (parsed?.version !== 1) return null;
    return parsed as SessionSnapshot;
  } catch {
    return null;
  }
}

// ── Find pane index path in a layout tree ──

function findPaneIndexPath(
  node: PersistedNode,
  targetPaneId: string,
  path: number[],
): number[] | null {
  if (node.type === "pane") {
    return path; // leaf pane found at this path
  }
  // split node: search children
  for (let i = 0; i < node.children.length; i++) {
    const result = findPaneIndexPath(node.children[i], targetPaneId, [
      ...path,
      i,
    ]);
    if (result) return result;
  }
  return null;
}

/**
 * Serialize the runtime layout to a persistable structure.
 * Strips: ids, files[], selectedFiles[], columnStack[], treeExpanded[], isSearch tabs
 */
export function serializeLayout(
  root: any,
  focusedPaneId: string,
): { layout: PersistedNode | null; focusPaneIndexPath: number[] } {
  function walk(node: any): PersistedNode | null {
    if (node.type === "pane") {
      // Filter out search tabs
      const tabEntries: PersistedTab[] = [];
      for (const t of node.tabs || []) {
        if (t.isSearch) continue;
        tabEntries.push({ path: t.path || "", title: t.title || "" });
      }
      if (tabEntries.length === 0) return null;
      const activeIdx = Math.max(
        0,
        tabEntries.findIndex(
          (_, i) =>
            node.tabs?.[i]?.id === node.activeTabId && !node.tabs[i]?.isSearch,
        ),
      );
      return {
        type: "pane",
        tabs: tabEntries,
        activeTabIndex: activeIdx,
      } as PersistedPane;
    }
    if (node.type === "split") {
      const children: PersistedNode[] = [];
      for (const c of node.children || []) {
        const serialized = walk(c);
        if (serialized) children.push(serialized);
      }
      if (children.length === 0) return null;
      if (children.length === 1) return children[0]; // flatten single-child split
      return {
        type: "split",
        direction: node.direction,
        children,
        sizes: node.sizes || [],
      } as PersistedSplit;
    }
    return null;
  }

  const layout = walk(root);
  const focusPaneIndexPath = layout
    ? findPaneIndexPath(layout, focusedPaneId, []) || [0]
    : [0];
  return { layout, focusPaneIndexPath };
}

// ── Counter helpers for ID generation (mirrors tabStore logic) ──
let _tc = 0;
let _nc = 0;

function _nid() {
  return `t_${++_tc}`;
}
function _npid() {
  return `n_${++_nc}`;
}

function _resetCounters() {
  _tc = 0;
  _nc = 0;
}

/**
 * Deserialize a persisted layout into runtime layout nodes with fresh IDs.
 */
export function deserializeLayout(persisted: PersistedNode): LayoutNode {
  _resetCounters();

  function walk(p: PersistedNode): LayoutNode {
    if (p.type === "pane") {
      const tabs: Tab[] = p.tabs.map((pt) => ({
        id: _nid(),
        path: pt.path,
        title: pt.title,
        files: [],
        selectedFiles: [],
      }));
      const activeIdx = Math.min(p.activeTabIndex, tabs.length - 1);
      return {
        type: "pane",
        id: _npid(),
        tabs,
        activeTabId: tabs[activeIdx]?.id || tabs[0]?.id || "",
      } as LayoutPane;
    }
    // split
    const children: LayoutNode[] = p.children.map(walk);
    return {
      type: "split",
      id: _npid(),
      direction: p.direction,
      children,
      sizes: p.sizes || [],
    } as LayoutSplit;
  }

  return walk(persisted);
}

/**
 * Resolve a pane node by index path in the persisted layout tree.
 */
export function resolvePaneByIndexPath(
  node: PersistedNode,
  indexPath: number[],
): PersistedPane | null {
  let current: PersistedNode = node;
  for (const idx of indexPath) {
    if (current.type === "split") {
      current = current.children[idx];
    } else {
      return current.type === "pane" ? current : null;
    }
  }
  return current.type === "pane" ? current : null;
}
