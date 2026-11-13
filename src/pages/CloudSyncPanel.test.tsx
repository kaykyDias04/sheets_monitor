import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CloudSyncPanel } from "./CloudSyncPanel";
import type { IntegrationState } from "@/types/google";
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), state: vi.fn(), connect: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => undefined) }));
vi.mock("@/services/platform", () => ({ desktop: true }));
vi.mock("@/services/google", async (original) => ({
  ...(await original<Record<string, unknown>>()),
  getIntegrationState: mocks.state,
  connectIntegration: mocks.connect,
}));
const session: IntegrationState = {
  generation: 7,
  configured: true,
  client_id: "fixture",
  drive_metadata_opt_in: false,
  auth: {
    status: "connected",
    account: { subject: "fixture-account", email: null, name: null },
    granted_scopes: ["https://www.googleapis.com/auth/drive.appdata"],
    expires_at: null,
  },
};
const conflict = {
  state: "conflict",
  message: "Versões conflitantes preservadas",
  conflicts: [{ commit_id: "a".repeat(64), created_at: "2026-11-01T12:00:00Z" }],
  last_synced_at: null,
  conflict_token: "fixture-conflict-token",
};
afterEach(cleanup);
beforeEach(() => {
  vi.clearAllMocks();
  mocks.state.mockResolvedValue(session);
  mocks.invoke.mockImplementation(async (command: string) =>
    command === "integration_get_features"
      ? [true, false]
      : { generation: 7, account_subject: "fixture-account", data: conflict },
  );
});
describe("Sincronização segura na interface", () => {
  it("preserva conflito até uma escolha explícita e mostra a recuperação", async () => {
    const changed = vi.fn();
    render(<CloudSyncPanel onChanged={changed} />);
    await screen.findByText("Versões conflitantes preservadas");
    expect(screen.getByText(/Nada será substituído automaticamente/)).toBeInTheDocument();
    expect(mocks.invoke.mock.calls.some(([name]) => name === "cloud_sync_now")).toBe(false);
    mocks.invoke.mockImplementation(async (command: string) =>
      command === "integration_get_features"
        ? [true, false]
        : {
            generation: 7,
            account_subject: "fixture-account",
            data: {
              state: "synced",
              message: "Configuração sincronizada.",
              conflicts: [],
              last_synced_at: null,
            },
          },
    );
    await userEvent.click(screen.getByRole("button", { name: "Manter configuração local" }));
    await screen.findByText("Configuração sincronizada.");
    expect(mocks.invoke).toHaveBeenCalledWith("cloud_sync_now", {
      expectedGeneration: 7,
      resolution: "keep_local",
      expectedConflictToken: conflict.conflict_token,
    });
    expect(
      screen.queryByRole("button", { name: "Manter configuração local" }),
    ).not.toBeInTheDocument();
    expect(changed).toHaveBeenCalledOnce();
  });
  it("descarta dados privados quando a conta troca durante a leitura", async () => {
    mocks.state.mockResolvedValueOnce(session).mockResolvedValue({
      ...session,
      generation: 8,
      auth: { ...session.auth, account: { subject: "other", name: null, email: null } },
    });
    render(<CloudSyncPanel onChanged={() => undefined} />);
    await waitFor(() => expect(mocks.state).toHaveBeenCalledTimes(2));
    expect(screen.queryByText("Versões conflitantes preservadas")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Conflitos de sincronização" }),
    ).not.toBeInTheDocument();
  });
  it("mantém escolhas locais após falha de rede, sem pedir novo consentimento", async () => {
    render(<CloudSyncPanel onChanged={() => undefined} />);
    await screen.findByText("Versões conflitantes preservadas");
    await userEvent.click(screen.getByLabelText(/Consultar auditoria Google Workspace/));
    mocks.invoke.mockRejectedValueOnce({ code: "cloud_network" });
    await userEvent.click(screen.getByRole("button", { name: "Sincronizar agora" }));
    await screen.findByRole("alert");
    expect(screen.getByLabelText(/Consultar auditoria Google Workspace/)).toBeChecked();
    expect(mocks.connect).not.toHaveBeenCalled();
  });
  it("envia a decisão remota vinculada ao conflito exibido", async () => {
    render(<CloudSyncPanel onChanged={() => undefined} />);
    await screen.findByText("Versões conflitantes preservadas");
    await userEvent.click(screen.getByRole("button", { name: /Usar versão remota de/ }));
    expect(mocks.invoke).toHaveBeenCalledWith("cloud_sync_now", {
      expectedGeneration: 7,
      resolution: `remote:${conflict.conflicts[0].commit_id}`,
      expectedConflictToken: conflict.conflict_token,
    });
  });
  it("exige atualizar conflitos persistidos sem token antes de decidir", async () => {
    mocks.invoke.mockImplementation(async (command: string) =>
      command === "integration_get_features"
        ? [true, false]
        : {
            generation: 7,
            account_subject: "fixture-account",
            data: { ...conflict, conflict_token: null },
          },
    );
    render(<CloudSyncPanel onChanged={() => undefined} />);
    await screen.findByText("Versões conflitantes preservadas");
    expect(screen.getByRole("button", { name: "Manter configuração local" })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Usar versão remota de/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Sincronizar agora" })).toBeEnabled();
  });
});
