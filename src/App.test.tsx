import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { call, resetDemo } from "./bridge";
import type { Dashboard } from "./bridge";

beforeEach(() => resetDemo());
afterEach(() => cleanup());
async function load() {
  render(<App />);
  await screen.findByText("Planejamento financeiro");
  return userEvent.setup();
}

describe("workspace", () => {
  it("identifica dados fictícios e mantém baseline pendente até a confirmação", async () => {
    const user = await load();
    expect(screen.getByText(/Demonstração no navegador/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Revisar baseline/ }));
    const dialog = screen.getByRole("dialog", { name: "Revisar baseline" });
    await within(dialog).findByText(/Snapshot da coleta/);
    expect(
      within(dialog).getByRole("button", { name: "Aprovar baseline" }),
    ).toBeDisabled();
    expect(
      (await call<Dashboard>("get_dashboard")).sheets[2].baseline_approved,
    ).toBe(false);
    await user.click(
      within(dialog).getByRole("checkbox", { name: /Revisei esta coleta/ }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Aprovar baseline" }),
    );
    await screen.findByText(/Baseline aprovada/);
    expect(
      (await call<Dashboard>("get_dashboard")).sheets[2].baseline_approved,
    ).toBe(true);
  });
  it("cadastra por URL e deduplica IDs autorizados sem aprovar automaticamente", async () => {
    const user = await load();
    await user.click(
      screen.getByRole("button", { name: "Adicionar planilha" }),
    );
    await user.type(screen.getByLabelText("Nome da planilha"), "Nova base");
    await user.type(
      screen.getByLabelText("Link ou ID do Google Sheets"),
      "https://docs.google.com/spreadsheets/d/fixture-new/edit",
    );
    await user.type(
      screen.getByLabelText(/IDs de Apps Script autorizados/),
      "script-one,script-one,script-two",
    );
    await user.click(
      screen.getByRole("checkbox", { name: /Autorizo a consulta/ }),
    );
    await user.click(
      screen.getByRole("button", { name: "Cadastrar planilha" }),
    );
    await screen.findByText("Nova base");
    const sheet = (await call<Dashboard>("get_dashboard")).sheets.find(
      (s) => s.name === "Nova base",
    );
    expect(sheet).toMatchObject({
      spreadsheet_id: "fixture-new",
      baseline_approved: false,
      script_ids: ["script-one", "script-two"],
    });
  });
  it("filtra planilhas e abre comparação dos valores", async () => {
    const user = await load();
    await user.type(
      screen.getByRole("textbox", { name: "Buscar planilhas" }),
      "financeiro",
    );
    expect(screen.getByText("Planejamento financeiro")).toBeInTheDocument();
    expect(
      screen.queryByText("Operação · acompanhamento"),
    ).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Ver histórico" }));
    await user.click(
      screen.getByRole("button", {
        name: /Indicadores comerciais Conteúdo ou metadados/,
      }),
    );
    const dialog = screen.getByRole("dialog", {
      name: "Detalhes da alteração",
    });
    expect(within(dialog).getByText("12450")).toBeInTheDocument();
    expect(within(dialog).getByText("13820")).toBeInTheDocument();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
  it("demonstração não declara conexão Google ou entrega de notificações", async () => {
    const user = await load();
    await user.click(screen.getByRole("button", { name: "Configurações" }));
    await user.type(
      screen.getByLabelText("Client ID do Google Cloud"),
      "test.apps.googleusercontent.com",
    );
    await user.click(screen.getByRole("button", { name: "Conectar Google" }));
    await screen.findByRole("alert");
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Nenhuma conexão foi realizada",
    );
    expect((await call<Dashboard>("get_dashboard")).connected).toBe(false);
    await user.click(screen.getByRole("button", { name: /Notificações/ }));
    await user.click(screen.getByRole("button", { name: "Reprocessar fila" }));
    await screen.findByText(/fila de demonstração permanece pendente/);
    expect(
      (await call<Dashboard>("get_dashboard")).notifications.every(
        (n) => n.status === "pending",
      ),
    ).toBe(true);
  });
  it("exige confirmação por nome para remover acompanhamento", async () => {
    const user = await load();
    await user.click(
      screen.getByRole("button", { name: "Remover Planejamento financeiro" }),
    );
    const button = screen.getByRole("button", {
      name: "Remover acompanhamento",
    });
    expect(button).toBeDisabled();
    await user.type(
      screen.getByLabelText("Digite o nome da planilha para confirmar"),
      "Planejamento financeiro",
    );
    await user.click(button);
    await screen.findByText("Planilha removida do acompanhamento.");
    expect((await call<Dashboard>("get_dashboard")).sheets).toHaveLength(2);
  });
});
describe("bridge demonstrativo", () => {
  it("rejeita fingerprint inválido e preserva baseline", async () => {
    await call("preview_baseline", { id: "demo-3" });
    await expect(
      call("approve_baseline", { id: "demo-3", fingerprint: "invalid" }),
    ).rejects.toThrow("Prévia expirada");
    expect(
      (await call<Dashboard>("get_dashboard")).sheets[2].baseline_approved,
    ).toBe(false);
  });
  it("limita o workspace a 30 planilhas", async () => {
    for (let n = 3; n < 30; n++)
      await call("add_sheet", {
        input: { id: `fixture-${n}`, name: `Base ${n}`, script_ids: [] },
      });
    await expect(
      call("add_sheet", {
        input: { id: "fixture-31", name: "Excesso", script_ids: [] },
      }),
    ).rejects.toThrow("30 planilhas");
    expect((await call<Dashboard>("get_dashboard")).sheets).toHaveLength(30);
  });
  it("gera uma entrada de fila para cada diferença sem verificar pendentes", async () => {
    const before = await call<Dashboard>("get_dashboard");
    await call("run_check");
    const after = await call<Dashboard>("get_dashboard");
    expect(after.events.length - before.events.length).toBe(2);
    expect(after.notifications.length - before.notifications.length).toBe(2);
    expect(after.sheets[2].last_checked).toBeNull();
  });
});
