import { beforeEach, afterEach, describe, it, expect, vi } from "vitest";

import { cleanup, render, screen, within } from "@testing-library/react";

import userEvent from "@testing-library/user-event";

import { invoke } from "@tauri-apps/api/core";

import App from "./App";

import { demoDashboard, type FoundationDashboard, type AppSettings } from "./fixtures";

import { getDashboard, saveSettings } from "./services/foundation";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true, invoke: vi.fn() }));

let persisted: FoundationDashboard;

const invokeMock = vi.mocked(invoke);

beforeEach(() => {
  persisted = demoDashboard();

  invokeMock.mockReset();

  invokeMock.mockImplementation(async (command, args) => {
    if (command === "foundation_get_dashboard") return structuredClone(persisted);

    if (command === "foundation_save_settings") {
      persisted.settings = { ...(args as { settings: AppSettings }).settings };

      return;
    }

    throw new Error(`Comando não permitido nesta sprint: ${command}`);
  });
});

afterEach(() => cleanup());

async function load() {
  render(<App />);

  await screen.findByRole("button", { name: "Ver exemplo de baseline de Indicadores comerciais" });

  return userEvent.setup();
}

describe("Sprint 01 · interface demonstrativa", () => {
  it("oferece as quatro telas demonstrativas e a nova integração e mantém identificação DEMO no desktop", async () => {
    const user = await load();

    const nav = screen.getByRole("navigation", { name: "Navegação principal" });

    expect(within(nav).getAllByRole("button")).toHaveLength(6);

    expect(screen.getByText(/Dados fictícios · Sprint 01/)).toBeInTheDocument();

    expect(screen.getByText(/Este workspace usa fixtures locais/)).toBeInTheDocument();

    for (const page of ["Planilhas", "Histórico", "Configurações", "Dashboard"]) {
      await user.click(within(nav).getByRole("button", { name: page }));

      expect(screen.getByRole("heading", { level: 1, name: page })).toBeInTheDocument();
    }
  });

  it("representa os quatro estados sem tratar alteração comum como falha", async () => {
    const user = await load();

    expect(screen.getAllByText("Online").length).toBeGreaterThan(0);

    expect(screen.getAllByText("Offline").length).toBeGreaterThan(0);

    expect(screen.getAllByText("Verificando").length).toBeGreaterThan(0);

    expect(screen.getAllByText("Indeterminado").length).toBeGreaterThan(0);

    const row = screen

      .getAllByText("Indicadores comerciais")

      .find((node) => node.closest("tr"))!
      .closest("tr")!;

    expect(within(row).getByText("Online")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Indicadores comerciais Conteúdo/ }));

    expect(screen.getByRole("dialog")).toHaveTextContent("integridade permanece Online");

    expect(screen.getByRole("dialog")).toHaveTextContent("Nenhuma alteração real foi observada");
  });

  it("mostra baseline apenas como exemplo, com diálogo acessível e sem aprovação real", async () => {
    const user = await load();

    const opener = screen.getByRole("button", {
      name: "Ver exemplo de baseline de Indicadores comerciais",
    });

    await user.click(opener);

    const dialog = screen.getByRole("dialog", { name: "Exemplo de baseline" });

    expect(dialog).toHaveTextContent("SEM APROVAÇÃO REAL");

    expect(dialog).toHaveTextContent("Sprint 03");

    expect(within(dialog).queryByRole("button", { name: /Aprovar/ })).not.toBeInTheDocument();

    await user.keyboard("{Escape}");

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    expect(opener).toHaveFocus();
  });

  it("salva preferências no bridge local e recarrega valor persistido", async () => {
    const user = await load();

    await user.click(screen.getByRole("button", { name: "Configurações" }));

    const interval = screen.getByLabelText("Intervalo de verificação planejado");

    expect(interval).toHaveValue(60);

    await user.clear(interval);

    await user.type(interval, "180");

    await user.click(screen.getByRole("button", { name: "Salvar preferências locais" }));

    await screen.findByText("Preferências demonstrativas salvas no SQLite local.");

    expect(invokeMock).toHaveBeenCalledWith("foundation_save_settings", {
      settings: { interval_seconds: 180, history_days: 90, start_with_windows: false },
    });

    expect((await getDashboard()).settings.interval_seconds).toBe(180);

    expect(screen.getByRole("switch")).toBeDisabled();

    expect(screen.getByText(/Configure a conta na tela/)).toBeInTheDocument();
  });

  it("filtra fixtures por nome e estado sem consulta externa", async () => {
    const user = await load();

    await user.click(screen.getByRole("button", { name: "Planilhas" }));

    await user.type(screen.getByRole("textbox", { name: "Buscar planilhas" }), "financeiro");

    expect(screen.getByText("Planejamento financeiro")).toBeInTheDocument();

    expect(screen.queryByText("Indicadores comerciais")).not.toBeInTheDocument();

    await user.selectOptions(screen.getByRole("combobox", { name: "Filtrar estado" }), "online");

    expect(screen.getByText("Nenhuma planilha nesta seleção")).toBeInTheDocument();
  });

  it("atualiza somente fixtures e não invoca OAuth, baseline ou monitoramento", async () => {
    const user = await load();

    await user.click(screen.getByRole("button", { name: "Atualizar demonstração" }));

    await screen.findByText("Demonstração atualizada. Nenhuma API Google foi consultada.");

    expect(invokeMock.mock.calls.every(([name]) => name === "foundation_get_dashboard")).toBe(true);

    await user.click(screen.getByRole("button", { name: "Sobre esta sprint" }));

    const dialog = screen.getByRole("dialog", { name: "Sprint 03 · Motor de integridade" });

    expect(dialog).toHaveTextContent("homologação Google real depende");

    await user.click(within(dialog).getByRole("button", { name: "Entendi" }));

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});

describe("contrato da fundação", () => {
  it("rejeita preferências fora dos limites antes de invocar o backend", async () => {
    await expect(
      saveSettings({ interval_seconds: 59, history_days: 90, start_with_windows: false }),
    ).rejects.toThrow("60 e 300");

    await expect(
      saveSettings({ interval_seconds: 301, history_days: 90, start_with_windows: false }),
    ).rejects.toThrow("60 e 300");

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
