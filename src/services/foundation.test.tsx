import { beforeEach, describe, it, expect, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { demoDashboard } from "@/fixtures";
import { getDashboard, saveSettings } from "./foundation";
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true, invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);
let persisted = demoDashboard();
beforeEach(() => {
  persisted = demoDashboard();
  invokeMock.mockReset();
});
describe("contrato da fundação", () => {
  it("rejeita preferências fora dos limites antes de invocar o backend", async () => {
    await expect(
      saveSettings({ interval_seconds: 59, history_days: 90, start_with_windows: false }),
    ).rejects.toThrow("60 e 600");

    await expect(
      saveSettings({ interval_seconds: 601, history_days: 90, start_with_windows: false }),
    ).rejects.toThrow("60 e 600");

    await expect(
      saveSettings({ interval_seconds: 60, history_days: 90, start_with_windows: true }),
    ).rejects.toThrow("inicialização automática desativada");

    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("recusa conteúdo sem marcação demonstrativa", async () => {
    invokeMock.mockResolvedValue({ ...persisted, is_demo: false });

    await expect(getDashboard()).rejects.toThrow("somente dados demonstrativos");
  });
});
