// ── 自动更新服务 ──
// ⚠️  本文件由 downloadSilently / installDownloadedUpdate 两分步函数组成
// ⛔ 禁止合并为一个 downloadAndInstall，会破坏静默下载→弹窗的 UX
// ⛔ 禁止修改函数签名，UpdaterChecker.vue 和 SettingsDialog.vue 依赖
//
// 封装 @tauri-apps/plugin-updater，提供状态管理和 Mock 测试支持

import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

// ============ 类型 ============

export type UpdateState =
  | "idle" | "checking" | "available"
  | "downloading" | "installing" | "ready_to_restart" | "error";

export type UpdateTaskState = Exclude<UpdateState, "available">;

export type UpdateErrorCode =
  | "CHECK_FAILED"
  | "NO_UPDATE"
  | "UPDATE_IN_PROGRESS"
  | "ALREADY_DOWNLOADED"
  | "INSTALL_FAILED";

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
  /** 下载进度百分比 (0-100)，仅 downloading 状态下有意义 */
  progress?: number;
}

export interface BackgroundInstallStartResult {
  started: boolean;
  snapshot: UpdateTaskSnapshot;
  /** 未启动的原因（仅在 started=false 时有意义） */
  reason?: string;
}

// ============ UpdateManager 类 ============

class UpdateManager {
  // ── Mock 状态 ──
  private mockEnabled = false;
  private mockScenario: "available" | "no_update" | "error" | "slow_download" = "available";

  // ── 运行状态 ──
  private downloadInFlight: Promise<UpdateResult> | null = null;
  private installInFlight: Promise<UpdateResult> | null = null;
  private cachedUpdate: Update | null = null;
  private taskSnapshot: UpdateTaskSnapshot = { state: "idle" };
  private listeners = new Set<(snapshot: UpdateTaskSnapshot) => void>();

  // ── Mock 控制 ──
  /** 启用 Mock，返回 cleanup 函数。Mock 执行一次后自动禁用 */
  enableMock(scenario: typeof this.mockScenario = "available"): () => void {
    this.mockEnabled = true;
    this.mockScenario = scenario;
    console.log("[Updater Mock] 已启用:", scenario);
    return () => {
      this.mockEnabled = false;
      console.log("[Updater Mock] 已自动禁用");
    };
  }

  disableMock() {
    this.mockEnabled = false;
    console.log("[Updater Mock] 已禁用");
  }

  isMockEnabled() {
    return { enabled: this.mockEnabled, scenario: this.mockScenario };
  }

  // ── 状态管理 ──
  private publishSnapshot(snapshot: UpdateTaskSnapshot): void {
    this.taskSnapshot = snapshot;
    this.listeners.forEach((fn) => fn(snapshot));
  }

  private setTaskState(
    state: UpdateTaskState,
    patch?: Partial<Pick<UpdateTaskSnapshot, "message" | "errorCode" | "progress">>,
  ): void {
    this.publishSnapshot({ state, ...patch });
  }

  getUpdateTaskSnapshot(): UpdateTaskSnapshot {
    return this.taskSnapshot;
  }

  subscribeUpdateTask(
    listener: (snapshot: UpdateTaskSnapshot) => void,
  ): () => void {
    this.listeners.add(listener);
    listener(this.taskSnapshot);
    return () => {
      this.listeners.delete(listener);
    };
  }

  // ── 检查更新 ──
  async checkForUpdates(): Promise<UpdateResult> {
    if (this.mockEnabled) {
      return this.mockCheckForUpdates();
    }

    try {
      this.setTaskState("checking");
      const update = await check();
      this.cachedUpdate = update?.available ? update : null;

      if (update?.available) {
        this.setTaskState("idle");
        return {
          state: "available",
          available: true,
          update: {
            version: update.version,
            body: update.body,
          },
        };
      }
      this.setTaskState("idle");
      return {
        state: "idle",
        available: false,
        errorCode: "NO_UPDATE",
        message: "当前已是最新版本",
      };
    } catch (error) {
      this.setTaskState("idle");
      return {
        state: "error",
        available: false,
        errorCode: "CHECK_FAILED",
        message: error instanceof Error ? error.message : String(error),
        error,
      };
    }
  }

  // ── 1. 静默下载（只下载，不安装）──
  downloadSilently(): BackgroundInstallStartResult {
    // P0: 防止 ready_to_restart 状态重复下载
    const current = this.getUpdateTaskSnapshot();
    if (current.state === "ready_to_restart") {
      return {
        started: false,
        snapshot: current,
        reason: "ALREADY_DOWNLOADED",
      };
    }
    if (this.downloadInFlight || this.installInFlight) {
      return {
        started: false,
        snapshot: current,
        reason: "UPDATE_IN_PROGRESS",
      };
    }
    if (!this.cachedUpdate?.available) {
      this.setTaskState("error", {
        message: "没有可用更新",
        errorCode: "NO_UPDATE",
      });
      return {
        started: false,
        snapshot: this.getUpdateTaskSnapshot(),
        reason: "NO_UPDATE",
      };
    }

    this.downloadInFlight = (async () => {
      try {
        if (this.mockEnabled) {
          this.setTaskState("downloading");
          await delay(2000);
          this.setTaskState("ready_to_restart", {
            message: "更新已下载，点击安装",
          });
          return {
            state: "ready_to_restart" as const,
            available: false,
            message: "Mock 下载完成",
          };
        }

        this.setTaskState("downloading", { message: "准备下载..." });
        // 重试最多 3 次
        const MAX_RETRIES = 3;
        for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
          try {
            // P0: 传入进度回调（累计已下载字节数）
            let downloadedBytes = 0;
            let totalBytes = 0;
            await this.cachedUpdate!.download((event) => {
              if (event.event === "Started") {
                totalBytes = event.data.contentLength ?? 0;
              } else if (event.event === "Progress") {
                downloadedBytes += event.data.chunkLength;
                const pct = totalBytes > 0
                  ? Math.round((downloadedBytes / totalBytes) * 100)
                  : 0;
                this.setTaskState("downloading", {
                  message: `下载中 ${Math.min(pct, 99)}%`,
                  progress: Math.min(pct, 99),
                });
              }
            });
            this.setTaskState("downloading", {
              message: "下载中 100%",
              progress: 100,
            });
            break;
          } catch (e) {
            if (attempt === MAX_RETRIES) throw e;
            this.setTaskState("downloading", {
              message: `下载失败，${attempt}/${MAX_RETRIES} 次重试...`,
            });
            await delay(2000);
          }
        }
        this.setTaskState("ready_to_restart", {
          message: "更新已下载，点击安装",
        });
        return {
          state: "ready_to_restart" as const,
          available: false,
          message: "下载完成",
        };
      } catch (error) {
        this.setTaskState("error", {
          message: error instanceof Error ? error.message : String(error),
          errorCode: "INSTALL_FAILED",
        });
        return {
          state: "error" as const,
          available: false,
          errorCode: "INSTALL_FAILED" as const,
        };
      } finally {
        this.downloadInFlight = null;
      }
    })();

