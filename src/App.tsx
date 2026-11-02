import { useState } from "react";
import {
  Activity,
  ArrowRight,
  Bell,
  CheckCircle2,
  ChevronRight,
  CircleAlert,
  Database,
  FileSpreadsheet,
  FlaskConical,
  History,
  Info,
  LayoutDashboard,
  LoaderCircle,
  LockKeyhole,
  RefreshCw,
  Settings2,
  ShieldCheck,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import { desktop } from "@/services/foundation";
import { baselineExample } from "@/fixtures";
import { useFoundation } from "@/hooks/useFoundation";
import { formatDate, categoryNames } from "@/lib/presentation";
import type { Page, DemoSpreadsheet, DemoActivity } from "@/types/foundation";
import { DashboardPage } from "@/pages/DashboardPage";
import { SpreadsheetsPage } from "@/pages/SpreadsheetsPage";
import { HistoryPage } from "@/pages/HistoryPage";
import { GooglePage } from "@/pages/GooglePage";
import { SettingsPage } from "@/pages/SettingsPage";
const pages = [
  { key: "google", label: "Integração Google", icon: LockKeyhole },
  { key: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { key: "sheets", label: "Planilhas", icon: FileSpreadsheet },
  { key: "history", label: "Histórico", icon: History },
  { key: "settings", label: "Configurações", icon: Settings2 },
] as const;
export default function App() {
  const { data, settings, setSettings, busy, error, notice, setNotice, refresh, submitSettings } =
    useFoundation();
  const [page, setPage] = useState<Page>("dashboard");
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const [about, setAbout] = useState(false);
  const [baseline, setBaseline] = useState<DemoSpreadsheet | null>(null);
  const [detail, setDetail] = useState<DemoActivity | null>(null);
  function navigate(next: Page) {
    setPage(next);
    setQuery("");
    setFilter("all");
    setNotice("");
  }
  const current = pages.find((item) => item.key === page)!;
  const visibleSheets = (data?.spreadsheets ?? []).filter(
    (sheet) =>
      `${sheet.name} ${sheet.id}`.toLowerCase().includes(query.toLowerCase()) &&
      (filter === "all" || sheet.status === filter),
  );
  const visibleEvents = (data?.activities ?? []).filter(
    (event) =>
      `${event.spreadsheet_name} ${event.diagnosis}`.toLowerCase().includes(query.toLowerCase()) &&
      (filter === "all" || event.category === filter),
  );
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <a
          className="brand"
          href="#dashboard"
          onClick={(e) => {
            e.preventDefault();
            navigate("dashboard");
          }}
          aria-label="Sheets Monitor — Dashboard"
        >
          <span className="brand-symbol">
            <Activity size={25} />
          </span>
          <span className="brand-name">
            Sheets<span>Monitor</span>
            <small>SEU WORKSPACE LOCAL</small>
          </span>
        </a>
        <div className="workspace">
          <span className="workspace-symbol">
            <Database size={17} />
          </span>
          <div>
            Meu workspace<small>Fundação do aplicativo</small>
          </div>
          <span className="workspace-dot" />
        </div>
        <p className="nav-label">ACOMPANHAMENTO</p>
        <nav aria-label="Navegação principal">
          {pages.map((item) => (
            <button
              key={item.key}
              aria-label={item.label}
              aria-current={page === item.key ? "page" : undefined}
              onClick={() => navigate(item.key)}
              className={cn("nav-item", page === item.key && "active")}
            >
              <item.icon size={18} />
              <span>{item.label}</span>
              {item.key === "sheets" && <small>{data?.spreadsheets.length ?? 0}</small>}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">
          <div className="local-promise">
            <ShieldCheck size={19} />
            <div>
              Local por natureza<small>Sem servidor próprio</small>
            </div>
          </div>
          <div className="version">
            <span>SPRINT 02 · INTEGRAÇÕES</span>
            <span>v0.1.0</span>
          </div>
        </div>
      </aside>
      <main className="main-shell">
        <header className="topbar">
          <div className="breadcrumbs">
            <span>Workspace</span>
            <ChevronRight size={14} />
            <strong>{current.label}</strong>
          </div>
          <div className="topbar-tools">
            {page !== "google" && (
              <Badge variant="demo">
                <FlaskConical size={12} />
                DEMONSTRAÇÃO
              </Badge>
            )}
            <Button
              variant="ghost"
              size="icon"
              aria-label="Ver histórico demonstrativo"
              onClick={() => navigate("history")}
            >
              <Bell size={18} />
            </Button>
            <span className="avatar">SM</span>
          </div>
        </header>
        <div className="main-content">
          {page !== "google" && (
            <div className="demo-banner">
              <FlaskConical size={16} />
              <p>
                <strong>Dados fictícios · Sprint 01.</strong> Este workspace usa fixtures locais.
                Estados, histórico e baseline são exemplos.
              </p>
              <button onClick={() => setAbout(true)}>
                Entender esta etapa
                <ArrowRight size={13} />
              </button>
            </div>
          )}
          {page !== "google" && error && (
            <div role="alert" className="feedback error">
              <CircleAlert size={17} />
              <span>{error}</span>
            </div>
          )}
          {notice && (
            <div role="status" className="feedback success">
              <CheckCircle2 size={17} />
              <span>{notice}</span>
            </div>
          )}
          {page !== "google" && (
            <div className="page-title">
              <div>
                <p className="eyebrow">
                  {page === "dashboard" ? "CLAREZA PARA CADA PLANILHA" : "SHEETS MONITOR"}
                </p>
                <h1>{current.label}</h1>
                <p>
                  {page === "dashboard"
                    ? "Um olhar sobre a integridade das suas bases e painéis."
                    : page === "sheets"
                      ? "Conheça a organização das fontes no workspace demonstrativo."
                      : page === "history"
                        ? "Eventos de exemplo com data, arquivo e diagnóstico."
                        : "Preferências locais e os limites desta primeira etapa."}
                </p>
              </div>
              <div className="heading-actions">
                <Button variant="outline" size="sm" onClick={() => setAbout(true)}>
                  <Info />
                  Sobre esta sprint
                </Button>
                {page !== "settings" && (
                  <Button size="sm" disabled={busy} onClick={() => void refresh()}>
                    <RefreshCw className={cn(busy && "animate-spin")} />
                    Atualizar demonstração
                  </Button>
                )}
              </div>
            </div>
          )}
          {page === "google" ? (
            <GooglePage />
          ) : !data ? (
            <Card className="loading-card">
              <LoaderCircle className="animate-spin text-primary" />
              <h2>Carregando workspace demonstrativo</h2>
              <p>Consultando {desktop ? "o banco SQLite local" : "as fixtures em memória"}.</p>
              {error && (
                <Button variant="outline" onClick={() => void refresh()}>
                  Tentar novamente
                </Button>
              )}
            </Card>
          ) : (
            <>
              {page === "dashboard" && (
                <DashboardPage
                  data={data}
                  setBaseline={setBaseline}
                  setDetail={setDetail}
                  navigate={navigate}
                />
              )}
              {page === "sheets" && (
                <SpreadsheetsPage
                  data={data}
                  setBaseline={setBaseline}
                  query={query}
                  setQuery={setQuery}
                  filter={filter}
                  setFilter={setFilter}
                  visibleSheets={visibleSheets}
                />
              )}
              {page === "history" && (
                <HistoryPage
                  data={data}
                  setDetail={setDetail}
                  query={query}
                  setQuery={setQuery}
                  filter={filter}
                  setFilter={setFilter}
                  visibleEvents={visibleEvents}
                />
              )}
              {page === "settings" && (
                <SettingsPage
                  data={data}
                  settings={settings}
                  setSettings={setSettings}
                  busy={busy}
                  submitSettings={submitSettings}
                />
              )}
            </>
          )}
          <footer className="app-footer">
            <span>
              <ShieldCheck size={13} />
              {page === "google"
                ? "Consultas Google autorizadas: somente leitura"
                : "Fundação local. Dados de demonstração."}
            </span>
            <span>Sheets Monitor · Sprint 02</span>
          </footer>
        </div>
      </main>
      <Dialog open={about} onOpenChange={setAbout}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Sprint 02 · Integração Google</DialogTitle>
            <DialogDescription>
              A área Integração Google realiza consultas manuais autorizadas. Dashboard, planilhas e
              histórico ainda usam dados fictícios.
            </DialogDescription>
          </DialogHeader>
          <div className="scope-list">
            <div>
              <CheckCircle2 />
              <p>
                <strong>Nesta etapa</strong>OAuth Desktop com PKCE, cofre Windows e leitores de
                Sheets, Drive e Apps Script.
              </p>
            </div>
            <div>
              <LockKeyhole />
              <p>
                <strong>Nas próximas sprints</strong>Baseline aprovada, motor de integridade,
                análise de Apps Script e alertas de produção.
              </p>
            </div>
          </div>
          <p className="scope-footer">
            Os estados de integridade são demonstrativos. A homologação Google real depende de
            credenciais e consentimento.
          </p>
          <Button onClick={() => setAbout(false)}>Entendi</Button>
        </DialogContent>
      </Dialog>
      <Dialog
        open={baseline !== null}
        onOpenChange={(open) => {
          if (!open) setBaseline(null);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Exemplo de baseline</DialogTitle>
            <DialogDescription>
              Prévia didática para {baseline?.name}. Não foi coletada do Google e não representa
              aprovação de baseline.
            </DialogDescription>
          </DialogHeader>
          <Badge variant="demo" className="w-fit">
            FIXTURE · SEM APROVAÇÃO REAL
          </Badge>
          <pre className="json-preview" tabIndex={0}>
            {JSON.stringify({ ...baselineExample, spreadsheet: baseline?.name }, null, 2)}
          </pre>
          <div className="baseline-info">
            <Info size={17} />
            <p>A baseline versionada e sua aprovação explícita pertencem à Sprint 03.</p>
          </div>
          <Button onClick={() => setBaseline(null)}>Fechar exemplo</Button>
        </DialogContent>
      </Dialog>
      <Dialog
        open={detail !== null}
        onOpenChange={(open) => {
          if (!open) setDetail(null);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Evento demonstrativo</DialogTitle>
            <DialogDescription>
              Registro fictício para ilustrar o histórico. Nenhuma alteração real foi observada.
            </DialogDescription>
          </DialogHeader>
          {detail && (
            <>
              <div className="detail-meta">
                <strong>{detail.spreadsheet_name}</strong>
                <span>{formatDate(detail.occurred_at)}</span>
              </div>
              <Badge variant="demo" className="w-fit">
                {categoryNames[detail.category] ?? detail.category} · DEMO
              </Badge>
              <p className="diagnosis">{detail.diagnosis}</p>
              <Button onClick={() => setDetail(null)}>Fechar evento</Button>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
