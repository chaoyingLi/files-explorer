import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import type { ContextMenuOption } from "@/types";

export function useContextMenu() {
  const { t } = useI18n();
  const store = useFileStore();

  const isMac =
    typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

  const showContextMenu = ref(false);
  const contextMenuPos = ref({ x: 0, y: 0 });
  const sidebarContextPath = ref("");
  const rightClickedPath = ref("");

  function openContextMenu(x: number, y: number) {
    contextMenuPos.value = { x, y };
    showContextMenu.value = true;
  }

  function closeContextMenu() {
    showContextMenu.value = false;
  }

  const items = computed<ContextMenuOption[]>(() => {
    const hasSelection = store.selectedFiles.size > 0;
    const singleSelection = store.selectedFiles.size === 1;
    const result: ContextMenuOption[] = [];

    // Platform-aware shortcut key helper: use "cmd*" keys on macOS
    const sk = (ctrlKey: string, cmdKey: string) => t(isMac ? cmdKey : ctrlKey);

    if (!store.currentPath) {
      result.push({ label: t("contextMenu.open"), action: "open" });
      return result;
    }

    result.push(
      {
        label: t("contextMenu.newFolder"),
        action: "newFolder",
        shortcut: sk("shortcuts.ctrlShiftN", "shortcuts.cmdShiftN"),
      },
      {
        label: t("contextMenu.newFile"),
        action: "newFile",
        shortcut: sk("shortcuts.ctrlN", "shortcuts.cmdN"),
      },
      { label: "", action: "", separator: true },
    );

    if (hasSelection) {
      result.push(
        { label: t("contextMenu.open"), action: "open" },
        { label: "", action: "", separator: true },
        {
          label: t("contextMenu.cut"),
          action: "cut",
          shortcut: sk("shortcuts.ctrlX", "shortcuts.cmdX"),
        },
        {
          label: t("contextMenu.copy"),
          action: "copy",
          shortcut: sk("shortcuts.ctrlC", "shortcuts.cmdC"),
        },
        { label: "", action: "", separator: true },
        {
          label: t("contextMenu.delete"),
          action: "delete",
          shortcut: t("shortcuts.del"),
          children: [
            { label: t("dialogs.delete"), action: "delete" },
            { label: t("dialogs.deletePermanent"), action: "deletePermanent" },
          ],
        },
        { label: "", action: "", separator: true },
      );

      if (singleSelection) {
        result.push({
          label: t("contextMenu.rename"),
          action: "rename",
          shortcut: t("shortcuts.f2"),
        });
      }
    }

    if (store.currentPath) {
      result.push(
        { label: "", action: "", separator: true },
        { label: t("contextMenu.openInTerminal"), action: "openInTerminal" },
        {
          label: t("split.label"),
          action: "split",
          children: [
            { label: t("split.left"), action: "splitLeft" },
            { label: t("split.right"), action: "splitRight" },
            { label: t("split.up"), action: "splitUp" },
            { label: t("split.down"), action: "splitDown" },
          ],
        },
      );
    }

    result.push(
      {
        label: t("contextMenu.paste"),
        action: "paste",
        shortcut: sk("shortcuts.ctrlV", "shortcuts.cmdV"),
      },
      { label: "", action: "", separator: true },
      {
        label: t("contextMenu.properties"),
        action: "properties",
      },
      {
        label: isMac
          ? t("contextMenu.showInFinder")
          : t("contextMenu.showInExplorer"),
        action: "showInExplorer",
      },
      { label: "", action: "", separator: true },
      {
        label: t("contextMenu.selectAll"),
        action: "selectAll",
        shortcut: sk("shortcuts.ctrlA", "shortcuts.cmdA"),
      },
      { label: "", action: "", separator: true },
      {
        label: t("contextMenu.refresh"),
        action: "refresh",
        shortcut: t("shortcuts.f5"),
      },
    );

    return result;
  });

  return {
    showContextMenu,
    contextMenuPos,
    sidebarContextPath,
    rightClickedPath,
    contextMenuItems: items,
    openContextMenu,
    closeContextMenu,
  };
}
