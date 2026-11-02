import { useCallback, useEffect, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  Activity,
  ArrowDownLeft,
  ArrowRight,
  ArrowUpRight,
  Bell,
  Check,
  CheckCheck,
  ChevronRight,
  CircleAlert,
  Clock3,
  Database,
  Download,
  FileCode2,
  FileSpreadsheet,
  LayoutDashboard,
  Link2,
  LoaderCircle,
  Plus,
  RefreshCw,
  Search,
  Settings2,
  ShieldCheck,
  Unplug,
  X,
} from "lucide-react";
import { call, desktop } from "./bridge";
import type { BaselinePreview, ChangeEvent, Dashboard, Sheet } from "./bridge";

type Page = "overview" | "history" | "scripts" | "notifications" | "settings";
const labels: Record<Sheet["status"], string> = {
  pending: "Aguardando baseline",
  healthy: "Sem alterações",
  changed: "Alterações detectadas",
  error: "Falha na coleta",
};
const when = (v: string | null) =>
  v
    ? new Intl.DateTimeFormat("pt-BR", {
        day: "2-digit",
        month: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      }).format(new Date(v))
    : "Ainda não verificada";
const value = (v: unknown) =>
  v === undefined ? "Ausente" : JSON.stringify(v, null, 2);
const sectionNames: Record<Page, string> = {
  overview: "Visão geral",
  history: "Histórico de alterações",
  scripts: "Apps Script",
  notifications: "Notificações",
  settings: "Configurações",
};

