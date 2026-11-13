import { errorGuidance } from "@/services/google";
import { AuditDetails } from "./AuditDetails";
import { FileSpreadsheet, ShieldCheck, CircleAlert, CircleHelp, Pause, Clock3 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { managementError } from "@/services/management";
import type { Activity, DashboardSummary, RegisteredSpreadsheet } from "@/types/management";

export const statusNames: Record<RegisteredSpreadsheet["status"] | "checking", string> = {
  configuration_pending: "Configuração pendente",
  not_checked: "Ainda não verificada",
  online: "Online",
  offline: "Offline",
  indeterminate: "Indeterminado",
  paused: "Pausada",
  checking: "Verificando",
};

export function formatLocalDateTime(value: string | number | null | undefined) {
  if (value === null || value === undefined || value === "") return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(date.getDate())}/${pad(date.getMonth() + 1)}/${date.getFullYear()} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

export function dateLabel(value: string | null | undefined) {
  if (!value) return "Ainda não verificada";
  return formatLocalDateTime(value) || "Data indisponível";
}

export function StatusBadge({ status }: { status: RegisteredSpreadsheet["status"] | "checking" }) {
  return (
    <Badge variant="outline" className={`op-status op-status-${status}`}>
      <span aria-hidden="true" />
      {statusNames[status]}
    </Badge>
  );
}

export function EmptyState({
  title,
  description,
  action,
  onAction,
  disabled = false,
}: {
  title: string;
  description: string;
  action?: string;
  onAction?: () => void;
  disabled?: boolean;
}) {
  return (
    <div className="op-empty-state">
      <div className="op-empty-icon">
        <FileSpreadsheet size={25} />
      </div>
      <h2>{title}</h2>
      <p>{description}</p>
      {action && onAction && (
        <Button onClick={onAction} disabled={disabled}>
          {action}
        </Button>
      )}
    </div>
  );
}

export function SummaryCards({ data }: { data: DashboardSummary }) {
  const metrics = [
    {
      label: "Cadastradas",
      value: data.registered,
      icon: FileSpreadsheet,
      detail: "Até 30 planilhas",
    },
    { label: "Online", value: data.online, icon: ShieldCheck, detail: "Integridade confirmada" },
    {
      label: "Offline",
      value: data.offline,
      icon: CircleAlert,
      detail: "Falha estrutural confirmada",
    },
    {
      label: "Indeterminadas",
      value: data.indeterminate,
      icon: CircleHelp,
      detail: "Requerem diagnóstico",
    },
    { label: "Pausadas", value: data.paused, icon: Pause, detail: "Análise suspensa" },
    {
      label: "A configurar / verificar",
      value: data.pending + data.not_checked,
      icon: Clock3,
      detail: "Sem estado Online presumido",
    },
  ];
  return (
    <div className="op-stats">
      {metrics.map(({ label, value, icon: Icon, detail }) => (
        <Card className="op-stat" key={label}>
          <div>
            <span>{label}</span>
            <Icon size={17} />
          </div>
          <strong>{value}</strong>
          <p>{detail}</p>
        </Card>
      ))}
    </div>
  );
}

const categories: Record<string, string> = {
  integrity: "Integridade",
  content: "Conteúdo",
  structure: "Estrutura",
  script: "Apps Script",
  operational: "Operacional",
};

const eventKinds: Record<string, string> = {
  violation: "Violação estrutural",
  inconclusive: "Análise inconclusiva",
  analysis_inconclusive: "Análise inconclusiva",
  failed: "Falha operacional",
  recovered: "Recuperação",
  analysis_concluded: "Análise concluída",
  value_changed: "Valor alterado",
  record_added: "Registro adicionado",
  record_removed: "Registro removido",
  record_order_changed: "Ordem de registros alterada",
  header_changed: "Cabeçalho alterado",
  region_changed: "Conteúdo da região alterado",
  script_code_changed: "Código de script alterado",
  script_file_added: "Arquivo de script adicionado",
  script_file_removed: "Arquivo de script removido",
  script_file_changed: "Arquivo de script alterado",
  sheet_title_changed: "Nome da aba alterado",
  sheet_position_changed: "Posição da aba alterada",
};

const connectivityNames: Record<string, string> = {
  available: "Disponível na captura",
  online: "Disponível na captura",
  reachable: "Disponível na captura",
  degraded: "Parcialmente disponível",
  unavailable: "Indisponível na captura",
  offline: "Sem conexão na captura",
};

const diagnoses: Record<string, string> = {
  protected_record_removed: "Um registro da referência foi removido contra a regra de preservação.",
  record_deletion_unproven:
    "Não foi possível comprovar a preservação dos registros sem uma chave única estável.",
  metadata_not_observed: "Os metadados da planilha não puderam ser consultados no momento.",
  capture_incomplete: "A captura não conseguiu ler todas as regiões ou abas exigidas pelas regras.",
  monitored_sheet_missing: "Uma aba protegida não foi encontrada na planilha.",
  sheet_not_observed: "Não foi possível consultar os dados da aba protegida.",
  protected_sheet_renamed: "O nome de uma aba protegida foi alterado.",
  protected_sheet_reordered: "A posição de uma aba protegida foi alterada.",
  protected_column_missing: "Uma coluna obrigatória não foi encontrada na região protegida.",
  protected_header_changed_or_column_missing:
    "Um cabeçalho protegido foi alterado ou sua coluna foi removida.",
  protected_column_reordered: "A posição de uma coluna protegida mudou.",
  column_identity_ambiguous:
    "Há cabeçalhos duplicados; a coluna não pôde ser identificada com segurança.",
  region_not_observed: "Não foi possível consultar os dados da região protegida.",
  protected_region_displaced: "Uma região protegida foi deslocada na aba.",
  named_range_changed: "Um intervalo nomeado protegido foi alterado.",
  named_range_missing: "Um intervalo nomeado protegido não foi encontrado.",
  named_range_not_observed: "Não foi possível verificar o intervalo nomeado protegido.",
  script_not_observed: "Não foi possível consultar o projeto Apps Script associado.",
  script_analysis_incomplete:
    "Há dependências dinâmicas no Apps Script que não podem ser verificadas estaticamente.",
  protected_formula_changed: "Uma fórmula protegida foi alterada.",
  protected_formula_removed_or_replaced:
    "Uma fórmula protegida foi removida ou substituída por valor fixo.",
  formula_equivalence_unproven:
    "Não foi possível comprovar que a fórmula manteve sua estrutura esperada.",
  formula_sheet_reference_missing: "A fórmula referencia uma aba que não foi encontrada.",
  formula_effective_error: "Uma fórmula protegida retornou erro de cálculo na planilha.",
  formula_loading:
    "O Google Sheets ainda estava calculando uma fórmula protegida durante a leitura.",
  formula_anchor_ambiguous: "O registro da fórmula não pôde ser identificado com segurança.",
  formula_column_or_key_not_resolved:
    "A coluna ou chave associada à fórmula não pôde ser identificada.",
  fixed_formula_anchor_changed: "A linha ou coluna fixa de uma fórmula protegida mudou.",
  formula_cell_not_observed: "A célula da fórmula protegida não pôde ser consultada.",
  baseline_formula_not_observed:
    "A referência atual não contém a fórmula necessária para comparação.",
  header_changed: "Um cabeçalho foi alterado.",
  region_changed: "O conteúdo de uma região foi alterado.",
  record_added: "Um registro foi adicionado.",
  record_removed: "Um registro foi removido.",
  record_order_changed: "A ordem dos registros mudou.",
  value_changed: "Um valor foi alterado.",
  content_changed_unspecified: "O conteúdo mudou; a operação exata não pôde ser determinada.",
  content_changed_operation_ambiguous:
    "O conteúdo mudou; a operação exata não pôde ser determinada.",
  script_file_added: "Um arquivo foi adicionado ao projeto Apps Script.",
  script_file_removed: "Um arquivo foi removido do projeto Apps Script.",
  script_file_changed: "Um arquivo do projeto Apps Script foi alterado.",
  script_code_changed: "O código do projeto Apps Script mudou.",
  sheet_title_changed: "O nome de uma aba foi alterado.",
  sheet_position_changed: "A posição de uma aba mudou.",
  critical_rule_recovered: "A regra crítica voltou a atender à referência aprovada.",
  analysis_concluded: "A análise anteriormente inconclusiva foi concluída com sucesso.",
  operational_recovered: "A comunicação com o Google foi restabelecida.",
  network: "Falha de conexão ao consultar os serviços do Google.",
  timeout: "A consulta excedeu o tempo limite de resposta.",
  rate_limited: "Limite temporário de consultas da API Google atingido.",
  forbidden: "Sem permissão de leitura no arquivo ou projeto Google.",
  not_found: "O recurso não foi encontrado ou deixou de estar compartilhado com a conta.",
  insufficient_scope: "Permissão de leitura pendente na conta Google conectada.",
  reconnect_required: "A sessão Google expirou e requer nova autorização.",
  authentication_required: "Autenticação Google pendente para executar a verificação.",
  local_storage: "Falha ao acessar o armazenamento local durante a verificação.",
  server: "Serviço Google temporariamente indisponível durante a leitura.",
};

const pluralDiagnoses: Record<string, (count: number) => string> = {
  value_changed: (n) => `${n} valores foram alterados nesta verificação.`,
  record_added: (n) => `${n} registros foram adicionados nesta verificação.`,
  record_removed: (n) => `${n} registros foram removidos nesta verificação.`,
  record_order_changed: (n) => `A ordem de ${n} conjuntos de registros mudou nesta verificação.`,
  header_changed: (n) => `${n} cabeçalhos foram alterados nesta verificação.`,
  region_changed: (n) => `O conteúdo de ${n} regiões foi alterado nesta verificação.`,
  script_file_added: (n) => `${n} arquivos foram adicionados ao projeto Apps Script.`,
  script_file_removed: (n) => `${n} arquivos foram removidos do projeto Apps Script.`,
  script_file_changed: (n) => `${n} arquivos do projeto Apps Script foram alterados.`,
  script_code_changed: (n) => `${n} trechos do projeto Apps Script mudaram.`,
};

export function diagnosticText(code: string): string {
  if (code.startsWith("rate_limited|")) {
    return `${diagnosticText("rate_limited")} Detalhes: ${code.slice("rate_limited|".length, 512)}`;
  }
  const guidance = errorGuidance(code);
  if (guidance)
    return `${guidance.label}. Impacto: ${guidance.impact} Solução: ${guidance.solution}`;
  const countMatch = /^([a-z0-9_]+):count=(\d+)$/.exec(code);
  if (countMatch) {
    const [, base, rawCount] = countMatch;
    const count = Number(rawCount);
    if (count > 1 && pluralDiagnoses[base]) return pluralDiagnoses[base](count);
    if (diagnoses[base]) return diagnoses[base];
  }
  return (
    diagnoses[code] ??
    (code.includes(" ")
      ? code
      : "Uma alteração ou restrição foi identificada. Consulte os detalhes da regra e da verificação.")
  );
}

export function activityTone(event: Activity): "failure" | "resolved" | "neutral" {
  if (event.tone === "failure" || event.tone === "resolved" || event.tone === "neutral") {
    return event.tone;
  }
  if (event.kind === "recovered" || event.kind === "analysis_concluded") {
    return "resolved";
  }
  if (
    event.kind === "violation" ||
    event.kind === "inconclusive" ||
    event.kind === "analysis_inconclusive" ||
    event.kind === "failed" ||
    event.severity === "critical" ||
    event.severity === "warning"
  ) {
    return "failure";
  }
  return "neutral";
}

export function eventKindText(kind: string) {
  return eventKinds[kind] ?? "Alteração registrada";
}

export function connectivityText(connectivity: string) {
  return connectivityNames[connectivity] ?? "Estado de conexão não confirmado";
}

export function accessErrorText(code: string | null | undefined) {
  if (!code) {
    return "Não foi possível confirmar o acesso na última tentativa. Verifique o compartilhamento do arquivo e tente atualizar os dados.";
  }
  return managementError({ code }, "acessar a planilha");
}

const severities: Record<string, string> = {
  critical: "Crítica",
  warning: "Aviso",
  info: "Informação",
};

const issueStatuses: Record<string, string> = {
  active: "Ativo",
  resolved: "Resolvido",
};

export function EventList({
  events,
  onOpen,
}: {
  events: Activity[];
  onOpen: (event: Activity) => void;
}) {
  return (
    <div className="op-event-list">
      {events.map((event) => {
        const tone = activityTone(event);
        const text = event.description || diagnosticText(event.diagnosis);
        const timestamp = event.formatted_detected_at || dateLabel(event.detected_at);
        const statusText = event.issue_status ? issueStatuses[event.issue_status] : undefined;
        return (
          <button
            type="button"
            className="op-event"
            key={event.event_id}
            onClick={() => onOpen(event)}
          >
            <span className="op-event-indicator" aria-hidden="true">
              <span className={`op-event-dot op-tone-${tone}`} />
            </span>
            <span className="op-event-body">
              <strong>{event.spreadsheet_name}</strong>
              <span>{text}</span>
              <small>
                {event.sheet_title ?? "Arquivo"} · {categories[event.category] ?? event.category} ·{" "}
                {severities[event.severity] ?? event.severity}
                {statusText ? ` · ${statusText}` : ""}
                {event.removed ? " · Cadastro removido" : ""}
              </small>
            </span>
            <time dateTime={event.detected_at}>{timestamp}</time>
          </button>
        );
      })}
    </div>
  );
}

export function EventDiagnostic({
  event,
  onClose,
}: {
  event: Activity | null;
  onClose: () => void;
}) {
  const uncertain =
    event?.kind.includes("inconclusive") ||
    event?.status === "indeterminate" ||
    event?.category === "operational";
  const description = event ? event.description || diagnosticText(event.diagnosis) : "";
  const timestamp = event ? event.formatted_detected_at || dateLabel(event.detected_at) : "";
  return (
    <Dialog
      open={event !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent className="op-diagnostic-dialog">
        <DialogHeader>
          <DialogTitle>Diagnóstico da alteração</DialogTitle>
          <DialogDescription>
            {event?.spreadsheet_name} · {timestamp}
          </DialogDescription>
        </DialogHeader>
        {event && (
          <div className="op-diagnostic">
            <div className="op-notice">
              <strong>{uncertain ? "Não foi possível determinar" : description}</strong>
              {uncertain && <p>{description}</p>}
            </div>
            <dl>
              <div>
                <dt>Local</dt>
                <dd>
                  {event.sheet_title ?? "Arquivo"}
                  {event.location ? ` · ${event.location}` : ""}
                </dd>
              </div>
              <div>
                <dt>Categoria e tipo</dt>
                <dd>
                  {categories[event.category] ?? "Registro"} · {eventKindText(event.kind)}
                </dd>
              </div>
              <div>
                <dt>Severidade</dt>
                <dd>{severities[event.severity] ?? event.severity}</dd>
              </div>
              <div>
                <dt>Estado resultante</dt>
                <dd>{statusNames[event.status as keyof typeof statusNames] ?? event.status}</dd>
              </div>
              <div>
                <dt>Regra afetada</dt>
                <dd>{event.rule_id ?? "Alteração observada sem regra específica"}</dd>
              </div>
              <div>
                <dt>Estrutura anterior / esperada</dt>
                <dd>{event.before ?? "Não disponível neste evento"}</dd>
              </div>
              <div>
                <dt>Estrutura atual / encontrada</dt>
                <dd>{event.after ?? "Não disponível neste evento"}</dd>
              </div>
              <div>
                <dt>Impacto e dependências</dt>
                <dd>
                  {uncertain
                    ? "A cobertura disponível não permite confirmar o impacto. Consulte o relatório e as dependências não verificáveis."
                    : event.severity === "critical"
                      ? "A violação confirmada pode comprometer regras críticas. Consulte o relatório da verificação para identificar dependências afetadas."
                      : "Consulte as regras e a cobertura da verificação. Uma mudança de conteúdo pode ser permitida sem comprometer a integridade."}
                </dd>
              </div>
              <div>
                <dt>Ação sugerida</dt>
                <dd>
                  {uncertain
                    ? "Revise o acesso, a cobertura das regiões e as dependências não verificáveis. Execute uma nova análise após resolver a causa."
                    : "Confira a estrutura e a regra indicada. Se a alteração for legítima, revise as regras e aprove uma nova referência explicitamente."}{" "}
                  Nenhuma correção é executada automaticamente.
                </dd>
              </div>
              <div>
                <dt>Identificador da verificação</dt>
                <dd>
                  <code>{event.check_id}</code>
                </dd>
              </div>
            </dl>
            <p>Responsável não identificado</p>
            <AuditDetails event={event} />
            <p className="op-hint">
              A detecção registra quando o aplicativo observou a diferença. A autoria da alteração
              não é determinada.
            </p>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
