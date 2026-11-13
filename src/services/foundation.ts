import { invoke, isTauri } from "@tauri-apps/api/core";
import { demoDashboard } from "@/fixtures";
import type { AppSettings, FoundationDashboard } from "@/types/foundation";
export const desktop = isTauri();
let browserDemo = demoDashboard();
export function resetBrowserDemo() {
  browserDemo = demoDashboard();
}
export async function getDashboard(): Promise<FoundationDashboard> {
  const result = desktop
    ? await invoke<FoundationDashboard>("foundation_get_dashboard")
    : structuredClone(browserDemo);
  if (
    result.is_demo !== true ||
    result.spreadsheets.some((s) => s.is_demo !== true) ||
    result.activities.some((e) => e.is_demo !== true)
  )
    throw new Error("Esta sprint aceita somente dados demonstrativos. A resposta foi recusada.");
  return result;
}
export async function saveSettings(settings: AppSettings): Promise<void> {
  if (
    !Number.isInteger(settings.interval_seconds) ||
    settings.interval_seconds < 60 ||
    settings.interval_seconds > 600
  )
    throw new Error("O intervalo deve ser um número inteiro entre 60 e 600 segundos.");
  if (settings.history_days !== 90 || settings.start_with_windows)
    throw new Error(
      "Nesta sprint, mantenha histórico de 90 dias e inicialização automática desativada.",
    );
  if (desktop) await invoke("foundation_save_settings", { settings });
  else browserDemo.settings = { ...settings };
}
