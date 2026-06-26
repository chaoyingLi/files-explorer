import { defineStore } from "pinia";
import { ref, triggerRef } from "vue";
import type { FileEntry } from "@/types";
import type { ColumnState } from "@/stores/viewStore";

export interface Tab {
  id: string;
  path: string;
  title: string;
  files: FileEntry[];
  selectedFiles: string[];
  treeExpanded?: string[];
  columnStack?: ColumnState[];
  isSearch?: boolean;
  searchQuery?: string;
  searchDone?: boolean;
  searchTotal?: number;
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

let tc = 0,
  nc = 0;
function nid() {
  return `t_${++tc}`;
}
function npid() {
  return `n_${++nc}`;
}
function cpane(path: string, title: string): LayoutPane {
  const ti = nid();
  return {
    type: "pane",
    id: npid(),
    tabs: [{ id: ti, path, title, files: [], selectedFiles: [] }],
    activeTabId: ti,
  };
}

export const useTabStore = defineStore("tab", () => {
  const root = ref<LayoutNode>(cpane("", ""));
  let lfp = "";

  function fpbytab(tid: string): LayoutPane | null {
    function s(n: LayoutNode): LayoutPane | null {
      if (n.type === "pane") {
        if (n.tabs.some((t) => t.id === tid)) return n;
      } else {
        for (const c of n.children) {
          const r = s(c);
          if (r) return r;
        }
      }
      return null;
    }
    return s(root.value);
  }

  function fpid(id: string): LayoutPane | null {
    function s(n: LayoutNode): LayoutPane | null {
      if (n.type === "pane" && n.id === id) return n;
      if (n.type === "split") {
        for (const c of n.children) {
          const r = s(c);
          if (r) return r;
        }
      }
      return null;
    }
    return s(root.value);
  }

  function allPanes(): LayoutPane[] {
    const ps: LayoutPane[] = [];
    function c(n: LayoutNode) {
      if (n.type === "pane") ps.push(n);
      else n.children.forEach(c);
    }
    c(root.value);
    return ps;
  }

  function fpane(): LayoutPane | null {
    const ps = allPanes();
    if (lfp) {
      const f = ps.find((p) => p.id === lfp);
      if (f) return f;
    }
    return ps[ps.length - 1] || null;
  }
  function ftab(): Tab | undefined {
    const p = fpane();
    return p?.tabs.find((t) => t.id === p.activeTabId);
  }
  function focus(p: string) {
    lfp = p;
  }

  function addTab(pid: string, path: string, title: string) {
    const p = fpid(pid);
    if (!p) return;
    const newId = nid();
    p.tabs.push({ id: newId, path, title, files: [], selectedFiles: [] });
    p.activeTabId = newId;
  }
  function closeTab(pid: string, tid: string) {
    const p = fpid(pid);
    if (!p || p.tabs.length <= 1) return;
    const i = p.tabs.findIndex((t) => t.id === tid);
    if (i < 0) return; // tab not found
    // Splice BEFORE computing target so indices are stable
    p.tabs.splice(i, 1);
    if (p.activeTabId === tid) {
      // Find the new active tab: prefer the one that was adjacent (same index if exists, else last)
      const newActive = p.tabs[Math.min(i, p.tabs.length - 1)];
      p.activeTabId = newActive?.id ?? p.tabs[p.tabs.length - 1]?.id ?? "";
    }
    triggerRef(root);
  }
  function switchTab(pid: string, tid: string) {
    const p = fpid(pid);
    if (!p) return;
    p.activeTabId = tid;
    focus(pid);
  }
  function updateTabPath(tid: string, path: string, title: string) {
    const p = fpbytab(tid);
    if (!p) return;
    const t = p.tabs.find((t2) => t2.id === tid);
    if (t) {
      t.path = path;
      t.title = title;
    }
  }

  function splitPane(
    pid: string,
    path: string,
    title: string,
    dir: "left" | "right" | "up" | "down",
  ) {
    const np = cpane(path, title);
    const sd = dir === "left" || dir === "right" ? "horizontal" : "vertical";
    function repl(
      n: LayoutNode,
      par: LayoutNode | null,
      pc: LayoutNode[] | null,
      idx: number,
    ): boolean {
      if (n.type === "pane" && n.id === pid) {
        const sp: LayoutSplit = {
          type: "split",
          id: npid(),
          direction: sd as "horizontal" | "vertical",
          children: dir === "left" || dir === "up" ? [np, n] : [n, np],
          sizes: [50, 50],
        };
        if (pc && idx >= 0) pc[idx] = sp;
        else root.value = sp;
        focus(np.id);
        return true;
      }
      if (n.type === "split") {
        for (let i = 0; i < n.children.length; i++) {
          if (repl(n.children[i], n, n.children, i)) return true;
        }
      }
      return false;
    }
    repl(root.value, null, null, -1);
    triggerRef(root);
  }

  function pnodeParent(id: string): LayoutSplit | null {
    function s(n: LayoutNode): LayoutSplit | null {
      if (n.type === "split") {
        for (const c of n.children) {
          if (c.id === id) return n;
          const r = s(c);
          if (r) return r;
        }
      }
      return null;
    }
    return s(root.value);
  }

  function closePane(pid: string) {
    function rm(
      n: LayoutNode,
      par: LayoutSplit | null,
      pc: LayoutNode[] | null,
      idx: number,
    ): boolean {
      if (n.type === "pane" && n.id === pid) {
        if (!par || !pc || idx < 0) {
          root.value = cpane("", "");
          return true;
        }
        pc.splice(idx, 1);
        if (pc.length === 1) {
          const gp = pnodeParent(par.id);
          if (gp) {
            const gi = gp.children.findIndex((c) => c.id === par.id);
            if (gi >= 0) gp.children[gi] = pc[0];
          } else root.value = pc[0];
        }
        return true;
      }
      if (n.type === "split") {
        for (let i = 0; i < n.children.length; i++) {
          if (rm(n.children[i], n, n.children, i)) return true;
        }
      }
      return false;
    }
    rm(root.value, null, null, -1);
    triggerRef(root);
  }

  const htid = ref<string | null>(null);
  let ht: ReturnType<typeof setTimeout> | null = null;

  // ── Drag-and-drop state (replaces window globals) ──
  const dragActive = ref(false);

  function endDrag() {
    dragActive.value = false;
  }

  function tdenter(tid: string) {
    htid.value = tid;
    if (ht) clearTimeout(ht);
    ht = setTimeout(() => {
      const p = fpbytab(tid);
      if (p) switchTab(p.id, tid);
    }, 500);
  }
  function tdleave(tid: string) {
    if (htid.value === tid) htid.value = null;
    if (ht) clearTimeout(ht);
  }

  function gff(): any[] {
    const t = ftab();
    return t?.files || [];
  }
  function sff(f: any[]) {
    const t = ftab();
    if (t) t.files = f;
  }
  function gfsf(): string[] {
    const t = ftab();
    return t?.selectedFiles || [];
  }
  function sfsf(p: string[]) {
    const t = ftab();
    if (t) t.selectedFiles = p;
  }
  function gfp(): string {
    const t = ftab();
    return t?.path || "";
  }

  function setRootLayout(node: LayoutNode) {
    root.value = node;
    triggerRef(root);
  }

  return {
    rootLayout: root,
    getAllPanes: allPanes,
    getFocusedPane: fpane,
    getFocusedTab: ftab,
    focusPane: focus,
    findPaneByTab: fpbytab,
    findPaneById: fpid,
    addTab,
    closeTab,
    switchTab,
    updateTabPath,
    splitPane,
    closePane,
    hoveredTabId: htid,
    onTabDragEnter: tdenter,
    onTabDragLeave: tdleave,
    getFocusedPath: gfp,
    dragActive,
    endDrag,
    setRootLayout,
  };
});
