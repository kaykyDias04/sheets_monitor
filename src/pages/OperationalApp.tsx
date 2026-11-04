import { useCallback, useEffect, useRef, useState } from "react";
import {
  Activity as ActivityIcon,
  LayoutDashboard,
  FileSpreadsheet,
  ListChecks,
  Settings2,
  ShieldCheck,
  ChevronRight,
  RefreshCw,
  LoaderCircle,
  Link2,
  ArrowRight,
  Eye,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { desktop } from "@/services/platform";
import {
  getIntegrationState,
  connectIntegration,
  disconnectIntegration,
  testScriptAccess,
} from "@/services/google";
import * as api from "@/services/management";
import type { IntegrationState } from "@/types/google";
import type { EngineEnvelope, BaselineSummary } from "@/types/integrity";
import type {
  Activity,
  BaselineComparison,
  DashboardSummary,
  Diagnostics,
  EventFilter,
  Mutation,
  OperationalPreview,
  Page,
  Preferences,
  RegisteredSpreadsheet,
} from "@/types/management";
import type { DiscoveryPage, DiscoveryQuery } from "@/types/discovery";
import {
  EventDiagnostic,
  EventList,
  EmptyState,
  SummaryCards,
  dateLabel,
  StatusBadge,
} from "./OperationalShared";
import { RegisteredTable, DiscoveryPanel } from "./SpreadsheetTables";
import { SpreadsheetDetails } from "./SpreadsheetDetails";
import { ActivitiesPanel } from "./ActivitiesPanel";
import { AccountCard, PreferenceForm } from "./OperationalSettings";
import { GooglePage } from "./GooglePage";
import { IntegrityPage } from "./IntegrityPage";
import { MonitoringWorkspace } from "./MonitoringWorkspace";

type Route = "overview" | "sheets" | "details" | "activities" | "rules" | "settings";
const routes = [
  { id: "overview", label: "Visão geral", icon: LayoutDashboard },
  { id: "sheets", label: "Planilhas", icon: FileSpreadsheet },
  { id: "details", label: "Detalhes da planilha", icon: Eye },
  { id: "activities", label: "Atividades", icon: ActivityIcon },
  { id: "rules", label: "Regras e baselines", icon: ListChecks },
  { id: "settings", label: "Configurações", icon: Settings2 },
] as const;
const initialDiscovery: DiscoveryQuery = {
  search: "",
  order: "name",
  page_token: null,
  drive_id: null,
  page_size: 50,
};
const initialEvents: EventFilter = { offset: 0, limit: 25, query: "", script_only: false };
function identity(state: IntegrationState | null) {
  return state?.auth.account?.subject ?? null;
}
function sameSession(a: IntegrationState, b: IntegrationState) {
  return a.generation === b.generation && identity(a) === identity(b);
}

export function OperationalApp() {
  const [session, setSession] = useState<IntegrationState | null>(null);
  const [route, setRoute] = useState<Route>("overview");
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [preferences, setPreferences] = useState<Preferences | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<Diagnostics | null>(null);
  const [preview, setPreview] = useState<OperationalPreview | null>(null);
  const [versions, setVersions] = useState<BaselineSummary[]>([]);
  const [versionOffset, setVersionOffset] = useState(0);
  const [comparison, setComparison] = useState<BaselineComparison | null>(null);
  const [query, setQuery] = useState(initialDiscovery);
  const [discovery, setDiscovery] = useState<DiscoveryPage | null>(null);
  const [eventFilter, setEventFilter] = useState(initialEvents);
  const [eventPage, setEventPage] = useState<Page<Activity> | null>(null);
  const [event, setEvent] = useState<Activity | null>(null);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState<string | null>(desktop ? "Carregando aplicativo" : null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [advanced, setAdvanced] = useState<"google" | "engine" | null>(null);
  const current = useRef<IntegrationState | null>(null);
  const mounted = useRef(true);
  const locked = useRef(false);
  const pendingRefresh = useRef(false);
  const refreshRef = useRef<() => Promise<void>>(async () => undefined);
  const epoch = useRef(0);
  const detailTab = useRef<string | null>(null);
  const feedbackRef = useRef<HTMLDivElement>(null);
  const noticeRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    // O resultado de salvar aparece no topo da página; rola até ele para o usuário que está no fim de
    // um formulário longo não concluir que o botão não fez nada.
    (feedbackRef.current ?? noticeRef.current)?.scrollIntoView?.({ block: "nearest" });
  }, [error, notice]);
  const clearPrivate = useCallback(() => {
    setSummary(null);
    setPreferences(null);
    setSelectedId(null);
    setDetail(null);
    setPreview(null);
    setVersions([]);
    setComparison(null);
    setVersionOffset(0);
    setDiscovery(null);
    setQuery(initialDiscovery);
    setEventPage(null);
    setEvent(null);
    setEventFilter(initialEvents);
    setUrl("");
    setAdvanced(null);
  }, []);
  const acceptSession = useCallback(
    (next: IntegrationState) => {
      if (current.current && !sameSession(current.current, next)) {
        epoch.current++;
        clearPrivate();
      }
      current.current = next;
      setSession(next);
    },
    [clearPrivate],
  );
  const run = useCallback(
    async <T,>(
      label: string,
      work: () => Promise<EngineEnvelope<T>>,
      apply: (data: T) => void = () => undefined,
    ) => {
      if (locked.current || !desktop) return false;
      locked.current = true;
      const request = epoch.current;
      setBusy(label);
      setError("");
      setNotice("");
      try {
        const before = await getIntegrationState();
        if (!mounted.current) return false;
        if (current.current && !sameSession(current.current, before)) {
          acceptSession(before);
          throw { code: "stale_session" };
        }
        acceptSession(before);
        if (!before.auth.account) throw { code: "auth_required" };
        const result = await work();
        const after = await getIntegrationState();
        if (!mounted.current) return false;
        if (
          !sameSession(before, after) ||
          result.generation !== after.generation ||
          result.account_subject !== identity(after) ||
          !after.auth.account
        ) {
          acceptSession(after);
          clearPrivate();
          throw { code: "stale_session" };
        }
        acceptSession(after);
        if (request !== epoch.current) return false;
        apply(result.data);
        return true;
      } catch (e) {
        if (mounted.current) {
          setError(api.managementError(e));
          try {
            acceptSession(await getIntegrationState());
          } catch {
            /* Keep the original safe error. */
          }
        }
        return false;
      } finally {
        locked.current = false;
        if (mounted.current) setBusy(null);
        if (pendingRefresh.current && mounted.current) {
          pendingRefresh.current = false;
          void Promise.resolve().then(() => refreshRef.current());
        }
      }
    },
    [acceptSession, clearPrivate],
  );
  const refresh = useCallback(async () => {
    if (!desktop) return;
    if (locked.current) {
      pendingRefresh.current = true;
      return;
    }
    try {
      const state = await getIntegrationState();
      if (!mounted.current) return;
      acceptSession(state);
      if (!state.auth.account) {
        clearPrivate();
        setBusy(null);
        return;
      }
      await run(
        "Atualizando os dados locais",
        async () => {
          const [dashboard, prefs] = await Promise.all([api.getDashboard(), api.getPreferences()]);
          if (
            dashboard.generation !== prefs.generation ||
            dashboard.account_subject !== prefs.account_subject
          )
            throw { code: "stale_session" };
          return { ...dashboard, data: { dashboard: dashboard.data, prefs: prefs.data } };
        },
        ({ dashboard, prefs }) => {
          setSummary(dashboard);
          setPreferences(prefs);
        },
      );
    } catch (e) {
      if (mounted.current) {
        setError(api.managementError(e));
        setBusy(null);
      }
    }
  }, [acceptSession, clearPrivate, run]);
  useEffect(() => {
    refreshRef.current = refresh;
  }, [refresh]);
  useEffect(() => {
    mounted.current = true;
    void Promise.resolve().then(() => {
      if (mounted.current) return refresh();
    });
    return () => {
      mounted.current = false;
    };
  }, [refresh]);
  useEffect(() => {
    const theme = preferences?.theme ?? "system";
    document.documentElement.dataset.appearance = theme;
    return () => {
      delete document.documentElement.dataset.appearance;
    };
  }, [preferences?.theme]);
  function invalidatePreview() {
    setPreview(null);
    setComparison(null);
  }
  function navigate(next: Route) {
    if (locked.current) return;
    epoch.current++;
    setRoute(next);
    setError("");
    setNotice("");
    if (next !== "settings") {
      setAdvanced(null);
      if (next === "sheets" && canDiscover && !discovery) {
        void (async () => {
          await refresh();
          await discover({ ...query, page_token: null });
        })();
      } else if (next === "activities") {
        void (async () => {
          await refresh();
          await loadEvents({ ...eventFilter, offset: 0 });
        })();
      } else void refresh();
    }
  }
  async function sessionAction(work: () => Promise<IntegrationState>, success: string) {
    if (locked.current || !desktop) return;
    locked.current = true;
    epoch.current++;
    clearPrivate();
    setBusy("Processando autorização Google");
    setError("");
    setNotice("");
    try {
      acceptSession(await work());
      setNotice(success);
    } catch (e) {
      setError(api.managementError(e));
      try {
        acceptSession(await getIntegrationState());
      } catch {
        /* no unsafe fallback */
      }
    } finally {
      locked.current = false;
      setBusy(null);
    }
    await refresh();
  }
  async function loadDetail(id: string, destination: Route = "details") {
    if (locked.current) return;
    epoch.current++;
    if (id !== selectedId || destination !== route) detailTab.current = null;
    setSelectedId(id);
    setDetail(null);
    setPreview(null);
    setComparison(null);
    setVersions([]);
    setVersionOffset(0);
    setRoute(destination);
    await run(
      "Carregando detalhes da planilha",
      async () => {
        const [diagnosis, history] = await Promise.all([
          api.diagnostics(id),
          api.listBaselines(id),
        ]);
        if (
          diagnosis.generation !== history.generation ||
          diagnosis.account_subject !== history.account_subject
        )
          throw { code: "stale_session" };
        return { ...diagnosis, data: { diagnosis: diagnosis.data, history: history.data } };
      },
      ({ diagnosis, history }) => {
        setDetail(diagnosis);
        setVersions(history.items);
      },
    );
  }
  const rows = summary?.spreadsheets ?? [];
  function mutation(): Mutation {
    if (!detail || !current.current) throw { code: "configuration_pending" };
    return {
      spreadsheet_id: detail.spreadsheet.spreadsheet_id,
      expected_generation: current.current.generation,
      expected_revision: detail.spreadsheet.revision,
    };
  }
  async function mutate<T>(label: string, work: () => Promise<EngineEnvelope<T>>, message: string) {
    const id = selectedId;
    if (await run(label, work)) {
      invalidatePreview();
      await refresh();
      if (id) await loadDetail(id, route === "rules" ? "rules" : "details");
      setNotice(message);
    }
  }
  async function discover(next = query) {
    if (locked.current) return;
    setQuery(next);
    await run("Consultando arquivos no Google Drive", () => api.discover(next), setDiscovery);
  }
  async function register(urlValue: string) {
    const generation = current.current?.generation;
    if (generation === undefined) return;
    let registered: RegisteredSpreadsheet | null = null;
    if (
      await run(
        "Cadastrando a planilha selecionada",
        () => api.register(urlValue, generation),
        (row) => {
          registered = row;
        },
      )
    ) {
      setUrl("");
      await refresh();
      if (registered) await loadDetail((registered as RegisteredSpreadsheet).spreadsheet_id);
      setNotice(
        "Planilha cadastrada. Selecione abas e configure a proteção antes de aprovar a referência.",
      );
    }
  }
  async function loadEvents(next: EventFilter) {
    if (locked.current) return;
    setEventFilter(next);
    await run("Consultando atividades no banco local", () => api.events(next), setEventPage);
  }
  const disabled = !desktop || busy !== null;
  const accountAvailable = !!session?.auth.account;
  const canDiscover =
    session?.drive_metadata_opt_in === true &&
    session.auth.granted_scopes.includes("https://www.googleapis.com/auth/drive.metadata.readonly");
  const pageTitle = routes.find((item) => item.id === route)?.label ?? "Sheets Monitor";
  const accountActions = {
    session,
    disabled,
    onConnect: () => void sessionAction(connectIntegration, "Autorização concluída."),
    onDisconnect: () =>
      void sessionAction(disconnectIntegration, "Conta desconectada deste dispositivo."),
    onDrive: (enabled: boolean) =>
      void sessionAction(
        () => api.setDriveOptIn(enabled),
        enabled
          ? "Descoberta habilitada. Reautorize as permissões se necessário."
          : "Descoberta desativada.",
      ),
    onRefresh: () => void refresh(),
  };
  return (
    <div className="app-shell op-app">
      <a className="op-skip" href="#operational-content">
        Ir para o conteúdo
      </a>
      <aside className="sidebar">
        <a
          className="brand"
          href="#overview"
          onClick={(e) => {
            e.preventDefault();
            navigate("overview");
          }}
        >
          <span className="brand-symbol">
            <ActivityIcon size={25} />
          </span>
          <span className="brand-name">
            Sheets<span>Monitor</span>
            <small>INTEGRIDADE DAS SUAS PLANILHAS</small>
          </span>
        </a>
        <div className="workspace">
          <span className="workspace-symbol">
            <ShieldCheck size={19} />
          </span>
          <div>
            Workspace local<small>Google somente leitura</small>
          </div>
        </div>
        <p className="nav-label">ACOMPANHAMENTO</p>
        <nav aria-label="Navegação principal">
          {routes.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              className={`nav-item ${route === id ? "active" : ""}`}
              aria-current={route === id ? "page" : undefined}
              disabled={busy !== null}
              onClick={() => navigate(id)}
            >
              <Icon size={18} />
              <span>{label}</span>
              {id === "sheets" && summary && <small aria-hidden="true">{summary.registered}</small>}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">
          <div className="local-promise">
            <ShieldCheck size={20} />
            <div>
              Seu controle, sua referência<small>Aprovação sempre explícita</small>
            </div>
          </div>
          <div className="version">
            <span>v0.1.0 · SPRINT 05</span>
            <span>WINDOWS 11</span>
          </div>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <div className="breadcrumbs">
            <span>Workspace</span>
            <ChevronRight size={13} />
            <strong>{pageTitle}</strong>
          </div>
          <div className="topbar-tools">
            <span className={`op-connection ${accountAvailable ? "connected" : ""}`}>
              <span />
              {session?.auth.status === "connected"
                ? "Google conectado"
                : session?.auth.status === "reconnect_required"
                  ? "Reautorizar conta"
                  : "Google desconectado"}
            </span>
            <button
              className="op-avatar"
              title="Configurações da conta"
              aria-label="Abrir configurações da conta"
              disabled={disabled}
              onClick={() => navigate("settings")}
            >
              {session?.auth.account?.name?.slice(0, 1).toLocaleUpperCase("pt-BR") ?? "G"}
            </button>
          </div>
        </header>
        <main className="main-content op-main" id="operational-content">
          <div className="page-title">
            <div>
              <p className="eyebrow">SHEETS MONITOR · MONITORAMENTO CONTÍNUO</p>
              <h1>{pageTitle}</h1>
              <p>
                {route === "overview"
                  ? "Acompanhe a integridade e entenda cada alteração detectada."
                  : route === "sheets"
                    ? "Escolha os arquivos e mantenha sua proteção organizada."
                    : route === "activities"
                      ? "Consulte alterações, falhas e recuperações observadas."
                      : route === "rules"
                        ? "Revise a proteção e aprove referências com segurança."
                        : route === "settings"
                          ? "Gerencie a conta e as preferências deste dispositivo."
                          : "Proteção, referências e diagnósticos do arquivo selecionado."}
              </p>
            </div>
            <Button variant="outline" size="sm" disabled={disabled} onClick={() => void refresh()}>
              <RefreshCw />
              Atualizar dados
            </Button>
          </div>
          {!desktop && (
            <p className="feedback error" role="status">
              Abra o aplicativo Windows para acessar sua conta e seus dados. Esta prévia não simula
              planilhas reais.
            </p>
          )}
          {error && (
            <div className="feedback error" role="alert" ref={feedbackRef}>
              {error}
            </div>
          )}
          {notice && (
            <div className="feedback success" role="status" ref={noticeRef}>
              {notice}
            </div>
          )}
          {busy && (
            <div className="op-progress" role="status">
              <LoaderCircle className="animate-spin motion-reduce:animate-none" size={16} />
              {busy}…
            </div>
          )}
          <MonitoringWorkspace
            key={`${session?.generation ?? 0}:${session?.auth.account?.subject ?? "disconnected"}`}
            session={session}
            rows={rows}
            route={route}
            disabled={disabled}
            onChanged={() => void refresh()}
            onNavigate={navigate}
            onOpenSheet={(id) => void loadDetail(id)}
          />
          {!accountAvailable && route !== "settings" ? (
            <div className="op-stack">
              <Card className="op-welcome">
                <CardContent>
                  <p className="eyebrow">BEM-VINDO AO SEU MONITOR</p>
                  <h2>
                    Da planilha à confiança,
                    <br />
                    uma referência de cada vez.
                  </h2>
                  <p>
                    Conecte sua conta, encontre os arquivos e escolha o que precisa de proteção.
                    Você revisa a estrutura antes de aprovar qualquer referência.
                  </p>
                  <ol className="op-onboarding">
                    <li>
                      <span>1</span>Entrar com Google
                    </li>
                    <li>
                      <span>2</span>Escolher planilhas
                    </li>
                    <li>
                      <span>3</span>Configurar e aprovar
                    </li>
                  </ol>
                </CardContent>
              </Card>
              <AccountCard {...accountActions} />
            </div>
          ) : (
            <>
              {route === "overview" && summary && (
                <div className="op-stack">
                  <SummaryCards data={summary} />
                  {rows.length === 0 ? (
                    <Card>
                      <EmptyState
                        title="Encontre suas primeiras planilhas"
                        description="Descubra arquivos da sua conta e selecione até 30 para acompanhar. Nenhuma planilha é considerada Online antes de uma verificação."
                        action="Escolher planilhas"
                        onAction={() => navigate("sheets")}
                      />
                    </Card>
                  ) : (
                    <Card>
                      <CardHeader>
                        <div className="op-section-heading">
                          <div>
                            <CardTitle>Últimas verificações</CardTitle>
                            <CardDescription>
                              O estado reflete o escopo aprovado e a última análise concluída.
                            </CardDescription>
                          </div>
                          <Button variant="ghost" size="sm" onClick={() => navigate("sheets")}>
                            Ver planilhas
                            <ArrowRight />
                          </Button>
                        </div>
                      </CardHeader>
                      <CardContent>
                        <div className="op-table-scroll">
                          <table className="op-table">
                            <thead>
                              <tr>
                                <th>Planilha</th>
                                <th>Integridade</th>
                                <th>Última verificação</th>
                                <th>Diagnóstico</th>
                              </tr>
                            </thead>
                            <tbody>
                              {[...rows]
                                .sort((a, b) =>
                                  (b.last_checked_at ?? "").localeCompare(a.last_checked_at ?? ""),
                                )
                                .slice(0, 8)
                                .map((row) => (
                                  <tr key={row.spreadsheet_id}>
                                    <td>
                                      <strong>{row.name}</strong>
                                    </td>
                                    <td>
                                      <StatusBadge status={row.status} />
                                    </td>
                                    <td>{dateLabel(row.last_checked_at)}</td>
                                    <td>
                                      <Button
                                        variant="ghost"
                                        size="sm"
                                        disabled={disabled}
                                        onClick={() => void loadDetail(row.spreadsheet_id)}
                                      >
                                        Abrir detalhes
                                        <ArrowRight />
                                      </Button>
                                    </td>
                                  </tr>
                                ))}
                            </tbody>
                          </table>
                        </div>
                      </CardContent>
                    </Card>
                  )}
                  <div className="op-two-columns">
                    <Card>
                      <CardHeader>
                        <CardTitle>Últimas alterações</CardTitle>
                        <CardDescription>
                          Todas as diferenças observadas ficam registradas.
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        {summary.recent_events.length ? (
                          <EventList events={summary.recent_events} onOpen={setEvent} />
                        ) : (
                          <p className="op-empty">Nenhuma alteração registrada ainda.</p>
                        )}
                      </CardContent>
                    </Card>
                    <Card>
                      <CardHeader>
                        <CardTitle>Falhas estruturais recentes</CardTitle>
                        <CardDescription>Revise os diagnósticos antes de agir.</CardDescription>
                      </CardHeader>
                      <CardContent>
                        {summary.recent_failures.length ? (
                          <EventList events={summary.recent_failures} onOpen={setEvent} />
                        ) : (
                          <p className="op-empty">Nenhuma falha estrutural registrada.</p>
                        )}
                      </CardContent>
                    </Card>
                  </div>
                </div>
              )}
              {route === "sheets" && (
                <div className="op-stack">
                  <RegisteredTable
                    disabled={disabled}
                    rows={rows}
                    onOpen={(row) => void loadDetail(row.spreadsheet_id)}
                    onDiscover={() =>
                      document
                        .getElementById("discovery-panel")
                        ?.scrollIntoView({ behavior: "smooth", block: "start" })
                    }
                  />
                  <DiscoveryPanel
                    query={query}
                    onQuery={(next) => {
                      epoch.current++;
                      setQuery({ ...next, page_token: null });
                      setDiscovery(null);
                    }}
                    page={discovery}
                    onSearch={() => void discover({ ...query, page_token: null })}
                    onNext={() => {
                      if (discovery?.next_page_token)
                        void discover({ ...query, page_token: discovery.next_page_token });
                    }}
                    onFirst={() => void discover({ ...query, page_token: null })}
                    onRegister={(file) =>
                      void register(`https://docs.google.com/spreadsheets/d/${file.id}/edit`)
                    }
                    registered={rows.map((row) => row.spreadsheet_id)}
                    disabled={disabled}
                    canDiscover={canDiscover}
                  />
                  <Card>
                    <CardHeader>
                      <CardTitle>Adicionar por link</CardTitle>
                      <CardDescription>
                        Use um link Google Sheets quando preferir selecionar um arquivo diretamente.
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <form
                        className="op-url-form"
                        onSubmit={(e) => {
                          e.preventDefault();
                          void register(url.trim());
                        }}
                      >
                        <label className="op-field">
                          Link da planilha
                          <Input
                            type="url"
                            value={url}
                            required
                            placeholder="https://docs.google.com/spreadsheets/d/…/edit"
                            maxLength={2048}
                            disabled={disabled}
                            onChange={(e) => setUrl(e.target.value)}
                          />
                        </label>
                        <Button type="submit" disabled={disabled || rows.length >= 30}>
                          <Link2 />
                          Cadastrar por link
                        </Button>
                      </form>
                      <p className="op-hint">
                        Duplicidades são verificadas por conta. Cadastros pausados também ocupam uma
                        das 30 vagas.
                      </p>
                    </CardContent>
                  </Card>
                </div>
              )}
              {(route === "details" || route === "rules") && (
                <div className="op-stack">
                  <label className="op-field op-file-selector">
                    Planilha selecionada
                    <select
                      value={selectedId ?? ""}
                      disabled={disabled}
                      onChange={(e) => {
                        if (e.target.value) void loadDetail(e.target.value, route);
                      }}
                    >
                      <option value="">Selecione uma planilha cadastrada</option>
                      {rows.map((row) => (
                        <option key={row.spreadsheet_id} value={row.spreadsheet_id}>
                          {row.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  {detail ? (
                    <SpreadsheetDetails
                      key={`${detail.spreadsheet.spreadsheet_id}:${detail.spreadsheet.revision}:${route}`}
                      data={detail}
                      initialTab={detailTab.current ?? (route === "rules" ? "rules" : "tabs")}
                      onTabChange={(tab) => {
                        detailTab.current = tab;
                      }}
                      preview={preview}
                      versions={versions}
                      comparison={comparison}
                      versionOffset={versionOffset}
                      defaultHistoryDays={preferences?.history_days ?? 90}
                      disabled={disabled}
                      onInvalidate={invalidatePreview}
                      onPreview={() =>
                        void run(
                          "Capturando a estrutura para revisão",
                          () => api.previewBaseline(mutation()),
                          setPreview,
                        )
                      }
                      onApprove={(captured) =>
                        void mutate(
                          "Salvando referência aprovada",
                          () =>
                            api.approveBaseline({
                              ...mutation(),
                              expected_generation: captured.preview.session_generation,
                              expected_revision: captured.revision,
                              snapshot_id: captured.preview.snapshot.id,
                              snapshot_fingerprint: captured.preview.snapshot_fingerprint,
                              expected_version: captured.preview.active_baseline?.version ?? 0,
                              confirmed: true,
                            }),
                          "Referência aprovada explicitamente e salva como nova versão.",
                        )
                      }
                      onVersionPage={(offset) => {
                        void run(
                          "Consultando versões aprovadas",
                          () => api.listBaselines(detail.spreadsheet.spreadsheet_id, offset),
                          (page) => {
                            setVersions(page.items);
                            setVersionOffset(page.offset);
                          },
                        );
                      }}
                      onCompare={(before, after) =>
                        void run(
                          "Comparando referências",
                          () =>
                            api.compareBaselines(detail.spreadsheet.spreadsheet_id, before, after),
                          setComparison,
                        )
                      }
                      onComparisonClear={() => setComparison(null)}
                      onSaveTabs={(ids) =>
                        void mutate(
                          "Salvando seleção de abas",
                          () => api.saveTabs({ ...mutation(), sheet_ids: ids }),
                          "Seleção de abas salva. Revise as regras de proteção.",
                        )
                      }
                      onSaveRules={(scope) =>
                        void mutate(
                          "Salvando regras de proteção",
                          () => api.saveRules({ ...mutation(), scope }),
                          "Regras salvas. Capture e aprove uma referência para este conjunto de regras.",
                        )
                      }
                      onSuggestRules={async () => {
                        let proposal: import("@/types/integrity").MonitorScope | null = null;
                        await run(
                          "Lendo cabeçalhos para sugerir proteção",
                          () => api.proposeRules(mutation()),
                          (scope) => {
                            proposal = scope;
                          },
                        );
                        return proposal;
                      }}
                      onPause={() =>
                        void mutate(
                          detail.spreadsheet.paused ? "Retomando cadastro" : "Pausando cadastro",
                          () =>
                            detail.spreadsheet.paused
                              ? api.resume(mutation())
                              : api.pause(mutation()),
                          detail.spreadsheet.paused
                            ? "Cadastro retomado. O serviço verificará a proteção quando o monitoramento estiver ativo."
                            : "Cadastro pausado.",
                        )
                      }
                      onCheck={() =>
                        void mutate(
                          "Adicionando verificação à fila",
                          () => api.check(mutation()),
                          "Verificação adicionada à fila. O resultado será atualizado pelo serviço.",
                        )
                      }
                      onRefresh={() =>
                        void mutate(
                          "Atualizando nome e abas no Google",
                          () => api.refreshTabs(detail.spreadsheet.spreadsheet_id),
                          "Metadados atualizados. Confira abas removidas, renomeadas e regras pendentes.",
                        )
                      }
                      onOpen={() =>
                        void run("Abrindo arquivo no navegador", () =>
                          api.openSpreadsheet(detail.spreadsheet.spreadsheet_id),
                        )
                      }
                      onHistory={() => {
                        setRoute("activities");
                        void loadEvents({
                          ...initialEvents,
                          spreadsheet_id: detail.spreadsheet.spreadsheet_id,
                        });
                      }}
                      onRemove={(history) => {
                        void (async () => {
                          if (
                            await run("Removendo cadastro local", () =>
                              api.remove({ ...mutation(), confirmed: true, history }),
                            )
                          ) {
                            setSelectedId(null);
                            setDetail(null);
                            setPreview(null);
                            setRoute("sheets");
                            await refresh();
                            setNotice(
                              "Cadastro removido localmente. O arquivo Google permanece intacto.",
                            );
                          }
                        })();
                      }}
                      onAssociateScript={(scriptId) =>
                        void mutate(
                          "Consultando e associando Apps Script",
                          async () => {
                            const result = await testScriptAccess({
                              script_id: scriptId,
                              spreadsheet_id: detail.spreadsheet.spreadsheet_id,
                            });
                            if (result.script.status !== "success")
                              throw result.script.error ?? { code: "forbidden" };
                            return {
                              generation: result.generation,
                              account_subject: result.account_subject,
                              data: null,
                            };
                          },
                          "Projeto consultado e associado. Inclua-o nas regras para analisar suas dependências.",
                        )
                      }
                    />
                  ) : (
                    !busy && (
                      <Card>
                        <EmptyState
                          title="Escolha uma planilha para continuar"
                          description="Abra um cadastro para selecionar abas, configurar proteção, revisar referências e consultar diagnósticos."
                          action="Ver planilhas"
                          onAction={() => navigate("sheets")}
                        />
                      </Card>
                    )
                  )}
                </div>
              )}
              {route === "activities" && (
                <ActivitiesPanel
                  filter={eventFilter}
                  onFilter={(next) => {
                    epoch.current++;
                    setEventFilter(next);
                    setEventPage(null);
                  }}
                  page={eventPage}
                  rows={rows}
                  disabled={disabled}
                  onSearch={() => void loadEvents({ ...eventFilter, offset: 0 })}
                  onPage={(offset) => void loadEvents({ ...eventFilter, offset })}
                  onOpen={setEvent}
                />
              )}
              {route === "settings" && (
                <div className="op-stack">
                  <AccountCard {...accountActions} />
                  {preferences && (
                    <PreferenceForm
                      key={JSON.stringify(preferences)}
                      value={preferences}
                      disabled={disabled}
                      onSave={(next) => {
                        const generation = current.current?.generation;
                        if (generation !== undefined)
                          void run(
                            "Salvando preferências",
                            () => api.savePreferences(next, generation),
                            (saved) => {
                              setPreferences(saved);
                              setNotice("Preferências salvas nesta conta.");
                            },
                          );
                      }}
                    />
                  )}
                  <Card>
                    <CardHeader>
                      <CardTitle>Ferramentas avançadas</CardTitle>
                      <CardDescription>
                        Diagnóstico técnico e configuração de desenvolvimento, fora do fluxo de uso
                        diário.
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="op-actions">
                        <Button
                          variant="outline"
                          disabled={disabled}
                          onClick={() => setAdvanced(advanced === "google" ? null : "google")}
                        >
                          Configuração OAuth do mantenedor
                        </Button>
                        <Button
                          variant="outline"
                          disabled={disabled}
                          onClick={() => setAdvanced(advanced === "engine" ? null : "engine")}
                        >
                          Diagnóstico técnico do motor
                        </Button>
                      </div>
                      {advanced && (
                        <div className="op-advanced-content">
                          {advanced === "google" ? (
                            <GooglePage onSessionChange={acceptSession} />
                          ) : (
                            <IntegrityPage />
                          )}
                          <Button
                            variant="outline"
                            onClick={() => {
                              setAdvanced(null);
                              void refresh();
                            }}
                          >
                            Fechar ferramentas e atualizar conexão
                          </Button>
                        </div>
                      )}
                    </CardContent>
                  </Card>
                </div>
              )}
            </>
          )}
          <footer className="op-footer">
            <ShieldCheck size={14} />
            Referências locais · Google somente leitura
            <span>Verificações no aplicativo · Alertas conforme sua preferência</span>
          </footer>
        </main>
      </div>
      <EventDiagnostic event={event} onClose={() => setEvent(null)} />
    </div>
  );
}
