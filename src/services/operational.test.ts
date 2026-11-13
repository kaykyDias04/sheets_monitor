import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { errorGuidance, extractErrorDiagnostic, integrationErrorMessage } from "./google";
import { savePreferences, managementError } from "./management";
import { saveSettings } from "./foundation";
import type { Preferences } from "@/types/management";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true, invoke: vi.fn() }));
const preferences: Preferences = {
  interval_seconds: 600,
  notifications_enabled: false,
  history_days: 90,
  start_with_windows: false,
  run_minimized: false,
  theme: "system",
};
beforeEach(() => vi.mocked(invoke).mockReset());

describe("intervalo operacional", () => {
  it("aceita 600 segundos nos clientes ativo e legado", async () => {
    await savePreferences(preferences, 1);
    expect(invoke).toHaveBeenCalledWith("save_management_preferences", {
      input: { preferences, expected_generation: 1 },
    });
    await saveSettings({ interval_seconds: 600, history_days: 90, start_with_windows: false });
    expect(invoke).toHaveBeenCalledWith("foundation_save_settings", {
      settings: { interval_seconds: 600, history_days: 90, start_with_windows: false },
    });
  });
  it("recusa limites inválidos antes do IPC", async () => {
    for (const interval_seconds of [59, 601, 60.5, Number.NaN]) {
      await expect(savePreferences({ ...preferences, interval_seconds }, 1)).rejects.toMatchObject({
        code: "invalid_preferences",
      });
    }
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("orientação de falhas", () => {
  it.each([
    ["auth_required", "auth"],
    ["network", "connectivity"],
    ["forbidden", "permission"],
    ["sync_failed", "sync"],
    ["rate_limited", "quota"],
    ["sync_busy", "sync"],
    ["cloud_network", "connectivity"],
    ["cloud_rate_limited", "quota"],
    ["cloud_server", "connectivity"],
    ["cloud_local_changed", "sync"],
    ["cloud_unauthorized", "auth"],
    ["cloud_forbidden", "permission"],
    ["cloud_opt_in_required", "permission"],
    ["cloud_storage", "sync"],
    ["cloud_stale_resolution", "sync"],
    ["cloud_invalid_data", "validation"],
    ["cloud_invalid_commit", "validation"],
    ["cloud_invalid_baseline", "validation"],
    ["cloud_hash_mismatch", "validation"],
    ["cloud_missing_parent", "validation"],
    ["cloud_invalid_graph", "validation"],
    ["cloud_listing_limit", "validation"],
    ["cloud_response_limit", "validation"],
    ["cloud_account", "validation"],
  ])("explica impacto e solução de %s", (code, category) => {
    expect(extractErrorDiagnostic({ code }).category).toBe(category);
    expect(errorGuidance({ code })).toMatchObject({ category });
    const text = integrationErrorMessage({ code });
    expect(text).toContain("Impacto:");
    expect(text).toContain("Solução:");
    expect(text).not.toContain("access_token");
  });
  it.each([
    "Sheets API / spreadsheets.get",
    "Drive API / files.list",
    "Apps Script API / projects.getContent",
    "Admin SDK Reports API / activities.list",
  ])("inclui diagnóstico sanitizado de quota de %s", (operation) => {
    const message = `${operation}: limite de leitura por minuto`;
    expect(integrationErrorMessage({ code: "rate_limited", message })).toContain(message);
    expect(extractErrorDiagnostic(`rate_limited|${message}`).category).toBe("quota");
    expect(
      integrationErrorMessage({ code: "rate_limited", message: "access_token=secret" }),
    ).not.toContain("secret");
  });
  it("preserva dados locais ao recusar versões remotas inválidas", () => {
    for (const code of [
      "cloud_invalid_data",
      "cloud_invalid_commit",
      "cloud_invalid_baseline",
      "cloud_hash_mismatch",
      "cloud_missing_parent",
      "cloud_invalid_graph",
      "cloud_listing_limit",
      "cloud_response_limit",
      "cloud_account",
    ]) {
      const text = integrationErrorMessage({ code });
      expect(text).toContain("configuração local é preservada");
      expect(text).not.toContain("conexão com a internet");
    }
    expect(integrationErrorMessage({ code: "cloud_opt_in_required" })).toContain(
      "Só habilite e reconecte se desejar",
    );
    expect(integrationErrorMessage({ code: "cloud_local_changed" })).toContain(
      "configuração local mudou",
    );
    expect(integrationErrorMessage({ code: "cloud_stale_resolution" })).toContain(
      "revise o conflito",
    );
    expect(integrationErrorMessage({ code: "cloud_storage" })).toContain("espaço em disco");
  });
  it("mantém solução de conflito de configuração sem substituir a causa", () => {
    expect(managementError({ code: "configuration_conflict" })).toContain("outra operação");
    expect(managementError({ code: "configuration_conflict" })).toContain("Impacto:");
  });
});
