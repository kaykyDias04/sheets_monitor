import { useCallback, useEffect, useRef, useState } from "react";
import {
  Activity as ActivityIcon,
  LayoutDashboard,
  FileSpreadsheet,
  Settings2,
  ChevronRight,
  RefreshCw,
  LoaderCircle,
  ArrowRight,
  ArrowLeft,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { desktop } from "@/services/platform";
import {
  getIntegrationState,
  connectIntegration,
  disconnectIntegration,
  revokeIntegration,
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
import { AddSpreadsheetsDialog, RegisteredTable } from "./SpreadsheetTables";
import { SpreadsheetDetails } from "./SpreadsheetDetails";
import { ActivitiesPanel } from "./ActivitiesPanel";
import { AccountCard, PreferenceForm } from "./OperationalSettings";
import { MonitoringWorkspace } from "./MonitoringWorkspace";
import { CloudSyncPanel } from "./CloudSyncPanel";

type Route = "overview" | "sheets" | "details" | "activities" | "settings";
const routes = [
  { id: "overview", label: "Visão geral", icon: LayoutDashboard },
  { id: "sheets", label: "Planilhas", icon: FileSpreadsheet },
  { id: "activities", label: "Atividades", icon: ActivityIcon },
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
function isConflictCode(error: unknown) {
  if (!error || typeof error !== "object") return false;
  const code = (error as Record<string, unknown>).code;
  return code === "configuration_conflict" || code === "integrity_baseline_conflict";
}

export function OperationalApp() {
  const [session, setSession] = useState<IntegrationState | null>(null);
  const [route, setRoute] = useState<Route>("overview");
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [preferences, setPreferences] = useState<Preferences | null>(null);
  const [detail, setDetail] = useState<Diagnostics | null>(null);
  const [detailResetKey, setDetailResetKey] = useState(0);
  const [initialDetailTab, setInitialDetailTab] = useState<string>("tabs");
  const [preview, setPreview] = useState<OperationalPreview | null>(null);
  const [versions, setVersions] = useState<BaselineSummary[]>([]);
  const [versionOffset, setVersionOffset] = useState(0);
  const [comparison, setComparison] = useState<BaselineComparison | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const [query, setQuery] = useState(initialDiscovery);
  const [discovery, setDiscovery] = useState<DiscoveryPage | null>(null);
  const [eventFilter, setEventFilter] = useState(initialEvents);
  const [eventPage, setEventPage] = useState<Page<Activity> | null>(null);
  const [event, setEvent] = useState<Activity | null>(null);
  const [exportFeedback, setExportFeedback] = useState<{
    empty: boolean;
    message: string;
  } | null>(null);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState<string | null>(desktop ? "Carregando aplicativo" : null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const current = useRef<IntegrationState | null>(null);
  const selectedIdRef = useRef<string | null>(null);
  const mounted = useRef(true);
  const locked = useRef(false);
  const bgRefresh = useRef<Promise<void> | null>(null);
  const pendingRefresh = useRef(false);
  const silentRefreshRef = useRef<() => Promise<void>>(async () => undefined);
  const epoch = useRef(0);
  const detailTab = useRef<string | null>(null);
  const feedbackRef = useRef<HTMLDivElement>(null);
  const noticeRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    (feedbackRef.current ?? noticeRef.current)?.scrollIntoView?.({ block: "nearest" });
  }, [error, notice]);

  const clearPrivate = useCallback(() => {
    setSummary(null);
    setPreferences(null);
    selectedIdRef.current = null;
    setDetail(null);
    setPreview(null);
    setVersions([]);
    setComparison(null);
    setVersionOffset(0);
    setAddOpen(false);
    setDiscovery(null);
    setQuery(initialDiscovery);
    setEventPage(null);
    setEvent(null);
    setEventFilter(initialEvents);
    setExportFeedback(null);
    setUrl("");
    detailTab.current = null;
    setInitialDetailTab("tabs");
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

  /**
   * Recarrega painel, preferências e (quando aberta) a planilha atual dentro de uma operação que
   * já detém `locked.current = true`, impedindo corridas com eventos `monitoring:changed`.
   */
  const reloadUnderLock = useCallback(
    async (expected: IntegrationState, targetSpreadsheetId: string | null) => {
      const [dashboard, prefs] = await Promise.all([api.getDashboard(), api.getPreferences()]);
      let detailPair: {
        diagnosis: EngineEnvelope<Diagnostics>;
        history: EngineEnvelope<Page<BaselineSummary>>;
      } | null = null;
      if (targetSpreadsheetId) {
        const [diagnosis, history] = await Promise.all([
          api.diagnostics(targetSpreadsheetId),
          api.listBaselines(targetSpreadsheetId, 0),
        ]);
        detailPair = { diagnosis, history };
      }
      const after = await getIntegrationState();
      if (!mounted.current) return false;
      const envelopes = [
        dashboard,
        prefs,
        ...(detailPair ? [detailPair.diagnosis, detailPair.history] : []),
      ];
      if (
        !sameSession(expected, after) ||
        !after.auth.account ||
        envelopes.some(
          (env) => env.generation !== after.generation || env.account_subject !== identity(after),
        )
      ) {
        acceptSession(after);
        clearPrivate();
        throw { code: "stale_session" };
      }
      acceptSession(after);
      setSummary(dashboard.data);
      setPreferences(prefs.data);
      if (detailPair && selectedIdRef.current === targetSpreadsheetId) {
        setDetail(detailPair.diagnosis.data);
        setVersions(detailPair.history.data.items);
        setVersionOffset(0);
      }
      return true;
    },
    [acceptSession, clearPrivate],
  );

  const handleFailureUnderLock = useCallback(
    async (failure: unknown, targetSpreadsheetId: string | null, context?: string) => {
      if (!mounted.current) return;
      if (isConflictCode(failure)) {
        try {
          const latest = await getIntegrationState();
          if (!mounted.current) return;
          acceptSession(latest);
          if (latest.auth.account) {
            setPreview(null);
            setComparison(null);
            await reloadUnderLock(latest, targetSpreadsheetId);
            pendingRefresh.current = false;
            if (mounted.current) {
              setError(
                "Os dados da planilha foram atualizados porque mudaram em outra operação. Revise os campos preservados na tela e tente novamente.",
              );
            }
            return;
          }
        } catch {
          /* Fall back to standard error handling below. */
        }
      }
      setError(api.managementError(failure, context));
      try {
        acceptSession(await getIntegrationState());
      } catch {
        /* Keep the original safe error. */
      }
    },
    [acceptSession, reloadUnderLock],
  );

  /**
   * Atualização silenciosa disparada por `monitoring:changed`: não bloqueia cliques do usuário
   * nem limpa mensagens de confirmação ou erro visíveis na tela.
   */
  const silentRefresh = useCallback(async () => {
    if (!desktop || !mounted.current) return;
    if (locked.current) {
      pendingRefresh.current = true;
      return;
    }
    if (bgRefresh.current) {
      pendingRefresh.current = true;
      return bgRefresh.current;
    }
    const task = (async () => {
      try {
        do {
          pendingRefresh.current = false;
          if (locked.current || !mounted.current) {
            if (locked.current) pendingRefresh.current = true;
            return;
          }
          const before = await getIntegrationState();
          if (!mounted.current || locked.current) {
            if (locked.current) pendingRefresh.current = true;
            return;
          }
          acceptSession(before);
          if (!before.auth.account) {
            clearPrivate();
            return;
          }
          const [dashboard, prefs] = await Promise.all([api.getDashboard(), api.getPreferences()]);
          const after = await getIntegrationState();
          if (!mounted.current || locked.current) {
            if (locked.current) pendingRefresh.current = true;
            return;
          }
          if (
            !sameSession(before, after) ||
            !after.auth.account ||
            dashboard.generation !== after.generation ||
            prefs.generation !== after.generation ||
            dashboard.account_subject !== identity(after) ||
            prefs.account_subject !== identity(after)
          ) {
            acceptSession(after);
            if (!after.auth.account) clearPrivate();
            continue;
          }
          acceptSession(after);
          setSummary(dashboard.data);
          setPreferences(prefs.data);
        } while (pendingRefresh.current && mounted.current && !locked.current);
      } catch {
        /* Background refresh errors do not clobber foreground UI state. */
      } finally {
        bgRefresh.current = null;
      }
    })();
    bgRefresh.current = task;
    return task;
  }, [acceptSession, clearPrivate]);

  useEffect(() => {
    silentRefreshRef.current = silentRefresh;
  }, [silentRefresh]);

  const run = useCallback(
    async <T,>(
      label: string,
      work: () => Promise<EngineEnvelope<T>>,
      apply: (data: T) => void = () => undefined,
      context?: string,
    ) => {
      if (locked.current || !desktop) return false;
      locked.current = true;
      const request = epoch.current;
      const targetId = selectedIdRef.current;
      setBusy(label);
      setError("");
      setNotice("");
      try {
        if (bgRefresh.current) await bgRefresh.current;
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
        await handleFailureUnderLock(e, targetId, context ?? label.toLocaleLowerCase("pt-BR"));
        return false;
      } finally {
        locked.current = false;
        if (mounted.current) setBusy(null);
        if (pendingRefresh.current && mounted.current) {
          pendingRefresh.current = false;
          void Promise.resolve().then(() => silentRefreshRef.current());
        }
      }
    },
    [acceptSession, clearPrivate, handleFailureUnderLock],
  );

  const refresh = useCallback(
    async (resetDetailForms = false) => {
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
        const targetId = selectedIdRef.current;
        await run(
          "Atualizando os dados locais",
          async () => {
            const [dashboard, prefs] = await Promise.all([
              api.getDashboard(),
              api.getPreferences(),
            ]);
            let detailPair: {
              diagnosis: EngineEnvelope<Diagnostics>;
              history: EngineEnvelope<Page<BaselineSummary>>;
            } | null = null;
            if (resetDetailForms && targetId) {
              const [diagnosis, history] = await Promise.all([
                api.diagnostics(targetId),
                api.listBaselines(targetId, 0),
              ]);
              detailPair = { diagnosis, history };
            }
            if (
              dashboard.generation !== prefs.generation ||
              dashboard.account_subject !== prefs.account_subject ||
              (detailPair &&
                (detailPair.diagnosis.generation !== dashboard.generation ||
                  detailPair.history.generation !== dashboard.generation ||
                  detailPair.diagnosis.account_subject !== dashboard.account_subject ||
                  detailPair.history.account_subject !== dashboard.account_subject))
            )
              throw { code: "stale_session" };
            return {
              ...dashboard,
              data: { dashboard: dashboard.data, prefs: prefs.data, detailPair },
            };
          },
          ({ dashboard, prefs, detailPair }) => {
            setSummary(dashboard);
            setPreferences(prefs);
            if (detailPair && selectedIdRef.current === targetId) {
              setDetail(detailPair.diagnosis.data);
              setVersions(detailPair.history.data.items);
              setVersionOffset(0);
              setInitialDetailTab(detailTab.current ?? "tabs");
              setDetailResetKey((n) => n + 1);
            }
          },
          "atualizar os dados locais",
        );
      } catch (e) {
        if (mounted.current) {
          setError(api.managementError(e, "atualizar os dados locais"));
          setBusy(null);
        }
      }
    },
    [acceptSession, clearPrivate, run],
  );

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
    setAddOpen(false);
    setError("");
    setNotice("");
    if (next !== "settings") {
      if (next === "activities") {
        void (async () => {
          await refresh();
          await loadEvents({ ...eventFilter, offset: 0 });
        })();
      } else {
        void refresh();
      }
    }
  }

  async function sessionAction(
    work: () => Promise<IntegrationState>,
    success: string,
    context: string,
  ) {
    if (locked.current || !desktop) return;
    locked.current = true;
    epoch.current++;
    setBusy("Processando autorização Google");
    setError("");
    setNotice("");
    try {
      if (bgRefresh.current) await bgRefresh.current;
      const next = await work();
      acceptSession(next);
      if (next.auth.account) {
        await reloadUnderLock(next, null);
      } else {
        clearPrivate();
      }
      if (mounted.current) setNotice(success);
    } catch (e) {
      setError(api.managementError(e, context));
      try {
        const latest = await getIntegrationState();
        acceptSession(latest);
        if (!latest.auth.account) clearPrivate();
      } catch {
        /* no unsafe fallback */
      }
    } finally {
      locked.current = false;
      if (mounted.current) setBusy(null);
    }
  }

  async function loadDetail(id: string): Promise<boolean> {
    if (locked.current) return false;
    epoch.current++;
    if (id !== selectedIdRef.current) {
      detailTab.current = null;
      setInitialDetailTab("tabs");
    } else {
      setInitialDetailTab(detailTab.current ?? "tabs");
    }
    selectedIdRef.current = id;
    setDetail(null);
    setPreview(null);
    setComparison(null);
    setVersions([]);
    setVersionOffset(0);
    setRoute("details");
    return run(
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
        setDetailResetKey((n) => n + 1);
      },
      "carregar os detalhes da planilha",
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

  async function mutate<T>(
    label: string,
    work: () => Promise<EngineEnvelope<T>>,
    message: string,
    options?: { resetDetailForms?: boolean; context?: string },
  ): Promise<boolean> {
    if (locked.current || !desktop) return false;
    locked.current = true;
    const id = selectedIdRef.current;
    setBusy(label);
    setError("");
    setNotice("");
    try {
      if (bgRefresh.current) await bgRefresh.current;
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
      invalidatePreview();
      await reloadUnderLock(after, id);
      if (options?.resetDetailForms && mounted.current) {
        setInitialDetailTab(detailTab.current ?? "tabs");
        setDetailResetKey((n) => n + 1);
      }
      pendingRefresh.current = false;
      if (mounted.current) setNotice(message);
      return true;
    } catch (e) {
      await handleFailureUnderLock(e, id, options?.context ?? label.toLocaleLowerCase("pt-BR"));
      return false;
    } finally {
      locked.current = false;
      if (mounted.current) setBusy(null);
      if (pendingRefresh.current && mounted.current) {
        pendingRefresh.current = false;
        void Promise.resolve().then(() => silentRefreshRef.current());
      }
    }
  }

  async function discover(next = query) {
    if (locked.current) return;
    setQuery(next);
    await run(
      "Consultando arquivos no Google Drive",
      () => api.discover(next),
      setDiscovery,
      "listar as planilhas do Google Drive",
    );
  }

  function openAddFlow() {
    if (locked.current) return;
    setError("");
    setNotice("");
    setAddOpen(true);
    if (canDiscover && !discovery) {
      void discover({ ...query, page_token: null });
    }
  }

  async function register(urlValue: string) {
    if (!urlValue) {
      setNotice("");
      setError("Informe o link completo da planilha no Google Sheets antes de cadastrar.");
      return;
    }
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
        "cadastrar a planilha selecionada",
      )
    ) {
      setUrl("");
      setAddOpen(false);
      await refresh();
      if (registered) {
        const loaded = await loadDetail((registered as RegisteredSpreadsheet).spreadsheet_id);
        if (loaded) {
          setNotice(
            "Planilha cadastrada. Selecione as abas monitoradas e salve para avançar às regras de proteção.",
          );
        }
      }
    }
  }

  async function loadEvents(next: EventFilter) {
    if (locked.current) return;
    setEventFilter(next);
    await run(
      "Consultando atividades no banco local",
      () => api.events(next),
      setEventPage,
      "consultar o histórico de atividades",
    );
  }

  const disabled = !desktop || busy !== null;
  const accountAvailable = !!session?.auth.account;
  const canDiscover =
    session?.drive_metadata_opt_in === true &&
    session.auth.granted_scopes.includes("https://www.googleapis.com/auth/drive.metadata.readonly");
  const activeNavId = route === "details" ? "sheets" : route;
  const pageTitle =
    route === "details"
      ? "Detalhes da planilha"
      : (routes.find((item) => item.id === route)?.label ?? "Sheets Monitor");
  const feedbackInsideDetails = route === "details" && detail !== null;
  const feedbackInsideAddDialog = route === "sheets" && addOpen;
  const feedbackDelegated = feedbackInsideDetails || feedbackInsideAddDialog;

  const accountActions = {
    session,
    disabled,
    onConnect: () =>
      void sessionAction(connectIntegration, "Autorização concluída.", "conectar a conta Google"),
    onDisconnect: () =>
      void sessionAction(
        disconnectIntegration,
        "Conta desconectada deste dispositivo.",
        "desconectar a conta Google",
      ),
    onRevoke: () =>
      void sessionAction(
        revokeIntegration,
        "Acesso revogado no Google e sessão encerrada neste dispositivo.",
        "revogar o acesso da conta no Google",
      ),
    onDrive: (enabled: boolean) =>
      void sessionAction(
        () => api.setDriveOptIn(enabled),
        enabled
          ? "Descoberta habilitada. Reautorize as permissões se necessário."
          : "Descoberta desativada.",
        "alterar a permissão de descoberta no Drive",
      ),
    onRefresh: () => void refresh(),
  };

  async function enableDiscoveryFromDialog() {
    if (locked.current || !desktop) return;
    let readyToList = false;
    await sessionAction(
      async () => {
        const optedIn = await api.setDriveOptIn(true);
        const hasDriveGrant = optedIn.auth.granted_scopes.includes(
          "https://www.googleapis.com/auth/drive.metadata.readonly",
        );
        const finalState = hasDriveGrant ? optedIn : await connectIntegration();
        readyToList =
          finalState.drive_metadata_opt_in &&
          finalState.auth.granted_scopes.includes(
            "https://www.googleapis.com/auth/drive.metadata.readonly",
          );
        return finalState;
      },
      "Descoberta pelo Google Drive habilitada.",
      "habilitar a descoberta de planilhas no Google Drive",
    );
    if (readyToList && mounted.current) {
      await discover({ ...query, page_token: null });
    }
  }

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
        <p className="nav-label">ACOMPANHAMENTO</p>
        <nav aria-label="Navegação principal">
          {routes.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              className={`nav-item ${activeNavId === id ? "active" : ""}`}
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
          <div className="version">
            <span>v1.0.5</span>
            <span>WINDOWS 11</span>
          </div>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <div className="topbar-inner">
            <div className="breadcrumbs">
              <span>Workspace</span>
              <ChevronRight size={14} aria-hidden="true" />
              {route === "details" ? (
                <>
                  <button
                    type="button"
                    className="op-breadcrumb-link"
                    disabled={busy !== null}
                    onClick={() => navigate("sheets")}
                  >
                    Planilhas
                  </button>
                  <ChevronRight size={14} aria-hidden="true" />
                  <strong>{detail?.spreadsheet.name ?? pageTitle}</strong>
                </>
              ) : (
                <strong>{pageTitle}</strong>
              )}
            </div>
            <div className="topbar-tools">
              <span
                className={`op-connection ${
                  session?.auth.status === "connected"
                    ? "connected"
                    : session?.auth.status === "reconnect_required"
                      ? "reconnect"
                      : ""
                }`}
              >
                <span className="op-connection-dot" aria-hidden="true" />
                <span className="op-connection-label">
                  {session?.auth.status === "connected"
                    ? "Google conectado"
                    : session?.auth.status === "reconnect_required"
                      ? "Reautorizar conta"
                      : "Google desconectado"}
                </span>
              </span>
              <button
                type="button"
                className="op-avatar"
                title={
                  session?.auth.account?.email
                    ? `Conta conectada: ${session.auth.account.email}`
                    : "Configurações da conta"
                }
                aria-label="Abrir configurações da conta"
                disabled={busy !== null}
                onClick={() => navigate("settings")}
              >
                {session?.auth.account?.name?.[0]?.toUpperCase() ??
                  session?.auth.account?.email?.[0]?.toUpperCase() ??
                  "G"}
              </button>
            </div>
          </div>
        </header>
        <main id="operational-content" className="content">
          <div className="page-head">
            <div>
              <h1>{pageTitle}</h1>
              <p>
                {route === "overview"
                  ? "Acompanhe a integridade e as verificações recentes das suas planilhas."
                  : route === "sheets"
                    ? "Gerencie e acompanhe até 30 planilhas cadastradas."
                    : route === "activities"
                      ? "Histórico local de alterações, falhas e recuperações."
                      : route === "settings"
                        ? "Gerencie a conta e as preferências deste dispositivo."
                        : "Escolha abas, defina regras de proteção e aprove a referência da planilha."}
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={disabled}
              onClick={() => void refresh(true)}
            >
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
          {!feedbackDelegated && error && (
            <div className="feedback error" role="alert" ref={feedbackRef}>
              {error}
            </div>
          )}
          {!feedbackDelegated && notice && (
            <div className="feedback success" role="status" ref={noticeRef}>
              {notice}
            </div>
          )}
          {!feedbackDelegated && busy && (
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
            suppressedError={feedbackDelegated ? "" : error}
            onChanged={() => void silentRefresh()}
            onNavigate={navigate}
            onOpenSheet={() => navigate("sheets")}
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
                        description="Adicione arquivos da sua conta e selecione até 30 para acompanhar. Nenhuma planilha é considerada Online antes de uma verificação."
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
                    onDiscover={openAddFlow}
                  />
                  <AddSpreadsheetsDialog
                    open={addOpen}
                    onOpenChange={(nextOpen) => {
                      setAddOpen(nextOpen);
                      if (!nextOpen) {
                        setError("");
                        setNotice("");
                      }
                    }}
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
                    onRegisterFile={(file) =>
                      void register(`https://docs.google.com/spreadsheets/d/${file.id}/edit`)
                    }
                    url={url}
                    onUrlChange={setUrl}
                    onRegisterUrl={(value) => void register(value)}
                    registered={rows.map((row) => row.spreadsheet_id)}
                    disabled={disabled}
                    canDiscover={canDiscover}
                    onEnableDiscovery={() => void enableDiscoveryFromDialog()}
                    busy={busy}
                    error={error}
                    notice={notice}
                  />
                </div>
              )}
              {route === "details" && (
                <div className="op-stack">
                  <div className="op-detail-backbar">
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={disabled}
                      onClick={() => navigate("sheets")}
                    >
                      <ArrowLeft size={16} />
                      Voltar para planilhas
                    </Button>
                  </div>
                  {detail ? (
                    <SpreadsheetDetails
                      key={`${detail.spreadsheet.spreadsheet_id}:${detailResetKey}`}
                      data={detail}
                      initialTab={initialDetailTab}
                      onTabChange={(tab) => {
                        detailTab.current = tab;
                      }}
                      onClearFeedback={() => {
                        setError("");
                        setNotice("");
                      }}
                      busy={busy}
                      error={error}
                      notice={notice}
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
                          (captured) => {
                            setPreview(captured);
                            setNotice(
                              "Estrutura capturada. Revise o resumo abaixo e marque a confirmação para aprovar.",
                            );
                          },
                          "capturar a estrutura da planilha",
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
                          { context: "aprovar a referência estrutural" },
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
                          "consultar as versões aprovadas",
                        );
                      }}
                      onCompare={(before, after) =>
                        void run(
                          "Comparando referências",
                          () =>
                            api.compareBaselines(detail.spreadsheet.spreadsheet_id, before, after),
                          setComparison,
                          "comparar as versões de referência",
                        )
                      }
                      onComparisonClear={() => setComparison(null)}
                      onSaveTabs={async (ids) => {
                        const saved = await mutate(
                          "Salvando seleção de abas",
                          () => api.saveTabs({ ...mutation(), sheet_ids: ids }),
                          "Seleção de abas salva. Defina as regras de proteção abaixo.",
                          { context: "salvar a seleção de abas" },
                        );
                        if (saved) detailTab.current = "rules";
                        return saved;
                      }}
                      onSaveRules={async (scope) => {
                        const saved = await mutate(
                          "Salvando regras de proteção",
                          () => api.saveRules({ ...mutation(), scope }),
                          "Regras salvas. Capture a estrutura abaixo para revisar e aprovar a referência.",
                          { context: "salvar as regras de proteção" },
                        );
                        if (saved) detailTab.current = "baselines";
                        return saved;
                      }}
                      onSuggestRules={async (headerRow) => {
                        let proposal: import("@/types/integrity").MonitorScope | null = null;
                        await run(
                          "Lendo cabeçalhos para sugerir proteção",
                          () => api.proposeRules({ ...mutation(), header_row: headerRow }),
                          (scope) => {
                            proposal = scope;
                            setNotice(
                              "Cabeçalhos lidos da planilha. Confira as colunas abaixo e clique em Salvar regras de proteção.",
                            );
                          },
                          "ler os cabeçalhos da planilha",
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
                          {
                            context: detail.spreadsheet.paused
                              ? "retomar o cadastro"
                              : "pausar o cadastro",
                          },
                        )
                      }
                      onCheck={() =>
                        void mutate(
                          "Adicionando verificação à fila",
                          () => api.check(mutation()),
                          "Verificação adicionada à fila. O resultado será atualizado pelo serviço.",
                          { context: "agendar a verificação da planilha" },
                        )
                      }
                      onRefresh={() =>
                        void mutate(
                          "Atualizando nome e abas no Google",
                          () => api.refreshTabs(detail.spreadsheet.spreadsheet_id),
                          "Metadados atualizados. Confira abas removidas, renomeadas e regras pendentes.",
                          {
                            resetDetailForms: true,
                            context: "atualizar o nome e as abas no Google",
                          },
                        )
                      }
                      onOpen={() =>
                        void run(
                          "Abrindo arquivo no navegador",
                          () => api.openSpreadsheet(detail.spreadsheet.spreadsheet_id),
                          undefined,
                          "abrir a planilha no navegador",
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
                            await run(
                              "Removendo cadastro local",
                              () => api.remove({ ...mutation(), confirmed: true, history }),
                              undefined,
                              "remover o cadastro da planilha",
                            )
                          ) {
                            selectedIdRef.current = null;
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
                          { context: "associar o projeto Apps Script" },
                        )
                      }
                    />
                  ) : (
                    !busy && (
                      <Card>
                        <EmptyState
                          title="Escolha uma planilha para continuar"
                          description="Abra um cadastro para selecionar abas, configurar proteção, revisar a referência e consultar diagnósticos."
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
                    setExportFeedback(null);
                  }}
                  page={eventPage}
                  rows={rows}
                  disabled={disabled}
                  exportFeedback={exportFeedback}
                  onSearch={() => void loadEvents({ ...eventFilter, offset: 0 })}
                  onExport={(format) => {
                    setExportFeedback(null);
                    void run(
                      "Exportando logs com os filtros selecionados",
                      () =>
                        api.exportEvents({
                          filter: { ...eventFilter, offset: 0 },
                          format,
                        }),
                      (result) => {
                        if (result.exported_count === 0) {
                          setExportFeedback({
                            empty: true,
                            message:
                              "Nenhum registro corresponde aos filtros selecionados para exportação.",
                          });
                        } else {
                          setExportFeedback({
                            empty: false,
                            message: `${result.exported_count} registro(s) exportado(s) em ${result.format.toUpperCase()}: ${result.file_path ?? "pasta local"}`,
                          });
                        }
                      },
                      "exportar o histórico de logs",
                    );
                  }}
                  onPage={(offset) => void loadEvents({ ...eventFilter, offset })}
                  onOpen={setEvent}
                />
              )}
              {route === "settings" && (
                <div className="op-stack">
                  <AccountCard {...accountActions} />
                  <CloudSyncPanel onChanged={() => void refresh()} />
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
                            "salvar as preferências",
                          );
                      }}
                    />
                  )}
                </div>
              )}
            </>
          )}
          <footer className="op-footer" aria-hidden="true" />
        </main>
      </div>
      <EventDiagnostic event={event} onClose={() => setEvent(null)} />
    </div>
  );
}
