import { Activity, Pause, Play, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { diagnosticText, formatLocalDateTime } from "./OperationalShared";
import type { MonitoringStatus, NotificationRecord } from "@/types/monitoring";
import type { Page, RegisteredSpreadsheet } from "@/types/management";

export function operationLabel(code: string | null) {
  if (!code) return "Disponível";
  const labels: Record<string, string> = {
    network: "Sem conexão",
    timeout: "Tempo de consulta esgotado",
    rate_limited: "Aguardando cota da API",
    forbidden: "Permissão indisponível",
    not_found: "Arquivo indisponível",
    reconnect_required: "Reautorize a conta",
    insufficient_scope: "Permissão insuficiente",
    server: "API temporariamente indisponível",
  };
  return labels[code] ?? "Consulta temporariamente indisponível";
}
export function monitoringDate(value: number | null) {
  if (value === null) return "Ainda não verificado";
  return formatLocalDateTime(value) || "Data indisponível";
}
export function MonitoringPanel({
  status,
  rows,
  disabled,
  onToggle,
  onCheck,
}: {
  status: MonitoringStatus | null;
  rows: RegisteredSpreadsheet[];
  disabled: boolean;
  onToggle: (enabled: boolean) => void;
  onCheck: () => void;
}) {
  const eligible = new Set(status?.schedules.map((item) => item.spreadsheet_id));
  const schedules =
    status?.schedules.filter((item) =>
      rows.some((row) => row.spreadsheet_id === item.spreadsheet_id && !row.paused),
    ) ?? [];
  const due = schedules.length ? Math.min(...schedules.map((item) => item.next_run_ms)) : null;
  const running = schedules.filter((item) => item.lease_owner !== null).length;
  const failed = schedules.filter((item) => item.operational_code !== null).length;
  return (
    <Card className="op-monitoring">
      <CardHeader>
        <CardTitle>
          <Activity size={18} /> Monitoramento contínuo
        </CardTitle>
        <CardDescription>
          {status?.control.enabled
            ? "Ativo no aplicativo, inclusive com a janela oculta."
            : "Pausado. Inicie quando as proteções estiverem prontas."}
        </CardDescription>
      </CardHeader>
      <CardContent className="op-stack">
        <div className="op-actions">
          <Button disabled={disabled || !status} onClick={() => onToggle(!status?.control.enabled)}>
            {status?.control.enabled ? <Pause /> : <Play />}
            {status?.control.enabled ? "Pausar monitoramento" : "Iniciar monitoramento"}
          </Button>
          <Button
            variant="outline"
            disabled={disabled || !status?.eligible_count}
            onClick={onCheck}
          >
            <RefreshCw /> Verificar elegíveis agora
          </Button>
        </div>
        <p className="op-hint" role="status">
          {status?.eligible_count ?? 0} planilhas prontas · {running} em verificação · intervalo de{" "}
          {status?.interval_seconds ?? 60}s{failed > 0 && ` · ${failed} consultas indeterminadas`}
        </p>
        {status?.service.warning && (
          <p className="op-hint" role="status">
            {status.service.warning === "interval_capacity_exceeded"
              ? "A duração das consultas excede o intervalo configurado. Aumente o intervalo ou reduza o escopo; verificações não são sobrepostas."
              : status.service.warning === "insufficient_scope"
                ? "Alguns projetos precisam de nova permissão para serem consultados."
                : status.service.warning === "authentication_required"
                  ? "Conecte ou reautorize a conta para retomar consultas."
                  : "O serviço encontrou uma indisponibilidade local e tentará se recuperar."}
          </p>
        )}
        {status?.control.enabled && due !== null && (
          <p className="op-hint">Próxima verificação: {monitoringDate(due)}</p>
        )}
        {status && status.eligible_count === 0 && (
          <p className="op-hint">
            Selecione abas, salve regras e aprove uma referência para habilitar verificações.
            Nenhuma referência é aprovada automaticamente.
          </p>
        )}
        {schedules.length > 0 && (
          <div className="op-table-wrap">
            <table className="op-table">
              <caption className="sr-only">Execução das verificações</caption>
              <thead>
                <tr>
                  <th>Planilha</th>
                  <th>Operação</th>
                  <th>Última conclusão</th>
                  <th>Próxima consulta</th>
                </tr>
              </thead>
              <tbody>
                {rows
                  .filter((row) => eligible.has(row.spreadsheet_id))
                  .map((row) => {
                    const item = schedules.find(
                      (schedule) => schedule.spreadsheet_id === row.spreadsheet_id,
                    );
                    if (!item) return null;
                    return (
                      <tr key={row.spreadsheet_id}>
                        <td>{row.name}</td>
                        <td>
                          {item.lease_owner
                            ? "Verificando"
                            : item.operational_code
                              ? `Indeterminado · ${operationLabel(item.operational_code)}`
                              : row.paused
                                ? "Pausado"
                                : "Aguardando"}
                        </td>
                        <td>{monitoringDate(item.last_completed_ms)}</td>
                        <td>
                          {status?.control.enabled ? monitoringDate(item.next_run_ms) : "Pausado"}
                        </td>
                      </tr>
                    );
                  })}
              </tbody>
            </table>
          </div>
        )}
        <p className="op-hint">
          Falhas de conexão não comprovam perda de integridade. Após uma interrupção, apenas o
          estado efetivamente observado pode ser comparado.
        </p>
      </CardContent>
    </Card>
  );
}

const notificationStates: Record<string, string> = {
  pending: "Aguardando tentativa",
  claimed: "Tentativa em andamento",
  dispatch_submitted: "Submissão nativa registrada",
  submitted: "Submissão nativa registrada",
  retryable: "Nova tentativa programada",
  retry: "Nova tentativa programada",
  dispatching: "Tentativa em andamento",
  permission_blocked: "Permissão bloqueada",
  unsupported: "Indisponível nesta plataforma",
  suppressed_by_preference: "Suprimida pela preferência",
  expired: "Prazo expirado",
};
export function NotificationHistory({
  page,
  disabled,
  onPage,
  onOpen,
}: {
  page: Page<NotificationRecord> | null;
  disabled: boolean;
  onPage: (offset: number) => void;
  onOpen: (record: NotificationRecord) => void;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Notificações</CardTitle>
        <CardDescription>
          Cada alteração detectada tem um registro próprio. Submissão nativa não confirma exibição
          nem leitura no Windows.
        </CardDescription>
      </CardHeader>
      <CardContent className="op-stack">
        {!page?.items.length ? (
          <p className="op-hint">Nenhuma notificação registrada nesta conta.</p>
        ) : (
          <ul className="op-notification-list">
            {page.items.map((item) => (
              <li key={item.id}>
                <div>
                  <strong>
                    {item.spreadsheet_name}
                    {item.sheet_title ? ` · ${item.sheet_title}` : ""}
                  </strong>
                  <p>{diagnosticText(item.diagnosis)}</p>
                  <small>
                    {item.event_type_label ? `${item.event_type_label} · ` : ""}
                    {item.formatted_detected_at || formatLocalDateTime(item.detected_at)} ·{" "}
                    {notificationStates[item.state] ?? "Estado de envio indisponível"} ·{" "}
                    {item.attempt_count} tentativas
                  </small>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={disabled}
                  onClick={() => onOpen(item)}
                >
                  Abrir evento
                </Button>
              </li>
            ))}
          </ul>
        )}
        {page && page.total > page.limit && (
          <div className="op-actions">
            <Button
              variant="outline"
              disabled={disabled || page.offset === 0}
              onClick={() => onPage(Math.max(0, page.offset - page.limit))}
            >
              Anteriores
            </Button>
            <span>
              {page.offset + 1}–{Math.min(page.offset + page.limit, page.total)} de {page.total}
            </span>
            <Button
              variant="outline"
              disabled={disabled || page.offset + page.limit >= page.total}
              onClick={() => onPage(page.offset + page.limit)}
            >
              Próximas
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
