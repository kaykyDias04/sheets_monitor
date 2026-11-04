import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { scopeForTabs } from "./pages/ScopeEditor";
import type { IntegrationState } from "./types/google";
import type { RegisteredSpreadsheet, Preferences } from "./types/management";
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true, invoke: vi.fn() }));
const signals = vi.hoisted(() => ({ changed: null as (() => void) | null }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, callback: () => void) => {
    if (name === "monitoring:changed") signals.changed = callback;
    return () => {
      if (signals.changed === callback) signals.changed = null;
    };
  }),
}));
const call = vi.mocked(invoke);
let session: IntegrationState;
let rows: RegisteredSpreadsheet[];
let preferences: Preferences;
let readHook: ((command: string) => void) | null;
const tabs = [
  { sheet_id: 17, title: "Receitas", index: 0, hidden: false, row_count: 500, column_count: 8 },
];
function fixtureRow(): RegisteredSpreadsheet {
  return {
    spreadsheet_id: "fixture-file",
    name: "Receitas de teste",
    url: "https://docs.google.com/spreadsheets/d/fixture-file/edit",
    account_email: "fixture@example.invalid",
    paused: false,
    removed: false,
    status: "configuration_pending",
    authorization: "available",
    access_error: null,
    last_checked_at: null,
    baseline_version: null,
    scope: null,
    scope_hash: null,
    revision: 1,
    tabs,
    selected_sheet_ids: [],
    tabs_refreshed_at: null,
  };
}
function envelope<T>(data: T) {
  return {
    generation: session.generation,
    account_subject: session.auth.account?.subject ?? "",
    data: structuredClone(data),
  };
}
beforeEach(() => {
  session = {
    generation: 1,
    configured: true,
    client_id: "fixture-client",
    drive_metadata_opt_in: true,
    auth: {
      status: "connected",
      account: {
        subject: "fixture-account",
        email: "fixture@example.invalid",
        name: "Conta de teste",
      },
      granted_scopes: [
        "https://www.googleapis.com/auth/spreadsheets.readonly",
        "https://www.googleapis.com/auth/drive.metadata.readonly",
      ],
      expires_at: null,
    },
  };
  rows = [];
  signals.changed = null;
  readHook = null;
  preferences = {
    interval_seconds: 60,
    notifications_enabled: false,
    history_days: 90,
    start_with_windows: false,
    run_minimized: false,
    theme: "light",
  };
  call.mockReset();
  call.mockImplementation(async (command, args) => {
    readHook?.(command);
    const input = (args as { input?: Record<string, unknown> } | undefined)?.input ?? {};
    if (command === "integration_get_state") return structuredClone(session);
    if (command === "notification_get_readiness")
      return { identity_mode: "development_powershell", setting: "enabled", error_code: null };
    if (command === "desktop_get_status")
      return {
        tray_available: true,
        notification_capability: "desktop_unverified",
        startup: { registered: false, managed: false, os_execution: "unknown", error_code: null },
        exiting: false,
      };
    if (command === "monitoring_get_status")
      return envelope({
        control: { enabled: false, generation: 0 },
        schedules: [],
        eligible_count: 0,
        interval_seconds: preferences.interval_seconds,
        service: { running: true, active: 0, queued: 0, warning: null },
      });
    if (command === "monitoring_notification_history")
      return envelope({ items: [], total: 0, offset: 0, limit: 25 });
    if (command === "get_dashboard_summary")
      return envelope({
        registered: rows.length,
        online: rows.filter((r) => r.status === "online").length,
        offline: 0,
        indeterminate: 0,
        paused: 0,
        pending: rows.filter((r) => r.status === "configuration_pending").length,
        not_checked: 0,
        spreadsheets: rows,
        recent_events: [],
        recent_failures: [],
      });
    if (command === "get_management_preferences") return envelope(preferences);
    if (command === "monitoring_check_file") return envelope(1);
    if (command === "save_management_preferences") {
      preferences = structuredClone(input.preferences as Preferences);
      return envelope(preferences);
    }
    if (command === "discover_spreadsheets")
      return envelope({
        files: [
          {
            id: input.page_token ? "fixture-second" : "fixture-file",
            name: input.page_token ? "Segunda planilha" : "Arquivo descoberto",
            mime_type: "application/vnd.google-apps.spreadsheet",
            modified_time: null,
            shared: true,
            drive_id: null,
            resource_key: null,
          },
        ],
        next_page_token: input.page_token ? null : "opaque-next",
        incomplete_search: false,
        complete: !!input.page_token,
        corpus: "allDrives",
      });
    if (command === "register_spreadsheet") {
      if (!String(input.url).startsWith("https://docs.google.com/spreadsheets/d/"))
        throw { code: "invalid_input" };
      if (rows.some((r) => String(input.url).includes(r.spreadsheet_id)))
        throw { code: "duplicate" };
      const row = fixtureRow();
      rows.push(row);
      return envelope(row);
    }
    if (command === "get_spreadsheet_diagnostics")
      return envelope({
        spreadsheet: rows.find((r) => r.spreadsheet_id === input.spreadsheet_id),
        report: null,
        baseline: null,
        scripts: [],
      });
    if (command === "list_baseline_versions")
      return envelope({ items: [], offset: Number(input.offset ?? 0), limit: 10, total: 0 });
    if (command === "update_monitored_tabs") {
      rows[0].selected_sheet_ids = input.sheet_ids as number[];
      rows[0].revision++;
      return envelope(rows[0]);
    }
    if (command === "update_integrity_rules") {
      if (input.expected_revision !== rows[0].revision) throw { code: "configuration_conflict" };
      rows[0].scope = structuredClone(input.scope as RegisteredSpreadsheet["scope"]);
      rows[0].scope_hash = "fixture-scope";
      rows[0].revision++;
      return envelope(rows[0]);
    }
    if (command === "propose_integrity_rules") {
      const scope = scopeForTabs(null, [17]);
      scope.sheets[0].regions[0].columns[0].header = "Código";
      return envelope(scope);
    }
    if (command === "preview_baseline") {
      if (input.expected_revision !== rows[0].revision) throw { code: "configuration_conflict" };
      return envelope({
        revision: rows[0].revision,
        preview: {
          session_generation: session.generation,
          snapshot_fingerprint: "pinned-fingerprint",
          active_baseline: null,
          snapshot: {
            schema_version: 1,
            id: "pinned-snapshot",
            account_subject: "fixture-account",
            spreadsheet_id: "fixture-file",
            capture_started_at: "2026-11-02T12:00:00Z",
            captured_at: "2026-11-02T12:00:00Z",
            scope_hash: "fixture-scope",
            scope: rows[0].scope,
            sheets: [
              {
                sheet_id: 17,
                title: "Receitas",
                index: 0,
                row_count: 500,
                column_count: 8,
                regions: [],
              },
            ],
            scripts: [],
            connectivity: "available",
            coverage: {
              complete: true,
              metadata_complete: true,
              atomic_across_services: false,
              issues: [],
            },
          },
        },
      });
    }
    if (command === "approve_baseline") {
      rows[0].baseline_version = 1;
      rows[0].status = "not_checked";
      return envelope({});
    }
    if (command === "remove_spreadsheet") {
      rows = rows.filter((r) => r.spreadsheet_id !== input.spreadsheet_id);
      return envelope(null);
    }
    if (command === "list_change_events")
      return envelope({ items: [], total: 0, offset: Number(input.offset ?? 0), limit: 25 });
    if (command === "integration_disconnect") {
      session = {
        ...session,
        generation: 2,
        auth: { ...session.auth, status: "disconnected", account: null, granted_scopes: [] },
      };
      return structuredClone(session);
    }
    throw { code: "invalid_input" };
  });
});
afterEach(cleanup);
async function settled() {
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Atualizar dados" })).toBeEnabled(),
  );
}
async function load() {
  render(<App />);
  await settled();
  return userEvent.setup();
}
async function go(user: ReturnType<typeof userEvent.setup>, label: string) {
  await user.click(
    within(screen.getByRole("navigation", { name: "Navegação principal" })).getByRole("button", {
      name: new RegExp(`^${label}(?:\\s+\\d+)?$`),
    }),
  );
  await settled();
}
async function openFirst(user: ReturnType<typeof userEvent.setup>) {
  await go(user, "Planilhas");
  await user.click(screen.getByRole("button", { name: "Abrir detalhes de Receitas de teste" }));
  await screen.findByRole("heading", { name: "Receitas de teste" });
  await settled();
}
describe("Sprint 04 · operação sem demonstração", () => {
  it("carrega cinco destinos na barra lateral sem item separado de regras e baselines", async () => {
    await load();
    const nav = within(screen.getByRole("navigation", { name: "Navegação principal" }));
    expect(nav.getAllByRole("button")).toHaveLength(5);
    expect(nav.queryByRole("button", { name: /Regras e baselines/i })).not.toBeInTheDocument();
    expect(screen.getByText("Encontre suas primeiras planilhas")).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name.startsWith("foundation_"))).toBe(false);
  });
  it("mostra login sem pedir credenciais Cloud ao usuário comum", async () => {
    session.auth.account = null;
    session.auth.status = "disconnected";
    await load();
    expect(screen.getByRole("button", { name: "Entrar com Google" })).toBeEnabled();
    expect(screen.queryByLabelText(/Client ID/)).not.toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "get_dashboard_summary")).toBe(false);
  });
  it("descobre e pagina arquivos compartilhados sem cadastro automático", async () => {
    const user = await load();
    await go(user, "Planilhas");
    await user.click(screen.getByRole("button", { name: "Buscar planilhas" }));
    await screen.findByText("Arquivo descoberto");
    await settled();
    expect(screen.getByText("Compartilhada")).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "register_spreadsheet")).toBe(false);
    await user.click(screen.getByRole("button", { name: "Próxima página" }));
    await screen.findByText("Segunda planilha");
    expect(call).toHaveBeenCalledWith("discover_spreadsheets", {
      input: expect.objectContaining({ page_token: "opaque-next" }),
    });
  });
  it("cadastra link válido como configuração pendente", async () => {
    const user = await load();
    await go(user, "Planilhas");
    await user.click(screen.getByLabelText("Link da planilha"));
    await user.paste("https://docs.google.com/spreadsheets/d/fixture-file/edit");
    await user.click(screen.getByRole("button", { name: "Cadastrar por link" }));
    await screen.findByRole("heading", { name: "Receitas de teste" });
    await settled();
    expect(screen.getByText("Configuração pendente")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Verificar agora" })).toBeDisabled();
    expect(call).toHaveBeenCalledWith("register_spreadsheet", {
      input: {
        url: "https://docs.google.com/spreadsheets/d/fixture-file/edit",
        expected_generation: 1,
      },
    });
  });
  it("trata URL inválida e cadastro duplicado", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await go(user, "Planilhas");
    const input = screen.getByLabelText("Link da planilha");
    await user.click(input);
    await user.paste("https://example.invalid/file");
    await user.click(screen.getByRole("button", { name: "Cadastrar por link" }));
    await screen.findByRole("alert");
    await settled();
    expect(rows).toHaveLength(1);
    await user.clear(input);
    await user.click(input);
    await user.paste("https://docs.google.com/spreadsheets/d/fixture-file/edit");
    await user.click(screen.getByRole("button", { name: "Cadastrar por link" }));
    await screen.findByText("Esta planilha já está cadastrada nesta conta.");
  });
  it("desabilita cadastro ao atingir 30 sem limitar descoberta", async () => {
    rows = Array.from({ length: 30 }, (_, i) => ({
      ...fixtureRow(),
      spreadsheet_id: `fixture${i}`,
      name: `Planilha ${i}`,
    }));
    const user = await load();
    await go(user, "Planilhas");
    expect(screen.getByRole("button", { name: "Cadastrar por link" })).toBeDisabled();
    expect(screen.getByText(/limite de 30 se aplica aos cadastros/)).toBeInTheDocument();
  });
  it("salva abas, sugere rascunho e aprova apenas a captura revisada", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    expect(rows[0].selected_sheet_ids).toEqual([17]);
    await user.click(screen.getByRole("button", { name: "Regras de proteção" }));
    await user.click(screen.getByRole("button", { name: "Sugerir regras para abas selecionadas" }));
    await settled();
    expect(screen.getByRole("textbox", { name: /Cabeçalho protegido/ })).toHaveValue("Código");
    expect(rows[0].scope).toBeNull();
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    await user.click(screen.getByRole("button", { name: "Referências" }));
    await user.click(screen.getByRole("button", { name: "Capturar estrutura para revisão" }));
    await screen.findByText("Revise antes de aprovar");
    await settled();
    expect(screen.getByRole("button", { name: "Aprovar referência versão 1" })).toBeDisabled();
    await user.click(screen.getByRole("checkbox", { name: /Revisei o escopo/ }));
    await user.click(screen.getByRole("button", { name: "Aprovar referência versão 1" }));
    await settled();
    expect(call).toHaveBeenCalledWith("approve_baseline", {
      input: {
        spreadsheet_id: "fixture-file",
        expected_generation: 1,
        expected_revision: 3,
        snapshot_id: "pinned-snapshot",
        snapshot_fingerprint: "pinned-fingerprint",
        expected_version: 0,
        confirmed: true,
      },
    });
  }, 15000);
  it("explica no próprio formulário por que regras incompletas não foram salvas", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    await user.click(screen.getByRole("button", { name: "Regras de proteção" }));
    // A regra inicial tem cabeçalho vazio: o usuário vê o botão ativo, mas sem texto digitado.
    const save = screen.getByRole("button", { name: "Salvar regras de proteção" });
    await user.click(save);
    const problem = await screen.findByText(/informe o cabeçalho protegido/);
    expect(problem).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "update_integrity_rules")).toBe(false);
    expect(save.closest("form")).toContainElement(problem);
    // Corrigido o campo, o salvamento é enviado ao backend e o aviso desaparece.
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    await user.click(save);
    await settled();
    expect(call.mock.calls.some(([name]) => name === "update_integrity_rules")).toBe(true);
    expect(rows[0].scope?.sheets[0].regions[0].columns[0].header).toBe("Código");
    expect(screen.queryByText(/informe o cabeçalho protegido/)).not.toBeInTheDocument();
  }, 15000);
  it("mantém a seção de regras aberta e mostra a confirmação após salvar", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    await user.click(screen.getByRole("button", { name: "Regras de proteção" }));
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(await screen.findByText(/Regras salvas\./)).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: /Cabeçalho protegido/ })).toHaveValue("Código");
    expect(screen.getByRole("button", { name: "Salvar regras de proteção" })).toBeDisabled();
  }, 15000);
  it("recarrega a planilha após salvar mesmo quando o monitoramento atualiza em segundo plano", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    await user.click(screen.getByRole("button", { name: "Regras de proteção" }));
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    // O serviço Rust avisa uma mudança enquanto a gravação ainda está em andamento.
    readHook = (command) => {
      if (command !== "update_integrity_rules") return;
      readHook = null;
      act(() => signals.changed?.());
    };
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(await screen.findByText(/Regras salvas\./)).toBeInTheDocument();
    // A tela precisa refletir a revisão gravada: nada de formulário "sujo" nem botão repetível.
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Salvar regras de proteção" })).toBeDisabled(),
    );
    // Uma nova edição usa a revisão atual e é aceita, sem sair e voltar para a planilha.
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), " 2");
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(rows[0].scope?.sheets[0].regions[0].columns[0].header).toBe("Código 2");
    // Capturar usa a mesma revisão fresca e também não é rejeitado em silêncio.
    await user.click(screen.getByRole("button", { name: "Referências" }));
    await user.click(screen.getByRole("button", { name: "Capturar estrutura para revisão" }));
    await screen.findByText("Revise antes de aprovar");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  }, 20000);
  it("recupera a tela sozinha quando a revisão local ficou desatualizada", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    await user.click(screen.getByRole("button", { name: "Regras de proteção" }));
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    rows[0].revision++; // outra operação alterou o cadastro depois que a tela foi carregada
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(await screen.findByText(/dados da planilha foram atualizados/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Regras de proteção" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(await screen.findByText(/Regras salvas\./)).toBeInTheDocument();
    expect(rows[0].scope?.sheets[0].regions[0].columns[0].header).toBe("Código");
  }, 20000);
  it("exige confirmação de remoção e escolha de histórico", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("button", { name: "Remover cadastro" }));
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByRole("button", { name: "Confirmar remoção" })).toBeDisabled();
    await user.selectOptions(
      within(dialog).getByLabelText("Histórico de atividades"),
      "delete_activity",
    );
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Confirmo a remoção deste cadastro." }),
    );
    await user.click(within(dialog).getByRole("button", { name: "Confirmar remoção" }));
    await screen.findByText("Cadastro removido localmente. O arquivo Google permanece intacto.");
    expect(call).toHaveBeenCalledWith("remove_spreadsheet", {
      input: {
        spreadsheet_id: "fixture-file",
        expected_generation: 1,
        expected_revision: 1,
        confirmed: true,
        history: "delete_activity",
      },
    });
  });
  it("envia a verificação individual para a fila sem declarar coleta concluída", async () => {
    rows.push({
      ...fixtureRow(),
      status: "not_checked",
      baseline_version: 1,
      selected_sheet_ids: [17],
      scope: scopeForTabs(null, [17]),
      scope_hash: "fixture-scope",
    });
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("button", { name: "Verificar agora" }));
    await screen.findByText(
      "Verificação adicionada à fila. O resultado será atualizado pelo serviço.",
    );
    expect(call).toHaveBeenCalledWith("monitoring_check_file", {
      input: { spreadsheet_id: "fixture-file", expected_generation: 1, expected_revision: 1 },
    });
    expect(call).not.toHaveBeenCalledWith("run_spreadsheet_check", expect.anything());
  });
  it("envia filtros do histórico ao backend com paginação", async () => {
    const user = await load();
    await go(user, "Atividades");
    await user.selectOptions(screen.getByLabelText("Severidade"), "critical");
    await user.selectOptions(screen.getByLabelText("Tipo de alteração"), "violation");
    await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
    await settled();
    expect(call).toHaveBeenCalledWith("list_change_events", {
      input: expect.objectContaining({
        severity: "critical",
        kind: "violation",
        offset: 0,
        limit: 25,
      }),
    });
  });
  it("persiste preferências por conta sem alterar o registro Windows implicitamente", async () => {
    const user = await load();
    await go(user, "Configurações");
    const interval = screen.getByLabelText(/Intervalo de verifica/);
    await user.clear(interval);
    await user.type(interval, "180");
    await user.click(
      screen.getByRole("checkbox", { name: "Enviar notificações de alterações e falhas" }),
    );
    await user.click(screen.getByRole("button", { name: "Salvar preferências" }));
    await settled();
    expect(preferences.interval_seconds).toBe(180);
    expect(preferences.notifications_enabled).toBe(true);
    expect(
      screen.getByRole("checkbox", { name: /Registrar início com Windows/ }),
    ).not.toBeChecked();
    expect(call).not.toHaveBeenCalledWith("desktop_set_startup", expect.anything());
    cleanup();
    render(<App />);
    await settled();
    await go(user, "Configurações");
    expect(screen.getByLabelText(/Intervalo de verifica/)).toHaveValue(180);
  });
  it("descarta resultado se a conta muda mesmo com envelope posterior válido", async () => {
    const user = await load();
    await go(user, "Planilhas");
    readHook = (command) => {
      if (command === "discover_spreadsheets")
        session = {
          ...session,
          generation: 2,
          auth: { ...session.auth, account: { subject: "other-account", email: null, name: null } },
        };
    };
    await user.click(screen.getByRole("button", { name: "Buscar planilhas" }));
    await screen.findByRole("alert");
    expect(screen.queryByText("Arquivo descoberto")).not.toBeInTheDocument();
  });
  it("desconecta e limpa a identidade da interface", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await go(user, "Configurações");
    await user.click(screen.getByRole("button", { name: "Desconectar conta" }));
    await settled();
    expect(screen.queryByText("fixture@example.invalid")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Entrar com Google" })).toBeInTheDocument();
  });
  it("propaga desconexão pelas ferramentas avançadas sem aguardar seu fechamento manual", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await go(user, "Configurações");
    await user.click(screen.getByRole("button", { name: "Configuração OAuth do mantenedor" }));
    const disconnect = await screen.findByRole("button", { name: "Desconectar" });
    await user.click(disconnect);
    await waitFor(() =>
      expect(screen.queryByText("fixture@example.invalid")).not.toBeInTheDocument(),
    );
    expect(screen.queryByRole("heading", { name: "Integração Google" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Entrar com Google" })).toBeInTheDocument();
  });
  it("bloqueia navegação de linha e marca durante consulta pendente sem invalidar seu resultado", async () => {
    rows.push(fixtureRow());
    const user = await load();
    const base = call.getMockImplementation()!;
    let finish: (() => void) | undefined;
    call.mockImplementation((command, args) => {
      if (command === "discover_spreadsheets")
        return new Promise((resolve) => {
          finish = () =>
            resolve(
              envelope({
                files: [
                  {
                    id: "fixture-late",
                    name: "Resultado pendente",
                    mime_type: "application/vnd.google-apps.spreadsheet",
                    modified_time: null,
                    shared: false,
                    drive_id: null,
                    resource_key: null,
                  },
                ],
                next_page_token: null,
                incomplete_search: false,
                complete: true,
                corpus: "allDrives",
              }),
            );
        });
      return base(command, args);
    });
    await user.click(
      within(screen.getByRole("navigation", { name: "Navegação principal" })).getByRole("button", {
        name: "Planilhas",
      }),
    );
    await waitFor(() => expect(finish).toBeDefined());
    const rowButton = screen.getByRole("button", { name: "Abrir detalhes de Receitas de teste" });
    expect(rowButton).toBeDisabled();
    await user.click(rowButton);
    await user.click(screen.getByRole("link", { name: /SheetsMonitor/ }));
    expect(screen.getByRole("heading", { level: 1, name: "Planilhas" })).toBeInTheDocument();
    const dashboardReads = call.mock.calls.filter(
      ([name]) => name === "get_dashboard_summary",
    ).length;
    rows[0].name = "Nome atualizado pelo serviço";
    act(() => signals.changed?.());
    await act(async () => finish?.());
    await screen.findByText("Resultado pendente");
    await screen.findByRole("button", { name: "Abrir detalhes de Nome atualizado pelo serviço" });
    expect(
      call.mock.calls.filter(([name]) => name === "get_dashboard_summary").length,
    ).toBeGreaterThan(dashboardReads);
    expect(call.mock.calls.some(([name]) => name === "get_spreadsheet_diagnostics")).toBe(false);
  });
});
