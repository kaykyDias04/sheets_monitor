import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { scopeForTabs } from "./pages/ScopeEditor";
import { extractErrorDiagnostic } from "./services/google";
import { managementError } from "./services/management";
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
    if (command === "updater_get_status")
      return {
        state: "up_to_date",
        current_version: "1.0.5",
        available_version: null,
        release_url: null,
        release_notes: null,
        published_at: null,
        install_mode: "per_user_nsis",
        can_auto_update: true,
        dismissed: false,
        last_checked_at: "2026-11-04T12:00:00Z",
        last_error_message: null,
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
      const anchor = Number(input.header_row ?? 1);
      const scope = scopeForTabs(null, [17]);
      scope.sheets[0].regions[0].range.start_row = anchor - 1;
      scope.sheets[0].regions[0].range.end_row = Math.min(500, anchor - 1 + 20_000);
      scope.sheets[0].regions[0].range.end_col = 8;
      scope.sheets[0].regions[0].header_row = anchor - 1;
      scope.sheets[0].regions[0].columns[0].header = anchor === 3 ? "Código Linha 3" : "Código";
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
    if (command === "export_change_events") {
      const fmt = String(input.format ?? "csv");
      const filterObj = (input.filter as Record<string, unknown> | undefined) ?? {};
      const empty = filterObj.event_type === "resolution";
      return envelope({
        exported_count: empty ? 0 : 4,
        file_path: empty ? null : `C:\\Exports\\sheets-monitor-logs.${fmt}`,
        format: fmt,
      });
    }
    if (command === "integration_disconnect") {
      session = {
        ...session,
        generation: 2,
        auth: { ...session.auth, status: "disconnected", account: null, granted_scopes: [] },
      };
      return structuredClone(session);
    }
    if (command === "integration_revoke") {
      if ((args as { confirmed?: boolean } | undefined)?.confirmed !== true)
        throw { code: "revocation_confirmation_required" };
      session = {
        ...session,
        generation: session.generation + 1,
        auth: { ...session.auth, status: "disconnected", account: null, granted_scopes: [] },
      };
      return structuredClone(session);
    }
    if (command === "integration_set_drive_opt_in") {
      const enabled = Boolean((args as { enabled?: boolean } | undefined)?.enabled);
      session = {
        ...session,
        generation: session.generation + 1,
        drive_metadata_opt_in: enabled,
      };
      return structuredClone(session);
    }
    if (command === "integration_connect") {
      session = {
        ...session,
        generation: session.generation + 1,
        auth: {
          status: "connected",
          account: {
            subject: "fixture-account",
            email: "fixture@example.invalid",
            name: "Conta de teste",
          },
          granted_scopes: [
            "https://www.googleapis.com/auth/spreadsheets.readonly",
            ...(session.drive_metadata_opt_in
              ? ["https://www.googleapis.com/auth/drive.metadata.readonly"]
              : []),
          ],
          expires_at: null,
        },
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

async function openAddDialog(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "Adicionar planilhas" }));
  const dialog = await screen.findByRole("dialog", { name: "Adicionar planilhas" });
  await waitFor(() =>
    expect(within(dialog).getByRole("button", { name: "Buscar planilhas" })).toBeEnabled(),
  );
  return dialog;
}

async function openFirst(user: ReturnType<typeof userEvent.setup>) {
  await go(user, "Planilhas");
  await user.click(screen.getByRole("button", { name: "Abrir detalhes de Receitas de teste" }));
  await screen.findByRole("heading", { name: "Receitas de teste" });
  expect(screen.queryByLabelText("Planilha selecionada")).not.toBeInTheDocument();
  await settled();
}

