import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFileStore } from "@/stores/fileStore";
import type { ContextMenuOption } from "@/types";

export function useContextMenu() {
  const { t } = useI18n();
  const store = useFileStore();

  const showContextMenu = ref(false);
  const contextMenuPos = ref({ x: 0, y: 0 });
  const sidebarContextPath = ref("");

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

    if (!store.currentPath) {
      result.push({ label: t("contextMenu.open"), action: "open" });
      return result;
    }

    result.push(
      {
        label: t("contextMenu.newFolder"),
        action: "newFolder",
        shortcut: t("shortcuts.ctrlShiftN"),
      },
      {
        label: t("contextMenu.newFile"),
        action: "newFile",
        shortcut: t("shortcuts.ctrlN"),
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
          shortcut: t("shortcuts.ctrlX"),
        },
        {
          label: t("contextMenu.copy"),
          action: "copy",
          shortcut: t("shortcuts.ctrlC"),
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
        shortcut: t("shortcuts.ctrlV"),
      },
      { label: "", action: "", separator: true },
      {
        label: t("contextMenu.properties"),
        action: "properties",
      },
      { label: "", action: "", separator: true },
      {
        label: t("contextMenu.selectAll"),
        action: "selectAll",
        shortcut: t("shortcuts.ctrlA"),
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
    contextMenuItems: items,
    openContextMenu,
    closeContextMenu,
  };
}
