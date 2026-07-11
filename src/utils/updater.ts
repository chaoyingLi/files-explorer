// ── 自动更新服务 ──
// 封装 @tauri-apps/plugin-updater，提供状态管理和 Mock 测试支持

import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

// ============ 类型 ============

export type UpdateState =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "ready_to_restart"
  | "error";

export type UpdateTaskState =
  | "idle"
  | "checking"
  | "downloading"
  | "installing"
  | "ready_to_restart"
  | "error";

export type UpdateErrorCode =
  "CHECK_FAILED" | "NO_UPDATE" | "UPDATE_IN_PROGRESS" | "INSTALL_FAILED";

export interface AvailableUpdate {
  version: string;
  body?: string;
}

export interface UpdateResult {
  state: UpdateState;
  available: boolean;
  update?: AvailableUpdate;
  errorCode?: UpdateErrorCode;
  message?: string;
  error?: unknown;
}

export interface UpdateTaskSnapshot {
  state: UpdateTaskState;
  message?: string;
  errorCode?: UpdateErrorCode;
}

export interface BackgroundInstallStartResult {
  started: boolean;
  snapshot: UpdateTaskSnapshot;
}

// ============ Mock（开发测试用） ============

let mockEnabled = false;
let mockScenario: "available" | "no_update" | "error" | "slow_download" =
  "available";

export function enableMock(scenario: typeof mockScenario = "available") {
  mockEnabled = true;
  mockScenario = scenario;
  console.log("[Updater Mock] 已启用:", scenario);
}

export function disableMock() {
  mockEnabled = false;
  console.log("[Updater Mock] 已禁用");
}

export function isMockEnabled() {
  return { enabled: mockEnabled, scenario: mockScenario };
}

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ============ 状态管理 ============

let installInFlight: Promise<UpdateResult> | null = null;
let updateTaskSnapshot: UpdateTaskSnapshot = { state: "idle" };
const updateTaskListeners = new Set<(snapshot: UpdateTaskSnapshot) => void>();

function publishSnapshot(snapshot: UpdateTaskSnapshot): void {
  updateTaskSnapshot = snapshot;
  updateTaskListeners.forEach((fn) => fn(snapshot));
}

function setTaskState(
  state: UpdateTaskState,
  patch?: Pick<UpdateTaskSnapshot, "message" | "errorCode">,
): void {
  publishSnapshot({
    state,
    message: patch?.message,
    errorCode: patch?.errorCode,
  });
}

export function getUpdateTaskSnapshot(): UpdateTaskSnapshot {
  return updateTaskSnapshot;
}

export function subscribeUpdateTask(
  listener: (snapshot: UpdateTaskSnapshot) => void,
): () => void {
  updateTaskListeners.add(listener);
  listener(updateTaskSnapshot);
  return () => {
    updateTaskListeners.delete(listener);
  };
}

// ============ 缓存的 Update 对象（避免重复 check()） ============

let cachedUpdate: Update | null = null;

// ============ 检查更新 ============

export async function checkForUpdates(): Promise<UpdateResult> {
  if (mockEnabled) {
    return mockCheckForUpdates();
  }

  try {
    setTaskState("checking");
    const update = await check();
    cachedUpdate = update?.available ? update : null;

    if (update?.available) {
      setTaskState("idle");
      return {
        state: "available",
        available: true,
        update: {
          version: update.version,
          body: update.body,
        },
      };
    }
    setTaskState("idle");
    return {
      state: "idle",
      available: false,
      errorCode: "NO_UPDATE",
      message: "当前已是最新版本",
    };
  } catch (error) {
    setTaskState("idle");
    return {
      state: "error",
      available: false,
      errorCode: "CHECK_FAILED",
      message: error instanceof Error ? error.message : String(error),
      error,
    };
  }
}

// ============ 静默下载+安装（后台完成，完成直接能重启） ============

let updateInFlight: Promise<UpdateResult> | null = null;

export function startSilentUpdate(): BackgroundInstallStartResult {
  if (updateInFlight) {
    return { started: false, snapshot: getUpdateTaskSnapshot() };
  }
  if (!cachedUpdate?.available) {
    setTaskState("error", { message: "没有可用更新", errorCode: "NO_UPDATE" });
    return { started: false, snapshot: getUpdateTaskSnapshot() };
  }

  updateInFlight = (async () => {
    try {
      if (mockEnabled) {
        setTaskState("downloading");
        await delay(1500);
        setTaskState("installing");
        await delay(1000);
        setTaskState("ready_to_restart", { message: "Mock 更新已安装，重启以生效" });
        return { state: "ready_to_restart" as const, available: false, message: "Mock" };
      }

      setTaskState("downloading");
      setTaskState("installing");
      await cachedUpdate!.downloadAndInstall();
      setTaskState("ready_to_restart", { message: "更新已安装，重启以生效" });
      return { state: "ready_to_restart" as const, available: false, message: "完成" };
    } catch (error) {
      setTaskState("error", {
        message: error instanceof Error ? error.message : String(error),
        errorCode: "INSTALL_FAILED",
      });
      return { state: "error" as const, available: false, errorCode: "INSTALL_FAILED" as const };
    } finally {
      updateInFlight = null;
    }
  })();

  return { started: true, snapshot: getUpdateTaskSnapshot() };
}

// 兼容旧 API

export const startBackgroundInstall = startSilentUpdate;
export const downloadSilently = startSilentUpdate;
export const installDownloadedUpdate = startSilentUpdate;

export async function relaunchAfterUpdate(): Promise<void> {
  await relaunch();
}

// ============ Mock 实现 ============

async function mockCheckForUpdates(): Promise<UpdateResult> {
  setTaskState("checking");
  await delay(800);

  switch (mockScenario) {
    case "available":
      setTaskState("idle");
      return {
        state: "available",
        available: true,
        update: {
          version: "9.9.9-test",
          body: "## 🎉 测试更新\n\n用于验证自动更新流程。",
        },
      };

    case "no_update":
      setTaskState("idle");
      return {
        state: "idle",
        available: false,
        errorCode: "NO_UPDATE",
        message: "当前已是最新版本",
      };

    case "error":
      setTaskState("idle");
      return {
        state: "error",
        available: false,
        errorCode: "CHECK_FAILED",
        message: "Mock: 网络错误，无法连接更新服务器",
      };

    case "slow_download":
      setTaskState("idle");
      return {
        state: "available",
        available: true,
        update: {
          version: "9.9.9-slow",
          body: "用于测试慢速下载",
        },
      };

    default:
      setTaskState("idle");
      return { state: "idle", available: false };
  }
}