describe("Sprint 04 · operação sem demonstração", () => {
  it("simplifica sidebar, footer e configurações sem ferramentas avançadas do mantenedor", async () => {
    const user = await load();
    const nav = within(screen.getByRole("navigation", { name: "Navegação principal" }));
    expect(nav.getAllByRole("button")).toHaveLength(4);
    expect(nav.queryByRole("button", { name: /Detalhes da planilha/i })).not.toBeInTheDocument();
    expect(nav.queryByRole("button", { name: /Regras e baselines/i })).not.toBeInTheDocument();
    expect(screen.queryByText(/Seu controle, sua referência/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/Workspace local/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/Google somente leitura/i)).not.toBeInTheDocument();
    const footer = document.querySelector("footer.op-footer");
    expect(footer).toBeInTheDocument();
    expect(footer?.textContent?.trim()).toBe("");
    expect(screen.getByText("Encontre suas primeiras planilhas")).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name.startsWith("foundation_"))).toBe(false);

    await go(user, "Configurações");
    expect(screen.queryByText(/Ferramentas avançadas/i)).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Configuração OAuth do mantenedor/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Diagnóstico técnico do motor/i }),
    ).not.toBeInTheDocument();
  });

  it("mostra login sem pedir credenciais Cloud ao usuário comum", async () => {
    session.auth.account = null;
    session.auth.status = "disconnected";
    await load();
    expect(screen.getByRole("button", { name: "Entrar com Google" })).toBeEnabled();
    expect(screen.queryByLabelText(/Client ID/)).not.toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "get_dashboard_summary")).toBe(false);
  });

  it("exibe a listagem do Google Drive exclusivamente dentro de + Adicionar planilhas", async () => {
    rows.push(fixtureRow());
    const user = await load();
    expect(screen.queryByRole("button", { name: /Abrir detalhes/i })).not.toBeInTheDocument();
    await go(user, "Planilhas");
    expect(
      screen.getByRole("button", { name: "Abrir detalhes de Receitas de teste" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Encontrar no Google Drive")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Link da planilha")).not.toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "discover_spreadsheets")).toBe(false);

    const dialog = await openAddDialog(user);
    expect(within(dialog).getByText("Encontrar no Google Drive")).toBeInTheDocument();
    expect(within(dialog).getByText("Arquivo descoberto")).toBeInTheDocument();
    expect(within(dialog).getByText("Compartilhada")).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "register_spreadsheet")).toBe(false);

    await user.click(within(dialog).getByRole("button", { name: "Próxima página" }));
    await within(dialog).findByText("Segunda planilha");
    expect(call).toHaveBeenCalledWith("discover_spreadsheets", {
      input: expect.objectContaining({ page_token: "opaque-next" }),
    });
  });

  it("cadastra link válido pelo fluxo de adição e abre configuração sem cards Passo 1/2/3", async () => {
    const user = await load();
    await go(user, "Planilhas");
    const dialog = await openAddDialog(user);
    await user.click(within(dialog).getByLabelText("Link da planilha"));
    await user.paste("https://docs.google.com/spreadsheets/d/fixture-file/edit");
    await user.click(within(dialog).getByRole("button", { name: "Cadastrar por link" }));
    await screen.findByRole("heading", { name: "Receitas de teste" });
    await settled();
    expect(screen.queryByRole("dialog", { name: "Adicionar planilhas" })).not.toBeInTheDocument();
    expect(screen.getByText("Configuração pendente")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Verificar agora" })).toBeDisabled();
    expect(screen.queryByText(/^Passo [123]:/)).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Continuar para regras de proteção" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Abas" })).toHaveAttribute("aria-current", "page");
    expect(call).toHaveBeenCalledWith("register_spreadsheet", {
      input: {
        url: "https://docs.google.com/spreadsheets/d/fixture-file/edit",
        expected_generation: 1,
      },
    });
  });

  it("trata URL inválida e cadastro duplicado sem duplicar alertas nem perder o preenchimento", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await go(user, "Planilhas");
    const dialog = await openAddDialog(user);
    const input = within(dialog).getByLabelText("Link da planilha");
    await user.click(input);
    await user.paste("https://example.invalid/file");
    await user.click(within(dialog).getByRole("button", { name: "Cadastrar por link" }));
    const alerts = await screen.findAllByRole("alert");
    expect(alerts).toHaveLength(1);
    await waitFor(() =>
      expect(within(dialog).getByRole("button", { name: "Cadastrar por link" })).toBeEnabled(),
    );
    expect(rows).toHaveLength(1);
    expect(input).toHaveValue("https://example.invalid/file");

    await user.clear(input);
    await user.click(input);
    await user.paste("https://docs.google.com/spreadsheets/d/fixture-file/edit");
    await user.click(within(dialog).getByRole("button", { name: "Cadastrar por link" }));
    await within(dialog).findByText(/Esta planilha já está cadastrada nesta conta\./);
    expect(screen.getAllByRole("alert")).toHaveLength(1);
  });

  it("desabilita cadastro ao atingir 30 sem limitar descoberta dentro do fluxo de adição", async () => {
    rows = Array.from({ length: 30 }, (_, i) => ({
      ...fixtureRow(),
      spreadsheet_id: `fixture${i}`,
      name: `Planilha ${i}`,
    }));
    const user = await load();
    await go(user, "Planilhas");
    const dialog = await openAddDialog(user);
    expect(within(dialog).getByRole("button", { name: "Cadastrar por link" })).toBeDisabled();
    expect(within(dialog).getByText(/limite de 30 se aplica aos cadastros/)).toBeInTheDocument();
  }, 15000);

  it("executa o fluxo completo: adicionar -> abas -> salvar -> regras -> salvar -> referência", async () => {
    const user = await load();
    await go(user, "Planilhas");
    const dialog = await openAddDialog(user);
    await user.click(within(dialog).getByRole("button", { name: "Monitorar" }));
    await screen.findByRole("heading", { name: "Receitas de teste" });
    await settled();

    // Etapa 1: selecionar abas e avançar automaticamente para Regras de proteção
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    expect(rows[0].selected_sheet_ids).toEqual([17]);
    expect(screen.getByRole("button", { name: "Regras de proteção" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.queryByRole("button", { name: "Revisar referência" })).not.toBeInTheDocument();

    // Etapa 2: informar linha âncora dos cabeçalhos, sugerir regras, salvar e avançar automaticamente para Referência
    const anchorInput = screen.getByLabelText("Linha âncora dos cabeçalhos");
    expect(anchorInput).toHaveValue(1);
    await user.clear(anchorInput);
    await user.type(anchorInput, "3");
    await user.click(screen.getByRole("button", { name: "Sugerir regras para abas selecionadas" }));
    await settled();
    expect(call).toHaveBeenCalledWith("propose_integrity_rules", {
      input: {
        spreadsheet_id: "fixture-file",
        expected_generation: 1,
        expected_revision: 2,
        header_row: 3,
      },
    });
    expect(screen.getByRole("textbox", { name: /Cabeçalho protegido/ })).toHaveValue(
      "Código Linha 3",
    );
    expect(rows[0].scope).toBeNull();
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(screen.getByRole("button", { name: "Referência" })).toHaveAttribute(
      "aria-current",
      "page",
    );

    // Etapa 3: capturar estrutura, verificar ausência de JSON e aprovar referência
    await user.click(screen.getByRole("button", { name: "Capturar estrutura para revisão" }));
    await screen.findByText("Revise antes de aprovar");
    await settled();
    expect(screen.queryByText(/Estrutura capturada e regras propostas/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/"pinned-snapshot"/)).not.toBeInTheDocument();
    expect(screen.queryByText(/"schema_version"/)).not.toBeInTheDocument();
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

  it("valida seleção vazia ou falha ao salvar abas permanecendo na etapa atual com dados preservados", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);

    // Validação local: nenhuma aba selecionada não avança nem invoca o backend
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    expect(
      await screen.findByText(/Selecione ao menos uma aba de grade antes de salvar/),
    ).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "update_monitored_tabs")).toBe(false);
    expect(screen.getByRole("button", { name: "Abas" })).toHaveAttribute("aria-current", "page");

    // Falha de rede ao salvar: permanece em Abas, preserva checkbox marcado e não exibe sucesso
    const base = call.getMockImplementation()!;
    const checkbox = screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ });
    await user.click(checkbox);
    call.mockImplementation(async (command, args) => {
      if (command === "update_monitored_tabs") throw { code: "network" };
      return base(command, args);
    });
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    expect(screen.getByRole("button", { name: "Abas" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ })).toBeChecked();
    expect(screen.getAllByRole("alert")).toHaveLength(1);
    expect(screen.getByRole("alert")).toHaveTextContent(/Sem conexão com os serviços do Google/i);
    expect(screen.queryByText(/Seleção de abas salva/)).not.toBeInTheDocument();
    call.mockImplementation(base);
  }, 15000);

  it("explica no próprio formulário por que regras incompletas não foram salvas e permanece na etapa", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();

    // Já avançou automaticamente para Regras de proteção; cabeçalho vazio bloqueia avanço.
    expect(screen.getByRole("button", { name: "Regras de proteção" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    const save = screen.getByRole("button", { name: "Salvar regras de proteção" });
    await user.click(save);
    const problem = await screen.findByText(/informe o cabeçalho protegido/);
    expect(problem).toBeInTheDocument();
    expect(call.mock.calls.some(([name]) => name === "update_integrity_rules")).toBe(false);
    expect(save.closest("form")).toContainElement(problem);
    expect(screen.getByRole("button", { name: "Regras de proteção" })).toHaveAttribute(
      "aria-current",
      "page",
    );

    // Corrigido o campo, salva e avança automaticamente para Referência.
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    await user.click(save);
    await settled();
    expect(call.mock.calls.some(([name]) => name === "update_integrity_rules")).toBe(true);
    expect(rows[0].scope?.sheets[0].regions[0].columns[0].header).toBe("Código");
    expect(screen.getByRole("button", { name: "Referência" })).toHaveAttribute(
      "aria-current",
      "page",
    );
  }, 15000);

  it("recarrega a planilha após salvar mesmo quando o monitoramento atualiza em segundo plano", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
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
    expect(screen.getByRole("button", { name: "Referência" })).toHaveAttribute(
      "aria-current",
      "page",
    );

    // Voltando para Regras de proteção, o formulário reflete a revisão gravada e aceita nova edição.
    await user.click(screen.getByRole("button", { name: "Regras de proteção" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Salvar regras de proteção" })).toBeDisabled(),
    );
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), " 2");
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(rows[0].scope?.sheets[0].regions[0].columns[0].header).toBe("Código 2");

    // Já avançou para Referência com a revisão fresca e captura sem conflito.
    await user.click(screen.getByRole("button", { name: "Capturar estrutura para revisão" }));
    await screen.findByText("Revise antes de aprovar");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  }, 20000);

  it("recupera a tela preservando os dados preenchidos quando a revisão local ficou desatualizada", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await openFirst(user);
    await user.click(screen.getByRole("checkbox", { name: /Monitorar aba Receitas/ }));
    await user.click(screen.getByRole("button", { name: "Salvar seleção de abas" }));
    await settled();
    await user.type(screen.getByRole("textbox", { name: /Cabeçalho protegido/ }), "Código");
    rows[0].revision++; // outra operação alterou o cadastro depois que a tela foi carregada
    await user.click(screen.getByRole("button", { name: "Salvar regras de proteção" }));
    await settled();
    expect(await screen.findByText(/dados da planilha foram atualizados/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Regras de proteção" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("textbox", { name: /Cabeçalho protegido/ })).toHaveValue("Código");
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
        history: "delete_activity",
        confirmed: true,
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

  it("exporta logs filtrados em CSV e JSON e informa quando não há registros", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await go(user, "Atividades");
    await user.selectOptions(screen.getByLabelText("Planilha"), "fixture-file");
    await user.selectOptions(screen.getByLabelText("Aba"), "17");
    await user.selectOptions(screen.getByLabelText("Tipo"), "failure");
    await user.selectOptions(screen.getByLabelText("Status"), "active");
    await user.click(screen.getByRole("button", { name: "Exportar logs" }));
    await settled();
    expect(call).toHaveBeenCalledWith("export_change_events", {
      input: {
        format: "csv",
        filter: expect.objectContaining({
          spreadsheet_id: "fixture-file",
          sheet_id: 17,
          event_type: "failure",
          issue_status: "active",
          offset: 0,
        }),
      },
    });
    expect(await screen.findByText(/4 registro\(s\) exportado\(s\) em CSV/)).toBeInTheDocument();

    // Modificação desabilita Status quando não aplicável e exporta JSON
    await user.selectOptions(screen.getByLabelText("Tipo"), "modification");
    expect(screen.getByLabelText("Status")).toBeDisabled();
    await user.selectOptions(screen.getByLabelText("Formato de exportação"), "json");
    await user.click(screen.getByRole("button", { name: "Exportar logs" }));
    await settled();
    expect(call).toHaveBeenCalledWith("export_change_events", {
      input: {
        format: "json",
        filter: expect.objectContaining({
          event_type: "modification",
          issue_status: null,
        }),
      },
    });
    expect(await screen.findByText(/4 registro\(s\) exportado\(s\) em JSON/)).toBeInTheDocument();

    // Mensagem amigável quando nenhum registro corresponde aos filtros
    await user.selectOptions(screen.getByLabelText("Tipo"), "resolution");
    await user.click(screen.getByRole("button", { name: "Exportar logs" }));
    await settled();
    expect(
      await screen.findByText(
        "Nenhum registro corresponde aos filtros selecionados para exportação.",
      ),
    ).toBeInTheDocument();
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
    await user.click(screen.getByRole("button", { name: "Adicionar planilhas" }));
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

  it("revoga explicitamente o acesso no Google somente após confirmação e atualiza o header", async () => {
    const user = await load();
    const header = document.querySelector("header.topbar .topbar-inner");
    expect(header).toBeInTheDocument();
    expect(screen.getByText("Google conectado")).toBeInTheDocument();

    await go(user, "Configurações");
    await user.click(screen.getByText("Permissões e controle de acesso"));
    const revokeBtn = screen.getByRole("button", { name: "Revogar acesso no Google" });
    expect(revokeBtn).toBeDisabled();
    await user.click(
      screen.getByRole("checkbox", {
        name: /Confirmo que desejo revogar no Google o acesso concedido/,
      }),
    );
    expect(revokeBtn).toBeEnabled();
    await user.click(revokeBtn);
    await settled();
    expect(call).toHaveBeenCalledWith("integration_revoke", { confirmed: true });
    expect(screen.getByText("Google desconectado")).toBeInTheDocument();
  });

  it("permite habilitar descoberta do Drive diretamente na janela Adicionar planilhas", async () => {
    session.drive_metadata_opt_in = false;
    session.auth.granted_scopes = ["https://www.googleapis.com/auth/spreadsheets.readonly"];
    const user = await load();
    await go(user, "Planilhas");
    await user.click(screen.getByRole("button", { name: "Adicionar planilhas" }));
    const dialog = await screen.findByRole("dialog", { name: "Adicionar planilhas" });
    expect(within(dialog).getByRole("button", { name: "Buscar planilhas" })).toBeDisabled();
    await user.click(within(dialog).getByRole("button", { name: "Habilitar descoberta no Drive" }));
    await within(dialog).findByText("Arquivo descoberto");
    expect(call).toHaveBeenCalledWith("integration_set_drive_opt_in", { enabled: true });
    expect(call).toHaveBeenCalledWith("integration_connect");
  });

  it("padroniza categorias de erro com causa e ação sem expor detalhes sensíveis", () => {
    expect(extractErrorDiagnostic({ code: "token_expired", message: "raw-token-secret" })).toEqual({
      code: "token_expired",
      category: "auth",
      retryable: false,
    });
    expect(extractErrorDiagnostic({ code: "forbidden", retryable: false })).toEqual({
      code: "forbidden",
      category: "permission",
      retryable: false,
    });
    expect(extractErrorDiagnostic({ code: "network", retryable: true })).toEqual({
      code: "network",
      category: "connectivity",
      retryable: true,
    });
    expect(extractErrorDiagnostic({ code: "invalid_input" })).toEqual({
      code: "invalid_input",
      category: "validation",
      retryable: false,
    });
    expect(extractErrorDiagnostic({ code: "vault_failed" })).toEqual({
      code: "vault_failed",
      category: "internal",
      retryable: false,
    });

    const unknownMsg = managementError(
      { code: "unexpected_internal_code", message: "Bearer secret-value" },
      "salvar as regras de proteção",
    );
    expect(unknownMsg).toContain("salvar as regras de proteção");
    expect(unknownMsg).not.toContain("secret-value");
    expect(unknownMsg).not.toContain("unexpected_internal_code");
  });

  it("bloqueia navegação de linha e marca durante consulta pendente sem invalidar seu resultado", async () => {
    rows.push(fixtureRow());
    const user = await load();
    await go(user, "Planilhas");
    const base = call.getMockImplementation()!;
    let finish: (() => void) | undefined;
    call.mockImplementation((command, args) => {
      if (command === "get_dashboard_summary" && !finish)
        return new Promise((resolve) => {
          finish = () =>
            resolve(
              envelope({
                registered: rows.length,
                online: 0,
                offline: 0,
                indeterminate: 0,
                paused: 0,
                pending: rows.length,
                not_checked: 0,
                spreadsheets: rows,
                recent_events: [],
                recent_failures: [],
              }),
            );
        });
      return base(command, args);
    });
    await user.click(screen.getByRole("button", { name: "Atualizar dados" }));
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
    await screen.findByRole("button", { name: "Abrir detalhes de Nome atualizado pelo serviço" });
    expect(
      call.mock.calls.filter(([name]) => name === "get_dashboard_summary").length,
    ).toBeGreaterThan(dashboardReads);
    expect(call.mock.calls.some(([name]) => name === "get_spreadsheet_diagnostics")).toBe(false);
  });
});
