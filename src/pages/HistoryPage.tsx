import { Search } from "lucide-react";
import { Card, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { SearchField, ActivityRow } from "@/components/foundation";
import { categoryNames } from "@/lib/presentation";
import type { FoundationDashboard, DemoActivity } from "@/types/foundation";
export function HistoryPage({
  data,
  setDetail,
  query,
  setQuery,
  filter,
  setFilter,
  visibleEvents,
}: {
  data: FoundationDashboard;
  setDetail: (event: DemoActivity) => void;
  query: string;
  setQuery: (value: string) => void;
  filter: string;
  setFilter: (value: string) => void;
  visibleEvents: DemoActivity[];
}) {
  return (
    <Card>
      <CardHeader className="panel-heading">
        <div>
          <CardTitle>Registro de eventos</CardTitle>
          <CardDescription>
            Exemplos sintéticos; nenhuma alteração real foi detectada.
          </CardDescription>
        </div>
        <Badge variant="demo">HISTÓRICO DEMO</Badge>
      </CardHeader>
      <div className="table-toolbar">
        <SearchField
          label="Buscar eventos"
          value={query}
          onChange={setQuery}
          placeholder="Buscar arquivo ou diagnóstico…"
        />
        <select
          aria-label="Filtrar categoria"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="select-input"
        >
          <option value="all">Todas as categorias</option>
          {Object.entries(categoryNames).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </div>
      {visibleEvents.map((event) => (
        <ActivityRow key={event.id} event={event} expanded onClick={() => setDetail(event)} />
      ))}
      {!visibleEvents.length && (
        <div className="empty-state">
          <Search size={26} />
          <h2>Nenhum evento nesta seleção</h2>
          <p>Experimente outro termo ou categoria.</p>
        </div>
      )}
      <div className="panel-footer">
        <span>{visibleEvents.length} eventos demonstrativos</span>
        <span>Retenção planejada: {data.settings.history_days} dias</span>
      </div>
    </Card>
  );
}
