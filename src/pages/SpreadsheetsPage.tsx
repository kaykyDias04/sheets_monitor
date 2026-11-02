import { Database, LockKeyhole } from "lucide-react";
import { Card, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Status, SearchField, SheetsTable } from "@/components/foundation";
import { stateLabels, stateHelp } from "@/lib/presentation";
import { desktop } from "@/services/foundation";
import type { FoundationDashboard, DemoSpreadsheet, IntegrityStatus } from "@/types/foundation";
export function SpreadsheetsPage({
  data,
  setBaseline,
  query,
  setQuery,
  filter,
  setFilter,
  visibleSheets,
}: {
  data: FoundationDashboard;
  setBaseline: (sheet: DemoSpreadsheet) => void;
  query: string;
  setQuery: (value: string) => void;
  filter: string;
  setFilter: (value: string) => void;
  visibleSheets: DemoSpreadsheet[];
}) {
  return (
    <>
      <Card className="sheet-management">
        <CardHeader className="panel-heading">
          <div>
            <CardTitle>
              Planilhas do workspace{" "}
              <Badge variant="secondary">{data.spreadsheets.length} / 30</Badge>
            </CardTitle>
            <CardDescription>
              Fontes sintéticas para validar a interface e a persistência.
            </CardDescription>
          </div>
          <Button variant="outline" size="sm" disabled>
            <LockKeyhole />
            Cadastro Google · próxima etapa
          </Button>
        </CardHeader>
        <div className="table-toolbar">
          <SearchField
            label="Buscar planilhas"
            value={query}
            onChange={setQuery}
            placeholder="Buscar por nome ou ID fictício…"
          />
          <select
            aria-label="Filtrar estado"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="select-input"
          >
            <option value="all">Todos os estados</option>
            {Object.entries(stateLabels).map(([state, label]) => (
              <option key={state} value={state}>
                {label}
              </option>
            ))}
          </select>
        </div>
        <SheetsTable sheets={visibleSheets} onPreview={setBaseline} />
        <div className="panel-footer">
          <span>
            <Database size={12} />
            {desktop ? "Fixtures persistidas no SQLite" : "Fixtures em memória nesta sessão"}
          </span>
          <span>{visibleSheets.length} fontes demonstrativas</span>
        </div>
      </Card>
      <section className="state-guide" aria-label="Significado dos estados">
        {Object.entries(stateLabels).map(([state, label]) => (
          <div key={state}>
            <Status status={state as IntegrityStatus} />
            <p>{stateHelp[state as IntegrityStatus].replace(" no cenário demonstrativo.", ".")}</p>
            <span className="sr-only">{label}</span>
          </div>
        ))}
      </section>
    </>
  );
}