export default function App() {
  const [data, setData] = useState<Dashboard | null>(null);
  const [page, setPage] = useState<Page>("overview");
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [modal, setModal] = useState<"add" | "baseline" | "remove" | null>(
    null,
  );
  const [selected, setSelected] = useState<Sheet | null>(null);
  const [candidate, setCandidate] = useState<BaselinePreview | null>(null);
  const [approved, setApproved] = useState(false);
  const [event, setEvent] = useState<ChangeEvent | null>(null);
  const [clientId, setClientId] = useState("");
  const [removeName, setRemoveName] = useState("");
  const actionLock = useRef(false);
  const refresh = useCallback(async () => {
    const next = await call<Dashboard>("get_dashboard");
    setData(next);
    return next;
  }, []);
  useEffect(() => {
    void refresh()
      .then((next) => setClientId(next.client_id))
      .catch((e) => setError(String(e)));
    const timer = window.setInterval(() => {
      if (!actionLock.current) void refresh().catch((e) => setError(String(e)));
    }, 10000);
    return () => clearInterval(timer);
  }, [refresh]);
  async function action(work: () => Promise<unknown>, success?: string) {
    if (actionLock.current) return;
    actionLock.current = true;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await work();
      await refresh();
      if (success) setNotice(success);
    } catch (e) {
      setError(String(e));
    } finally {
      actionLock.current = false;
      setBusy(false);
    }
  }
  async function showBaseline(sheet: Sheet) {
    setSelected(sheet);
    setCandidate(null);
    setApproved(false);
    setModal("baseline");
    await action(async () =>
      setCandidate(
        await call<BaselinePreview>("preview_baseline", { id: sheet.id }),
      ),
    );
  }
  async function addSheet(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const form = new FormData(e.currentTarget);
    const rawId = String(form.get("id")).trim();
    const id = rawId.match(/\/spreadsheets\/d\/([a-zA-Z0-9_-]+)/)?.[1] ?? rawId;
    if (!/^[a-zA-Z0-9_-]{3,200}$/.test(id)) {
      setError("Informe um ID de planilha ou um link válido do Google Sheets.");
      return;
    }
    const script_ids = String(form.get("scripts"))
      .split(/[\s,]+/)
      .map((s) => s.trim())
      .filter(Boolean);
    if (script_ids.some((s) => !/^[a-zA-Z0-9_-]{3,200}$/.test(s))) {
      setError(
        "Os IDs de script devem conter apenas letras, números, hífen ou sublinhado.",
      );
      return;
    }
    await action(async () => {
      await call("add_sheet", {
        input: {
          id,
          name: String(form.get("name")).trim(),
          script_ids: [...new Set(script_ids)],
        },
      });
      setModal(null);
    }, "Planilha cadastrada. Revise e aprove a baseline para começar.");
  }
  const sheets = (data?.sheets ?? []).filter(
    (s) =>
      s.name.toLowerCase().includes(query.toLowerCase()) ||
      (s.spreadsheet_id ?? s.id).includes(query),
  );
  const events = (data?.events ?? []).filter(
    (e) =>
      (page !== "scripts" || /script|source|manifest/i.test(e.path)) &&
      (e.sheet_name.toLowerCase().includes(query.toLowerCase()) ||
        e.path.toLowerCase().includes(query.toLowerCase())) &&
      (filter === "all" || e.kind === filter),
  );
  const changed =
    data?.sheets.filter((s) => s.status === "changed").length ?? 0;
  const pending = data?.sheets.filter((s) => !s.baseline_approved).length ?? 0;
  const queued =
    data?.notifications.filter((n) => n.status !== "sent").length ?? 0;
  const navigation: [Page, typeof Activity][] = [
    ["overview", LayoutDashboard],
    ["history", Activity],
    ["scripts", FileCode2],
    ["notifications", Bell],
    ["settings", Settings2],
  ];

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <a
          className="brand"
          href="#"
          onClick={(e) => {
            e.preventDefault();
            setPage("overview");
          }}
        >
          <span className="brand-icon">
            <Activity size={23} />
          </span>
          <span>
            Sheets<span className="brand-light">Monitor</span>
            <small>CONTROLE LOCAL</small>
          </span>
        </a>
        <div className="workspace">
          <span className="workspace-icon">
            <Database size={17} />
          </span>
          <div>
            Meu workspace<small>Armazenamento neste dispositivo</small>
          </div>
          <span className="online-dot" />
        </div>
        <p className="nav-caption">MONITORAMENTO</p>
        <nav aria-label="Navegação principal">
          {navigation.map(([key, Icon]) => (
            <button
              key={key}
              aria-label={sectionNames[key]}
              aria-current={page === key ? "page" : undefined}
              className={`nav-item ${page === key ? "active" : ""}`}
              onClick={() => {
                setPage(key);
                setQuery("");
                setFilter("all");
              }}
            >
              <Icon size={18} />
              <span>{sectionNames[key]}</span>
              {key === "notifications" && queued > 0 && (
                <span className="nav-count">{queued}</span>
              )}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="local-badge">
            <ShieldCheck size={18} />
            <div>
              Seus dados ficam aqui<small>Sem servidor hospedado</small>
            </div>
          </div>
          <div className="version">
            <span>Sheets Monitor</span>
            <span>v0.1.0</span>
          </div>
        </div>
      </aside>
      <main>
        <header className="topbar">
          <div className="breadcrumb">
            Workspace <ChevronRight size={14} />
            <strong>{sectionNames[page]}</strong>
          </div>
          <div className="topbar-right">
            <span
              className={`mode-indicator ${data?.settings.mode === "google" ? "google" : ""}`}
            >
              <span />
              {data?.settings.mode === "google"
                ? "Google · somente leitura"
                : "Fixtures locais"}
            </span>
            <button
              className="icon-button"
              aria-label="Abrir notificações"
              onClick={() => setPage("notifications")}
            >
              <Bell size={19} />
              {queued > 0 && <i />}
            </button>
            <span className="avatar">SM</span>
          </div>
        </header>
        <div className="content">
          {!desktop && (
            <div className="demo-banner">
              <Database size={15} />
              <span>
                Demonstração no navegador. Dados fictícios em memória; nenhuma
                conta Google está conectada.
              </span>
              <span className="demo-tag">DEMO</span>
            </div>
          )}
          {busy && (
            <div className="operation-progress" role="status">
              <LoaderCircle size={14} className="spin" />
              Operação em andamento.{" "}
              {page === "settings"
                ? "Se solicitado, conclua o consentimento no navegador do sistema."
                : "Aguarde a conclusão da coleta local."}
            </div>
          )}
          {error && (
            <div className="alert error" role="alert">
              <CircleAlert size={18} />
              <span>{error}</span>
              <button aria-label="Fechar erro" onClick={() => setError("")}>
                <X size={16} />
              </button>
            </div>
          )}
          {notice && (
            <div className="alert success" role="status">
              <Check size={18} />
              <span>{notice}</span>
              <button
                aria-label="Fechar mensagem"
                onClick={() => setNotice("")}
              >
                <X size={16} />
              </button>
            </div>
          )}
          <div className="page-heading">
            <div>
              <p className="eyebrow">
                {page === "overview"
                  ? "ACOMPANHE CADA MUDANÇA"
                  : "SHEETS MONITOR"}
              </p>
              <h1>{sectionNames[page]}</h1>
              <p className="subtitle">
                {page === "overview"
                  ? "Suas planilhas, scripts e permissões sob acompanhamento."
                  : page === "history"
                    ? "Todas as diferenças coletadas, com o valor anterior e o atual."
                    : page === "scripts"
                      ? "Alterações nos projetos que você autorizou explicitamente."
                      : page === "notifications"
                        ? "Uma entrada persistente para cada diferença detectada."
                        : "Conexão, frequência e modo de coleta deste dispositivo."}
              </p>
            </div>
            <div className="heading-actions">
              {page !== "settings" && (
                <button
                  className="button secondary"
                  disabled={busy || !data}
                  onClick={() =>
                    void action(async () => {
                      const result = await call<{ path: string }>(
                        "export_report",
                      );
                      setNotice(`Relatório salvo em ${result.path}`);
                    })
                  }
                >
                  <Download size={16} />
                  Exportar
                </button>
              )}
              {page === "overview" && (
                <button
                  className="button primary"
                  disabled={busy || !data || data.sheets.length >= 30}
                  onClick={() => {
                    setError("");
                    setModal("add");
                  }}
                >
                  <Plus size={17} />
                  Adicionar planilha
                </button>
              )}
            </div>
          </div>
          {!data ? (
            <div className="empty-state">
              <LoaderCircle className="spin" />
              <h2>Carregando workspace</h2>
              <p>Consultando o armazenamento local.</p>
              <button
                className="button secondary"
                onClick={() => void action(refresh)}
              >
                Tentar novamente
              </button>
            </div>
          ) : (
            <>
              {page === "overview" && (
                <>
                  <section
                    className="stats"
                    aria-label="Resumo do monitoramento"
                  >
                    <Stat
                      label="Planilhas monitoradas"
                      number={data.sheets.length}
                      detail="de 30 disponíveis"
                      icon={<FileSpreadsheet size={20} />}
                    />
                    <Stat
                      label="Com alterações"
                      number={changed}
                      detail="desde a última verificação"
                      icon={<Activity size={20} />}
                      tone="amber"
                    />
                    <Stat
                      label="Aguardando baseline"
                      number={pending}
                      detail="requerem sua aprovação"
                      icon={<ShieldCheck size={20} />}
                      tone="blue"
                    />
                    <Stat
                      label="Notificações na fila"
                      number={queued}
                      detail="entrega acompanhada localmente"
                      icon={<Bell size={20} />}
                    />
                  </section>
                  <section className="panel">
                    <div className="panel-heading">
                      <div>
                        <h2>
                          Planilhas acompanhadas{" "}
                          <span className="count-badge">
                            {data.sheets.length}
                          </span>
                        </h2>
                        <p>
                          Conteúdo, estrutura, permissões e scripts vinculados.
                        </p>
                      </div>
                      <button
                        className="button secondary small"
                        disabled={
                          busy || !data.sheets.some((s) => s.baseline_approved)
                        }
                        onClick={() =>
                          void action(
                            () => call("run_check"),
                            "Verificação solicitada. Consulte o estado de cada planilha.",
                          )
                        }
                      >
                        {busy ? (
                          <LoaderCircle size={15} className="spin" />
                        ) : (
                          <RefreshCw size={15} />
                        )}
                        Verificar agora
                      </button>
                    </div>
                    <div className="table-toolbar">
                      <div className="search">
                        <Search size={16} />
                        <input
                          aria-label="Buscar planilhas"
                          placeholder="Buscar por nome ou ID da planilha..."
                          value={query}
                          onChange={(e) => setQuery(e.target.value)}
                        />
                      </div>
                      <span className="muted text-small">
                        Intervalo: {data.settings.interval_seconds / 60} min
                      </span>
                    </div>
                    <div className="table-scroll">
                      <table>
                        <thead>
                          <tr>
                            <th>PLANILHA</th>
                            <th>STATUS</th>
                            <th>ÚLTIMA VERIFICAÇÃO</th>
                            <th>SCRIPTS</th>
                            <th>
                              <span className="sr-only">Ações</span>
                            </th>
                          </tr>
                        </thead>
                        <tbody>
                          {sheets.map((s) => (
                            <tr key={s.id}>
                              <td>
                                <div className="sheet-name">
                                  <span className="sheet-icon">
                                    <FileSpreadsheet size={20} />
                                  </span>
                                  <div>
                                    <strong>{s.name}</strong>
                                    <small title={s.spreadsheet_id ?? s.id}>
                                      {s.spreadsheet_id ?? s.id}
                                    </small>
                                  </div>
                                </div>
                                {s.last_error && (
                                  <p className="row-error">{s.last_error}</p>
                                )}
                              </td>
                              <td>
                                <span className={`status ${s.status}`}>
                                  <span />
                                  {labels[s.status]}
                                </span>
                              </td>
                              <td className="date-cell">
                                {when(s.last_checked)}
                              </td>
                              <td>
                                <span className="script-count">
                                  <FileCode2 size={14} />
                                  {s.script_ids.length}
                                </span>
                              </td>
                              <td>
                                <div className="row-actions">
                                  {!s.baseline_approved ? (
                                    <button
                                      className="text-button"
                                      disabled={busy}
                                      onClick={() => void showBaseline(s)}
                                    >
                                      Revisar baseline
                                      <ArrowRight size={14} />
                                    </button>
                                  ) : (
                                    <button
                                      className="icon-button"
                                      title={`Verificar ${s.name}`}
                                      aria-label={`Verificar ${s.name}`}
                                      disabled={busy}
                                      onClick={() =>
                                        void action(
                                          () => call("run_check", { id: s.id }),
                                          "Verificação solicitada.",
                                        )
                                      }
                                    >
                                      <RefreshCw size={15} />
                                    </button>
                                  )}
                                  <button
                                    className="icon-button remove"
                                    aria-label={`Remover ${s.name}`}
                                    disabled={busy}
                                    onClick={() => {
                                      setSelected(s);
                                      setRemoveName("");
                                      setModal("remove");
                                    }}
                                  >
                                    <X size={15} />
                                  </button>
                                </div>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                    {sheets.length === 0 && (
                      <Empty
                        icon={<FileSpreadsheet />}
                        title={
                          query
                            ? "Nenhuma planilha encontrada"
                            : "Comece com sua primeira planilha"
                        }
                        description={
                          query
                            ? "Tente outro nome ou ID."
                            : "Adicione uma planilha e aprove sua baseline para iniciar o acompanhamento."
                        }
                      />
                    )}
                    <div className="panel-footer">
                      <span>
                        <span className="online-dot" />
                        {data.settings.mode === "fixture"
                          ? "Fonte de teste local"
                          : data.connected
                            ? "Conexão Google autorizada"
                            : "Google desconectado"}
                      </span>
                      <span>{data.sheets.length} de 30 planilhas</span>
                    </div>
                  </section>
                  <div className="bottom-grid">
                    <section className="panel recent-panel">
                      <div className="panel-heading">
                        <h2>Atividade recente</h2>
                        <button
                          className="text-button"
                          onClick={() => {
                            setPage("history");
                            setQuery("");
                          }}
                        >
                          Ver histórico
                          <ArrowRight size={14} />
                        </button>
                      </div>
                      {data.events.slice(0, 3).map((e) => (
                        <EventRow
                          key={e.id}
                          event={e}
                          onClick={() => setEvent(e)}
                        />
                      ))}
                      {!data.events.length && (
                        <p className="inline-empty">
                          As diferenças aparecerão aqui após a primeira coleta.
                        </p>
                      )}
                    </section>
                    <section className="baseline-note">
                      <span className="note-icon">
                        <ShieldCheck size={24} />
                      </span>
                      <h2>Um ponto de partida confiável</h2>
                      <p>
                        A primeira coleta aguarda sua revisão. O monitoramento
                        começa depois que você aprova a baseline.
                      </p>
                      <div>
                        <Check size={14} />
                        APROVAÇÃO EXPLÍCITA
                      </div>
                    </section>
                  </div>
                </>
              )}
              {(page === "history" || page === "scripts") && (
                <>
                  {page === "scripts" && (
                    <div className="info-strip">
                      <ShieldCheck size={20} />
                      <p>
                        Somente os IDs de script cadastrados são consultados. A
                        análise é estática: os scripts nunca são executados.
                        Revise o conteúdo completo na baseline.
                      </p>
                    </div>
                  )}
                  <section className="panel">
                    <div className="table-toolbar">
                      <div className="search">
                        <Search size={16} />
                        <input
                          aria-label="Buscar alterações"
                          value={query}
                          onChange={(e) => setQuery(e.target.value)}
                          placeholder="Buscar planilha ou caminho alterado..."
                        />
                      </div>
                      <select
                        aria-label="Filtrar tipo de alteração"
                        value={filter}
                        onChange={(e) => setFilter(e.target.value)}
                      >
                        <option value="all">Todos os tipos</option>
                        <option value="changed">Modificações</option>
                        <option value="added">Adições</option>
                        <option value="removed">Remoções</option>
                      </select>
                    </div>
                    {events.map((e) => (
                      <EventRow
                        key={e.id}
                        event={e}
                        onClick={() => setEvent(e)}
                      />
                    ))}
                    {!events.length && (
                      <Empty
                        icon={page === "scripts" ? <FileCode2 /> : <Activity />}
                        title="Nenhuma alteração nesta seleção"
                        description="A lista mostra as diferenças preservadas no histórico local."
                      />
                    )}
                    <div className="panel-footer">
                      <span>{events.length} diferenças nesta seleção</span>
                      <span>Clique em uma alteração para comparar</span>
                    </div>
                  </section>
                  {page === "scripts" && (
                    <section className="panel scripts-list">
                      <div className="panel-heading">
                        <h2>Projetos autorizados</h2>
                      </div>
                      {data.sheets
                        .filter((s) => s.script_ids.length)
                        .map((s) => (
                          <div className="project-row" key={s.id}>
                            <FileCode2 size={20} />
                            <div>
                              <strong>{s.name}</strong>
                              {s.script_ids.map((id) => (
                                <code key={id}>{id}</code>
                              ))}
                            </div>
                            <span className="status healthy">
                              Somente leitura
                            </span>
                          </div>
                        ))}
                      {!data.sheets.some((s) => s.script_ids.length) && (
                        <p className="inline-empty">
                          Nenhum projeto vinculado. Informe os IDs ao cadastrar
                          uma planilha.
                        </p>
                      )}
                    </section>
                  )}
                </>
              )}
              {page === "notifications" && (
                <section className="panel">
                  <div className="panel-heading">
                    <div>
                      <h2>
                        Fila de entrega{" "}
                        <span className="count-badge">{queued}</span>
                      </h2>
                      <p>
                        As diferenças ficam no histórico mesmo se a notificação
                        falhar.
                      </p>
                    </div>
                    <button
                      className="button secondary small"
                      disabled={busy || queued === 0}
                      onClick={() =>
                        void action(
                          () => call("retry_notifications"),
                          "Fila de notificações processada.",
                        )
                      }
                    >
                      <RefreshCw size={15} />
                      Reprocessar fila
                    </button>
                  </div>
                  {data.notifications.map((n) => {
                    const e = data.events.find((e) => e.id === n.event_id);
                    return (
                      <div className="notification-row" key={n.id}>
                        <span className={`notification-icon ${n.status}`}>
                          {n.status === "sent" ? (
                            <CheckCheck size={19} />
                          ) : (
                            <Bell size={19} />
                          )}
                        </span>
                        <div>
                          <strong>
                            {e?.sheet_name ?? "Alteração registrada"}
                          </strong>
                          <p>{e?.path ?? `Evento ${n.event_id}`}</p>
                          <small>{n.attempts} tentativa(s)</small>
                        </div>
                        <span
                          className={`status ${n.status === "sent" ? "healthy" : n.status === "failed" ? "error" : "pending"}`}
                        >
                          {n.status === "sent"
                            ? "Enviada"
                            : n.status === "failed"
                              ? "Falhou · será repetida"
                              : "Pendente"}
                        </span>
                      </div>
                    );
                  })}
                  {!data.notifications.length && (
                    <Empty
                      icon={<Bell />}
                      title="A fila está vazia"
                      description="Cada diferença detectada gera uma entrada de notificação."
                    />
                  )}
                </section>
              )}
              {page === "settings" && (
                <div className="settings-grid">
                  <section className="panel settings-panel">
                    <div className="settings-title">
                      <span className="setting-icon">
                        <Link2 size={21} />
                      </span>
                      <div>
                        <h2>Conexão Google</h2>
                        <p>OAuth 2.0 desktop com PKCE</p>
                      </div>
                      <span
                        className={`status ${data.connected ? "healthy" : "pending"}`}
                      >
                        {data.connected ? "Autorizada" : "Desconectada"}
                      </span>
                    </div>
                    <p className="settings-description">
                      Use um cliente OAuth do tipo aplicativo para computador. A
                      autorização abre no navegador do sistema e os tokens ficam
                      no cofre do Windows.
                    </p>
                    <label className="field-label" htmlFor="client-id">
                      Client ID do Google Cloud
                    </label>
                    <input
                      id="client-id"
                      placeholder="…apps.googleusercontent.com"
                      value={clientId}
                      onChange={(e) => setClientId(e.target.value)}
                      autoComplete="off"
                      spellCheck={false}
                    />
                    <div className="settings-actions">
                      <button
                        className="button primary"
                        disabled={busy || !clientId.trim() || data.connected}
                        onClick={() =>
                          void action(
                            () =>
                              call("connect_google", {
                                clientId: clientId.trim(),
                              }),
                            "Autorização Google concluída.",
                          )
                        }
                      >
                        <Link2 size={16} />
                        Conectar Google
                      </button>
                      <button
                        className="button secondary"
                        disabled={busy || !data.connected}
                        onClick={() =>
                          void action(
                            () => call("disconnect_google"),
                            "Conta desconectada deste dispositivo.",
                          )
                        }
                      >
                        <Unplug size={16} />
                        Desconectar
                      </button>
                    </div>
                    <p className="field-help">
                      Nenhuma célula, permissão ou código será alterado no
                      Google.
                    </p>
                  </section>
                  <section className="panel settings-panel">
                    <div className="settings-title">
                      <span className="setting-icon">
                        <Clock3 size={21} />
                      </span>
                      <div>
                        <h2>Coleta periódica</h2>
                        <p>Enquanto o aplicativo estiver em execução</p>
                      </div>
                    </div>
                    <label className="field-label" htmlFor="mode">
                      Fonte de dados
                    </label>
                    <select
                      id="mode"
                      value={data.settings.mode}
                      disabled={busy}
                      onChange={(e) =>
                        void action(
                          () =>
                            call("update_settings", {
                              settings: {
                                ...data.settings,
                                mode: e.target.value,
                              },
                            }),
                          "Fonte de dados atualizada.",
                        )
                      }
                    >
                      <option value="fixture">Fixtures locais · testes</option>
                      <option value="google">Google · dados reais</option>
                    </select>
                    <p className="field-help">
                      Os modos mantêm históricos e baselines separados.
                    </p>
                    <label className="field-label" htmlFor="interval">
                      Verificar a cada
                    </label>
                    <select
                      id="interval"
                      value={data.settings.interval_seconds}
                      disabled={busy}
                      onChange={(e) =>
                        void action(
                          () =>
                            call("update_settings", {
                              settings: {
                                ...data.settings,
                                interval_seconds: Number(e.target.value),
                              },
                            }),
                          "Intervalo atualizado.",
                        )
                      }
                    >
                      <option value={60}>1 minuto</option>
                      <option value={300}>5 minutos</option>
                      <option value={900}>15 minutos</option>
                      <option value={1800}>30 minutos</option>
                    </select>
                  </section>
                  <section className="privacy-card">
                    <ShieldCheck size={23} />
                    <div>
                      <h2>Operação local, acesso de leitura</h2>
                      <p>
                        SQLite armazena snapshots e histórico neste computador.
                        A coleta pode incluir valores, fórmulas, metadados,
                        permissões e fontes de scripts autorizados. Exporte
                        relatórios somente para destinos confiáveis.
                      </p>
                    </div>
                  </section>
                </div>
              )}
            </>
          )}
          <footer className="app-footer">
            <span>
              <ShieldCheck size={13} />
              Local por natureza. Visível em cada mudança.
            </span>
            <span>Sheets · Drive · Apps Script</span>
          </footer>
        </div>
      </main>
      {modal && (
        <Modal
          title={
            modal === "add"
              ? "Adicionar planilha"
              : modal === "baseline"
                ? "Revisar baseline"
                : "Remover planilha"
          }
          onClose={() => {
            if (!busy) setModal(null);
          }}
          wide={modal === "baseline"}
        >
          {error && (
            <div className="alert error" role="alert">
              <CircleAlert size={16} />
              <span>{error}</span>
            </div>
          )}
          {modal === "add" && (
            <form onSubmit={(e) => void addSheet(e)}>
              <p className="modal-description">
                Cadastre a fonte e os projetos que você autoriza a consultar. A
                baseline será aprovada em uma etapa separada.
              </p>
              <label className="field-label" htmlFor="sheet-name">
                Nome da planilha
              </label>
              <input
                id="sheet-name"
                name="name"
                required
                maxLength={100}
                autoFocus
                placeholder="Ex.: Indicadores comerciais"
              />
              <label className="field-label" htmlFor="sheet-id">
                Link ou ID do Google Sheets
              </label>
              <input
                id="sheet-id"
                name="id"
                required
                maxLength={500}
                placeholder="https://docs.google.com/spreadsheets/d/…"
              />
              <label className="field-label" htmlFor="script-ids">
                IDs de Apps Script autorizados{" "}
                <span className="muted">(opcional)</span>
              </label>
              <textarea
                id="script-ids"
                name="scripts"
                rows={3}
                placeholder="Um ID por linha ou separado por vírgula"
              />
              <p className="field-help">
                Os projetos não são descobertos automaticamente. Você pode obter
                o ID nas configurações do projeto Apps Script.
              </p>
              <label className="checkbox-label">
                <input type="checkbox" required />
                Autorizo a consulta somente de leitura desta planilha e dos
                scripts informados.
              </label>
              <div className="modal-actions">
                <button
                  type="button"
                  className="button secondary"
                  disabled={busy}
                  onClick={() => setModal(null)}
                >
                  Cancelar
                </button>
                <button className="button primary" disabled={busy}>
                  {busy && <LoaderCircle size={16} className="spin" />}Cadastrar
                  planilha
                </button>
              </div>
            </form>
          )}
          {modal === "baseline" && (
            <>
              <p className="modal-description">
                Revise a coleta de <strong>{selected?.name}</strong>. Este
                conteúdo será o ponto de partida para as comparações seguintes.
              </p>
              {candidate ? (
                <>
                  <div className="preview-caption">
                    <Database size={15} />
                    Snapshot da coleta ·{" "}
                    {data?.settings.mode === "fixture"
                      ? "fixture local"
                      : "Google"}
                  </div>
                  <pre className="baseline-preview" tabIndex={0}>
                    {value(candidate.snapshot)}
                  </pre>
                  <label className="checkbox-label">
                    <input
                      type="checkbox"
                      checked={approved}
                      onChange={(e) => setApproved(e.target.checked)}
                    />
                    Revisei esta coleta e aprovo seu uso como baseline.
                  </label>
                  <div className="modal-actions">
                    <button
                      className="button secondary"
                      disabled={busy}
                      onClick={() => setModal(null)}
                    >
                      Cancelar
                    </button>
                    <button
                      className="button primary"
                      disabled={!approved || busy}
                      onClick={() =>
                        void action(async () => {
                          await call("approve_baseline", {
                            id: selected?.id,
                            fingerprint: candidate.fingerprint,
                          });
                          setModal(null);
                        }, "Baseline aprovada. O monitoramento pode comparar as próximas coletas.")
                      }
                    >
                      <ShieldCheck size={16} />
                      Aprovar baseline
                    </button>
                  </div>
                </>
              ) : (
                <div className="baseline-loading">
                  {busy ? (
                    <>
                      <LoaderCircle className="spin" />
                      <p>Coletando prévia…</p>
                    </>
                  ) : (
                    <>
                      <CircleAlert />
                      <p>Não foi possível obter a prévia.</p>
                      <button
                        className="button secondary"
                        onClick={() => selected && void showBaseline(selected)}
                      >
                        Tentar novamente
                      </button>
                    </>
                  )}
                </div>
              )}
            </>
          )}
          {modal === "remove" && (
            <>
              <p className="modal-description">
                A planilha <strong>{selected?.name}</strong> deixará de ser
                monitorada. Nenhum arquivo será alterado no Google.
              </p>
              <label className="field-label" htmlFor="confirm-remove">
                Digite o nome da planilha para confirmar
              </label>
              <input
                id="confirm-remove"
                value={removeName}
                onChange={(e) => setRemoveName(e.target.value)}
                autoFocus
              />
              <div className="modal-actions">
                <button
                  className="button secondary"
                  disabled={busy}
                  onClick={() => setModal(null)}
                >
                  Cancelar
                </button>
                <button
                  className="button danger"
                  disabled={busy || removeName !== selected?.name}
                  onClick={() =>
                    void action(async () => {
                      await call("remove_sheet", { id: selected?.id });
                      setModal(null);
                    }, "Planilha removida do acompanhamento.")
                  }
                >
                  Remover acompanhamento
                </button>
              </div>
            </>
          )}
        </Modal>
      )}
      {event && (
        <Modal
          title="Detalhes da alteração"
          onClose={() => setEvent(null)}
          wide
        >
          <div className="event-detail-heading">
            <strong>{event.sheet_name}</strong>
            <span className="muted">{when(event.detected_at)}</span>
          </div>
          <code className="event-path">{event.path}</code>
          <div className="diff-grid">
            <div>
              <span className="diff-label before">ANTERIOR</span>
              <pre tabIndex={0}>{value(event.before)}</pre>
            </div>
            <div>
              <span className="diff-label after">ATUAL</span>
              <pre tabIndex={0}>{value(event.after)}</pre>
            </div>
          </div>
          <div className="modal-actions">
            <button className="button primary" onClick={() => setEvent(null)}>
              Fechar comparação
            </button>
          </div>
        </Modal>
      )}
    </div>
  );
}
function Stat({
  label,
  number,
  detail,
  icon,
  tone = "green",
}: {
  label: string;
  number: number;
  detail: string;
  icon: React.ReactNode;
  tone?: string;
}) {
  return (
    <article className="stat-card">
      <div className="stat-top">
        <span>{label}</span>
        <span className={`stat-icon ${tone}`}>{icon}</span>
      </div>
      <strong>{number.toString().padStart(2, "0")}</strong>
      <p>{detail}</p>
    </article>
  );
}
function EventRow({
  event,
  onClick,
}: {
  event: ChangeEvent;
  onClick: () => void;
}) {
  return (
    <button className="event-row" onClick={onClick}>
      <span className={`event-icon ${event.kind}`}>
        {event.kind === "removed" ? (
          <ArrowDownLeft size={17} />
        ) : event.kind === "added" ? (
          <Plus size={17} />
        ) : (
          <ArrowUpRight size={17} />
        )}
      </span>
      <div>
        <strong>{event.sheet_name}</strong>
        <p>
          {/script/i.test(event.path) ? "Apps Script" : "Conteúdo ou metadados"}{" "}
          <span>·</span>{" "}
          {event.kind === "added"
            ? "Adição"
            : event.kind === "removed"
              ? "Remoção"
              : "Modificação"}
        </p>
        <code>{event.path}</code>
      </div>
      <time>{when(event.detected_at)}</time>
      <ChevronRight size={16} />
    </button>
  );
}
function Empty({
  icon,
  title,
  description,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="empty-state">
      <span className="empty-icon">{icon}</span>
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  );
}
function Modal({
  title,
  children,
  onClose,
  wide = false,
}: {
  title: string;
  children: React.ReactNode;
  onClose: () => void;
  wide?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const close = useRef(onClose);
  close.current = onClose;
  useEffect(() => {
    const prev = document.activeElement as HTMLElement | null;
    ref.current?.focus();
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") close.current();
      if (e.key === "Tab") {
        const nodes = ref.current?.querySelectorAll<HTMLElement>(
          'button:not(:disabled),input:not(:disabled),textarea,select,[tabindex="0"]',
        );
        if (!nodes?.length) return;
        const first = nodes[0],
          last = nodes[nodes.length - 1];
        if (
          e.shiftKey &&
          (document.activeElement === first ||
            document.activeElement === ref.current)
        ) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", handler);
    return () => {
      document.removeEventListener("keydown", handler);
      prev?.focus();
    };
  }, []);
  return (
    <div className="modal-overlay">
      <div
        className={`modal ${wide ? "wide" : ""}`}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        ref={ref}
      >
        <div className="modal-heading">
          <h2>{title}</h2>
          <button
            className="icon-button"
            aria-label="Fechar diálogo"
            onClick={onClose}
          >
            <X size={20} />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
