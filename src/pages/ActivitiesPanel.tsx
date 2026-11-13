import { useState } from "react";
import { Download, Search } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { EventList } from "./OperationalShared";
import type {
  Activity,
  EventFilter,
  ExportFormat,
  Page,
  RegisteredSpreadsheet,
} from "@/types/management";

function dateValue(value: string | null | undefined) {
  if (!value) return "";
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return "";
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function boundary(value: string, end: boolean) {
  return value ? new Date(`${value}T${end ? "23:59:59.999" : "00:00:00.000"}`).toISOString() : null;
}
export function ActivitiesPanel({
  filter,
  onFilter,
  page,
  rows,
  disabled,
  exportFeedback,
  onSearch,
  onExport,
  onPage,
  onOpen,
}: {
  filter: EventFilter;
  onFilter: (filter: EventFilter) => void;
  page: Page<Activity> | null;
  rows: RegisteredSpreadsheet[];
  disabled: boolean;
  exportFeedback?: { empty: boolean; message: string } | null;
  onSearch: () => void;
  onExport?: (format: ExportFormat) => void;
  onPage: (offset: number) => void;
  onOpen: (event: Activity) => void;
}) {
  const [exportFormat, setExportFormat] = useState<ExportFormat>("csv");
  const tabs = rows.find((row) => row.spreadsheet_id === filter.spreadsheet_id)?.tabs ?? [];
  const statusApplicable = filter.event_type !== "modification";
  function change(patch: Partial<EventFilter>) {
    onFilter({ ...filter, ...patch, offset: 0 });
  }
  return (
    <Card>
      <CardHeader>
        <div className="op-section-heading">
          <div>
            <CardTitle>Linha do tempo</CardTitle>
            <CardDescription>
              Atualizações vinculadas às regras de proteção, persistidas e paginadas no banco local.
              A data indica a detecção; a autoria não é inferida. Falhas de consulta ficam em
              Monitoramento.
            </CardDescription>
          </div>
          {onExport && (
            <div className="op-export-bar" role="group" aria-label="Exportação de logs">
              <select
                aria-label="Formato de exportação"
                value={exportFormat}
                disabled={disabled}
                onChange={(e) => setExportFormat(e.target.value as ExportFormat)}
              >
                <option value="csv">CSV (UTF-8 / Excel)</option>
                <option value="json">JSON estruturado</option>
                <option value="xlsx">XLSX (Excel)</option>
              </select>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={disabled}
                onClick={() => onExport(exportFormat)}
              >
                <Download size={15} />
                Exportar logs
              </Button>
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {exportFeedback && (
          <p
            className={`feedback ${exportFeedback.empty ? "" : "success"} op-export-feedback`}
            role="status"
          >
            {exportFeedback.message}
          </p>
        )}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSearch();
          }}
          className="op-activity-filters"
        >
          <label className="op-field">
            Pesquisar no histórico
            <Input
              value={filter.query ?? ""}
              maxLength={200}
              placeholder="Planilha, diagnóstico ou regra"
              onChange={(e) => change({ query: e.target.value })}
            />
          </label>
          <label className="op-field">
            Planilha
            <select
              value={filter.spreadsheet_id ?? ""}
              onChange={(e) => change({ spreadsheet_id: e.target.value || null, sheet_id: null })}
            >
              <option value="">Todas, incluindo histórico preservado</option>
              {rows.map((row) => (
                <option key={row.spreadsheet_id} value={row.spreadsheet_id}>
                  {row.name}
                </option>
              ))}
            </select>
          </label>
          <label className="op-field">
            Aba
            <select
              value={filter.sheet_id ?? ""}
              disabled={!filter.spreadsheet_id}
              onChange={(e) =>
                change({ sheet_id: e.target.value === "" ? null : Number(e.target.value) })
              }
            >
              <option value="">Todas as abas</option>
              {tabs.map((tab) => (
                <option key={tab.sheet_id} value={tab.sheet_id}>
                  {tab.title}
                </option>
              ))}
            </select>
          </label>
          <label className="op-field">
            Data inicial
            <Input
              type="date"
              value={dateValue(filter.from)}
              onChange={(e) => change({ from: boundary(e.target.value, false) })}
            />
          </label>
          <label className="op-field">
            Data final
            <Input
              type="date"
              value={dateValue(filter.to)}
              onChange={(e) => change({ to: boundary(e.target.value, true) })}
            />
          </label>
          <label className="op-field">
            Tipo
            <select
              value={filter.event_type ?? ""}
              onChange={(e) => {
                const nextType = (e.target.value || null) as EventFilter["event_type"];
                change({
                  event_type: nextType,
                  issue_status: nextType === "modification" ? null : filter.issue_status,
                });
              }}
            >
              <option value="">Todos os eventos</option>
              <option value="failure">Falha</option>
              <option value="modification">Modificação</option>
              <option value="resolution">Resolução</option>
            </select>
          </label>
          <label className="op-field">
            Status
            <select
              value={statusApplicable ? (filter.issue_status ?? "") : ""}
              disabled={!statusApplicable}
              onChange={(e) =>
                change({
                  issue_status: (e.target.value || null) as EventFilter["issue_status"],
                })
              }
            >
              <option value="">{statusApplicable ? "Todos" : "Não aplicável"}</option>
              <option value="active">Ativo</option>
              <option value="resolved">Resolvido</option>
            </select>
          </label>
          <label className="op-field">
            Severidade
            <select
              value={filter.severity ?? ""}
              onChange={(e) => change({ severity: e.target.value || null })}
            >
              <option value="">Todas</option>
              <option value="critical">Crítica</option>
              <option value="warning">Aviso</option>
              <option value="info">Informação</option>
            </select>
          </label>
          <label className="op-field">
            Estado resultante
            <select
              value={filter.status ?? ""}
              onChange={(e) => change({ status: e.target.value || null })}
            >
              <option value="">Todos</option>
              <option value="online">Online</option>
              <option value="offline">Offline</option>
              <option value="indeterminate">Indeterminado</option>
            </select>
          </label>
          <label className="op-field">
            Categoria
            <select
              value={filter.category ?? ""}
              onChange={(e) => change({ category: e.target.value || null })}
            >
              <option value="">Todas</option>
              <option value="integrity">Integridade e regras</option>
            </select>
          </label>
          <label className="op-field">
            Tipo de alteração
            <select
              value={filter.kind ?? ""}
              onChange={(e) => change({ kind: e.target.value || null })}
            >
              <option value="">Todos</option>
              <option value="violation">Violação estrutural</option>
              <option value="analysis_inconclusive">Análise inconclusiva</option>
              <option value="recovered">Recuperação</option>
              <option value="analysis_concluded">Análise concluída</option>
            </select>
          </label>
          <div className="op-activity-actions">
            <label className="op-check">
              <input
                type="checkbox"
                checked={filter.script_only ?? false}
                onChange={(e) => change({ script_only: e.target.checked })}
              />
              Somente Apps Script
            </label>
            <Button type="submit" disabled={disabled}>
              <Search />
              Aplicar filtros
            </Button>
          </div>
        </form>
        {page ? (
          <>
            {page.items.length > 0 ? (
              <EventList events={page.items} onOpen={onOpen} />
            ) : (
              <p className="op-empty">
                Nenhum evento corresponde aos filtros. Uma verificação sem diferenças não cria
                alterações artificiais.
              </p>
            )}
            <div className="op-pagination">
              <span>
                {page.total} evento(s) ·{" "}
                {page.items.length ? `${page.offset + 1}–${page.offset + page.items.length}` : "0"}
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={disabled || page.offset === 0}
                onClick={() => onPage(Math.max(0, page.offset - page.limit))}
              >
                Atividades anteriores
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={disabled || page.offset + page.limit >= page.total}
                onClick={() => onPage(page.offset + page.limit)}
              >
                Próximas atividades
              </Button>
            </div>
          </>
        ) : (
          <p className="op-empty">Aplique os filtros para consultar as atividades.</p>
        )}
        <p className="op-hint">
          Somente violações, inconclusões e recuperações de regras aparecem aqui. Edições livres de
          conteúdo não são atualizações de proteção. Consulte Monitoramento para falhas
          operacionais.
        </p>
      </CardContent>
    </Card>
  );
}
