import {
  Check,
  CheckCircle2,
  CircleAlert,
  RefreshCw,
  ShieldCheck,
  WifiOff,
  ArrowRight,
} from "lucide-react";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { SheetsPanel, ActivityRow } from "@/components/foundation";
import { stateLabels } from "@/lib/presentation";
import { cn } from "@/lib/utils";
import type { FoundationDashboard, DemoSpreadsheet, DemoActivity, Page } from "@/types/foundation";
export function DashboardPage({
  data,
  setBaseline,
  setDetail,
  navigate,
}: {
  data: FoundationDashboard;
  setBaseline: (sheet: DemoSpreadsheet) => void;
  setDetail: (event: DemoActivity) => void;
  navigate: (page: Page) => void;
}) {
  return (
    <>
      <section className="stats-grid" aria-label="Resumo de integridade demonstrativa">
        {(
          [
            {
              status: "online",
              icon: CheckCircle2,
              description: "Estruturalmente íntegras",
            },
            {
              status: "offline",
              icon: WifiOff,
              description: "Falha crítica confirmada",
            },
            {
              status: "checking",
              icon: RefreshCw,
              description: "Avaliação em andamento",
            },
            {
              status: "indeterminate",
              icon: CircleAlert,
              description: "Integridade não confirmada",
            },
          ] as const
        ).map((item) => (
          <Card key={item.status} className="stat-card">
            <div className="stat-label">
              <span>{stateLabels[item.status]}</span>
              <span className={cn("stat-icon", item.status)}>
                <item.icon size={19} />
              </span>
            </div>
            <strong>
              {data.spreadsheets
                .filter((s) => s.status === item.status)
                .length.toString()
                .padStart(2, "0")}
            </strong>
            <p>{item.description}</p>
          </Card>
        ))}
      </section>
      <SheetsPanel
        sheets={data.spreadsheets}
        onPreview={setBaseline}
        dashboard
        onAll={() => navigate("sheets")}
      />
      <div className="lower-grid">
        <Card className="activity-card">
          <CardHeader className="panel-heading">
            <CardTitle>Atividade demonstrativa</CardTitle>
            <button className="text-action" onClick={() => navigate("history")}>
              Ver histórico
              <ArrowRight size={14} />
            </button>
          </CardHeader>
          {data.activities.slice(0, 3).map((event) => (
            <ActivityRow key={event.id} event={event} onClick={() => setDetail(event)} />
          ))}
        </Card>
        <div className="integrity-note">
          <span className="note-icon">
            <ShieldCheck size={26} />
          </span>
          <h2>Mudança e integridade têm papéis diferentes</h2>
          <p>
            Editar um valor comum pode gerar uma notificação e manter a planilha Online. Offline
            representa uma falha crítica confirmada.
          </p>
          <span className="note-footer">
            <Check size={14} />
            REGRA ILUSTRADA COM FIXTURES
          </span>
        </div>
      </div>
    </>
  );
}
