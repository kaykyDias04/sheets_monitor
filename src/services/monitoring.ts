import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { desktop } from "./platform";
import type { EngineEnvelope } from "@/types/integrity";
import type { Page } from "@/types/management";
import type {
  DesktopStatus,
  MonitoringStatus,
  NotificationRecord,
  NativeReadiness,
  UpdaterStatus,
} from "@/types/monitoring";

function command<T>(name: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!desktop) return Promise.reject({ code: "desktop_required" });
  return invoke<T>(name, args);
}
export const getMonitoringStatus = () =>
  command<EngineEnvelope<MonitoringStatus>>("monitoring_get_status");
export const setMonitoringEnabled = (enabled: boolean, expected_generation: number) =>
  command<EngineEnvelope<MonitoringStatus>>("monitoring_set_enabled", {
    enabled,
    expectedGeneration: expected_generation,
  });
export const checkNow = (expected_generation: number) =>
  command<EngineEnvelope<number>>("monitoring_check_now", {
    expectedGeneration: expected_generation,
  });
export const notificationHistory = (offset = 0) =>
  command<EngineEnvelope<Page<NotificationRecord>>>("monitoring_notification_history", {
    offset,
    limit: 25,
  });
export const notificationById = (id: string) =>
  command<EngineEnvelope<NotificationRecord | null>>("monitoring_notification", { id });
export const getDesktopStatus = () => command<DesktopStatus>("desktop_get_status");
export const retryBlockedNotifications = (expectedGeneration: number) =>
  command<EngineEnvelope<number>>("notification_retry_blocked", { expectedGeneration });
export const getNotificationReadiness = () =>
  command<NativeReadiness>("notification_get_readiness");
export const setStartup = (enabled: boolean) =>
  command<DesktopStatus>("desktop_set_startup", { enabled });
export const exitDesktop = () => command<void>("desktop_exit");
export const getUpdaterStatus = () => command<UpdaterStatus>("updater_get_status");
export const checkForUpdates = () => command<UpdaterStatus>("updater_check");
export const dismissUpdate = (version: string) =>
  command<UpdaterStatus>("updater_dismiss", { version });
export const applyUpdateAndRestart = () => command<void>("updater_apply_and_restart");
export const onMonitoringChanged = (callback: () => void) => listen("monitoring:changed", callback);
export const onDesktopNavigate = (callback: (target: string) => void) =>
  listen<string>("desktop:navigate", (event) => callback(event.payload));
export const onNotificationOpen = (callback: (id: string) => void) =>
  listen<string>("notification:open", (event) => callback(event.payload));
export const onUpdaterStateChanged = (callback: (status: UpdaterStatus) => void) =>
  listen<UpdaterStatus>("updater-state-changed", (event) => callback(event.payload));
