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

export interface AvailableUpdateRef {
  version: string;
  body?: string;
  raw: Exclude<RawUpdate, null>;
}

type RawUpdate = Awaited<ReturnType<typeof check>>;

export interface UpdateResult {
  state: UpdateState;
  available: boolean;
  update?: AvailableUpdateRef;
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

let checkInFlight: Promise<UpdateResult> | null = null;
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

// ============ 检查更新 ============

export async function checkForUpdates(): Promise<UpdateResult> {
  if (checkInFlight) return checkInFlight;

  if (mockEnabled) {
    return mockCheckForUpdates();
  }

  checkInFlight = (async () => {
    try {
      setTaskState("checking");
      const update = await check();
      if (update?.available) {
        setTaskState("idle");
        return {
          state: "available",
          available: true,
          update: {
            version: update.version,
            body: update.body,
            raw: update,
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
    } finally {
      checkInFlight = null;
    }
  })();

  return checkInFlight;
}

// ============ 安装更新 ============

export function startBackgroundInstall(
  updateRef?: AvailableUpdateRef | null,
): BackgroundInstallStartResult {
  if (installInFlight) {
    return { started: false, snapshot: getUpdateTaskSnapshot() };
  }

  installInFlight = (async () => {
    try {
      let update = updateRef?.raw;
      if (!update?.available) {
        setTaskState("checking");
        const latest = await check();
        if (!latest?.available) {
          setTaskState("idle", {
            message: "当前已是最新版本",
            errorCode: "NO_UPDATE",
          });
          return {
            state: "idle",
            available: false,
            errorCode: "NO_UPDATE",
            message: "当前已是最新版本",
          };
        }
        update = latest;
      }

      setTaskState("downloading");
      setTaskState("installing");
      await update.downloadAndInstall();
      setTaskState("ready_to_restart", {
        message: "更新已安装，重启以生效",
      });

      return {
        state: "ready_to_restart",
        available: false,
        message: "更新已安装，正在重启...",
      };
    } catch (error) {
      setTaskState("error", {
        message: error instanceof Error ? error.message : String(error),
        errorCode: "INSTALL_FAILED",
      });
      return {
        state: "error",
        available: false,
        errorCode: "INSTALL_FAILED",
        message: error instanceof Error ? error.message : String(error),
        error,
      };
    } finally {
      installInFlight = null;
    }
  })();

  return { started: true, snapshot: getUpdateTaskSnapshot() };
}

export async function waitForInstallCompletion(): Promise<UpdateResult | null> {
  if (!installInFlight) return null;
  return installInFlight;
}

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
          raw: createMockUpdate(),
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
          raw: createMockUpdate({ slowMode: true }),
        },
      };

    default:
      setTaskState("idle");
      return { state: "idle", available: false };
  }
}

function createMockUpdate(options?: { slowMode?: boolean }): Update {
  const slowMode = options?.slowMode ?? false;
  return {
    available: true,
    version: slowMode ? "9.9.9-slow" : "9.9.9-test",
    date: new Date().toISOString(),
    body: slowMode ? "Slow download test" : "Mock update",
    downloadAndInstall: async (
      eventHandler?: (event: {
        event: string;
        data: { chunkLength: number };
      }) => void,
    ) => {
      setTaskState("downloading");
      const totalSteps = slowMode ? 10 : 3;
      const stepDelay = slowMode ? 2000 : 500;
      for (let i = 1; i <= totalSteps; i++) {
        await delay(stepDelay);
        if (eventHandler) {
          eventHandler({
            event: "Progress",
            data: { chunkLength: 1024 * 100 },
          });
        }
      }
      setTaskState("installing");
      await delay(slowMode ? 3000 : 500);
      setTaskState("ready_to_restart", {
        message: slowMode
          ? "慢速下载完成，重启以生效"
          : "Mock: 更新已安装，重启以生效",
      });
    },
  } as Update;
}
