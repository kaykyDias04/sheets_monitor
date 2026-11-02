import {
  ArrowRight,
  ArrowUpRight,
  BookOpen,
  ChevronRight,
  CircleAlert,
  FileCode2,
  FileSpreadsheet,
  FlaskConical,
  Search,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { stateHelp, stateLabels, formatDate, categoryNames } from "@/lib/presentation";
import type { DemoSpreadsheet, DemoActivity, IntegrityStatus } from "@/types/foundation";
export function Status({ status }: { status: IntegrityStatus }) {
  return (
    <Badge variant="outline" className={cn("status-pill", status)} title={stateHelp[status]}>
      <span
        className={cn(
          "size-1.5 rounded-full bg-current",
          status === "checking" && "animate-pulse motion-reduce:animate-none",
        )}
      />
      {stateLabels[status]}
    </Badge>
  );
}

export function SearchField({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <div className="search-field">
      <Search size={16} />
      <Input
        aria-label={label}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
      />
    </div>
  );
}
export function SheetsPanel({
  sheets,
  onPreview,
  dashboard,
  onAll,
}: {
  sheets: DemoSpreadsheet[];
  onPreview: (sheet: DemoSpreadsheet) => void;
  dashboard?: boolean;
  onAll?: () => void;
}) {
  return (
    <Card>
      <CardHeader className="panel-heading">
        <div>
          <CardTitle>
            Integridade das planilhas <Badge variant="secondary">{sheets.length} / 30</Badge>
          </CardTitle>
          <CardDescription>Uma prévia de cada estado previsto na especificação.</CardDescription>
        </div>
        {dashboard && (
          <button className="text-action" onClick={onAll}>
            Ver planilhas
            <ArrowRight size={14} />
          </button>
        )}
      </CardHeader>
      <SheetsTable sheets={sheets} onPreview={onPreview} />
      <div className="panel-footer">
        <span>
          <FlaskConical size={12} />
          Estados simulados · nenhuma coleta real
        </span>
        <span>Capacidade planejada: 30 arquivos</span>
      </div>
    </Card>
  );
}
export function SheetsTable({
  sheets,
  onPreview,
}: {
  sheets: DemoSpreadsheet[];
  onPreview: (sheet: DemoSpreadsheet) => void;
}) {
  return (
    <>
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th>PLANILHA</th>
              <th>INTEGRIDADE</th>
              <th>ÚLTIMA AVALIAÇÃO DEMO</th>
              <th>ABAS</th>
              <th>
                <span className="sr-only">Ações</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {sheets.map((sheet) => (
              <tr key={sheet.id}>
                <td>
                  <div className="sheet-name">
                    <span className="sheet-symbol">
                      <FileSpreadsheet size={20} />
                    </span>
                    <div>
                      <strong>{sheet.name}</strong>
                      <small>{sheet.url}</small>
                    </div>
                  </div>
                </td>
                <td>
                  <Status status={sheet.status} />
                </td>
                <td className="date-cell">{formatDate(sheet.last_checked_at)}</td>
                <td className="sheet-tabs" title={sheet.selected_sheets.join(", ")}>
                  {sheet.selected_sheets.length
                    ? `${sheet.selected_sheets.length} selecionada(s)`
                    : "Arquivo inteiro"}
                </td>
                <td>
                  <Button
                    variant="ghost"
                    size="sm"
                    aria-label={`Ver exemplo de baseline de ${sheet.name}`}
                    onClick={() => onPreview(sheet)}
                  >
                    <BookOpen />
                    <span className="preview-action-text">Ver exemplo</span>
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {!sheets.length && (
        <div className="empty-state">
          <FileSpreadsheet />
          <h2>Nenhuma planilha nesta seleção</h2>
          <p>Os exemplos continuam disponíveis ao limpar o filtro.</p>
        </div>
      )}
    </>
  );
}
export function ActivityRow({
  event,
  onClick,
  expanded = false,
}: {
  event: DemoActivity;
  onClick: () => void;
  expanded?: boolean;
}) {
  return (
    <button className={cn("activity-row", expanded && "expanded")} onClick={onClick}>
      <span className={cn("activity-icon", event.category)}>
        {event.category === "operational" ? (
          <CircleAlert size={17} />
        ) : event.category === "script" ? (
          <FileCode2 size={17} />
        ) : (
          <ArrowUpRight size={17} />
        )}
      </span>
      <div>
        <strong>{event.spreadsheet_name}</strong>
        <p>
          {categoryNames[event.category] ?? event.category}
          <span>·</span>Evento demonstrativo
        </p>
        {expanded && <p className="event-diagnosis">{event.diagnosis}</p>}
      </div>
      <time>{formatDate(event.occurred_at)}</time>
      <ChevronRight size={16} />
    </button>
  );
}
