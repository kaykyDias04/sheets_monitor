import { Search } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { EventList } from "./OperationalShared";
import type { Activity, EventFilter, Page, RegisteredSpreadsheet } from "@/types/management";

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
  onSearch,
  onPage,
  onOpen,
}: {
  filter: EventFilter;
  onFilter: (filter: EventFilter) => void;
  page: Page<Activity> | null;
  rows: RegisteredSpreadsheet[];
  disabled: boolean;
  onSearch: () => void;
  onPage: (offset: number) => void;
  onOpen: (event: Activity) => void;
}) {
  const tabs = rows.find((row) => row.spreadsheet_id === filter.spreadsheet_id)?.tabs ?? [];
  function change(patch: Partial<EventFilter>) {
    onFilter({ ...filter, ...patch, offset: 0 });
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle>Linha do tempo</CardTitle>
        <CardDescription>
          Eventos persistidos e paginados no banco local. A data indica a detecção; a autoria não é
          inferida.
        </CardDescription>
      </CardHeader>
      <CardContent>
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
              <option value="content">Conteúdo</option>
              <option value="structure">Estrutura</option>
              <option value="script">Apps Script</option>
              <option value="operational">Operacional</option>
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
              <option value="value_changed">Valor alterado</option>
              <option value="record_added">Registro adicionado</option>
              <option value="record_removed">Registro removido</option>
              <option value="record_order_changed">Ordem de registros</option>
              <option value="header_changed">Cabeçalho alterado</option>
              <option value="script_code_changed">Código de script alterado</option>
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
          Mudanças de conteúdo podem ser permitidas e continuar Online. Violações de regras críticas
          podem resultar em Offline. Consulte o diagnóstico e o estado da verificação.
        </p>
      </CardContent>
    </Card>
  );
}