    return { started: true, snapshot: this.getUpdateTaskSnapshot() };
  }

  // ── 2. 安装已下载的更新 ──
  installDownloadedUpdate(): BackgroundInstallStartResult {
    if (this.installInFlight) {
      return {
        started: false,
        snapshot: this.getUpdateTaskSnapshot(),
        reason: "INSTALL_IN_PROGRESS",
      };
    }
    if (!this.cachedUpdate?.available) {
      this.setTaskState("error", {
        message: "没有可用更新",
        errorCode: "NO_UPDATE",
      });
      return {
        started: false,
        snapshot: this.getUpdateTaskSnapshot(),
        reason: "NO_UPDATE",
      };
    }

    this.installInFlight = (async () => {
      try {
        if (this.mockEnabled) {
          this.setTaskState("installing");
          await delay(1000);
          this.setTaskState("ready_to_restart", {
            message: "Mock 已安装，重启以生效",
          });
          return {
            state: "ready_to_restart" as const,
            available: false,
            message: "Mock 已安装",
          };
        }

        this.setTaskState("installing", { message: "正在安装更新..." });
        await this.cachedUpdate!.install();
        this.setTaskState("ready_to_restart", {
          message: "更新已安装，重启以生效",
        });
        return {
          state: "ready_to_restart" as const,
          available: false,
          message: "已启动",
        };
      } catch (error) {
        this.setTaskState("error", {
          message: error instanceof Error ? error.message : String(error),
          errorCode: "INSTALL_FAILED",
        });
        return {
          state: "error" as const,
          available: false,
          errorCode: "INSTALL_FAILED" as const,
        };
      } finally {
        this.installInFlight = null;
      }
    })();

    return { started: true, snapshot: this.getUpdateTaskSnapshot() };
  }

  async relaunchAfterUpdate(): Promise<void> {
    await relaunch();
  }

  // 兼容：一键下载+安装
  readonly startBackgroundInstall = this.downloadSilently;

  // ============ Mock 实现 ============

  private async mockCheckForUpdates(): Promise<UpdateResult> {
    this.setTaskState("checking");
    // P1: 一次性 Mock — 自动禁用，防止干扰后续真实检查
    const wasEnabled = this.mockEnabled;
    this.mockEnabled = false;

    if (!wasEnabled) {
      return this.checkForUpdates();
    }

    await delay(800);

    switch (this.mockScenario) {
      case "available":
        this.setTaskState("idle");
        return {
          state: "available",
          available: true,
          update: {
            version: "9.9.9-test",
            body: "## 🎉 测试更新\n\n用于验证自动更新流程。",
          },
        };

      case "no_update":
        this.setTaskState("idle");
        return {
          state: "idle",
          available: false,
          errorCode: "NO_UPDATE",
          message: "当前已是最新版本",
        };

      case "error":
        this.setTaskState("idle");
        return {
          state: "error",
          available: false,
          errorCode: "CHECK_FAILED",
          message: "Mock: 网络错误，无法连接更新服务器",
        };

      case "slow_download":
        this.setTaskState("idle");
        return {
          state: "available",
          available: true,
          update: {
            version: "9.9.9-slow",
            body: "用于测试慢速下载",
          },
        };

      default:
        this.setTaskState("idle");
        return { state: "idle", available: false };
    }
  }
}

// ============ 全局单例 ============

const updateManager = new UpdateManager();

// ============ 兼容导出（保持原有函数签名） ============

export function enableMock(
  scenario?: Parameters<UpdateManager["enableMock"]>[0],
): ReturnType<UpdateManager["enableMock"]> {
  return updateManager.enableMock(scenario);
}
export function disableMock() {
  updateManager.disableMock();
}
export function isMockEnabled() {
  return updateManager.isMockEnabled();
}
export function getUpdateTaskSnapshot() {
  return updateManager.getUpdateTaskSnapshot();
}
export function subscribeUpdateTask(
  ...args: Parameters<UpdateManager["subscribeUpdateTask"]>
) {
  return updateManager.subscribeUpdateTask(...args);
}
export async function checkForUpdates() {
  return updateManager.checkForUpdates();
}
export function downloadSilently() {
  return updateManager.downloadSilently();
}
export function installDownloadedUpdate() {
  return updateManager.installDownloadedUpdate();
}
export async function relaunchAfterUpdate() {
  return updateManager.relaunchAfterUpdate();
}
export const startBackgroundInstall = downloadSilently;

// ============ 辅助函数 ============

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
