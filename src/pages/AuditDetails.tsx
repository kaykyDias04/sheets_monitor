import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { desktop } from "@/services/platform";
import { getIntegrationState } from "@/services/google";
import { managementError } from "@/services/management";
import type { Activity } from "@/types/management";
import type { EngineEnvelope } from "@/types/integrity";
import { formatLocalDateTime } from "@/lib/syncPresentation";
interface AuditReport {
  responsible: string;
  note: string;
  records: { actor: string | null; occurred_at: string; action: string }[];
  truncated: boolean;
}
export function AuditDetails({ event }: { event: Activity }) {
  const [report, setReport] = useState<AuditReport | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [request, setRequest] = useState<string | null>(null);
  const live = useRef(true);
  const requestId = useRef(0);
  useEffect(() => {
    live.current = true;
    return () => {
      live.current = false;
    };
  }, []);
  async function load() {
    const sequence = ++requestId.current;
    setRequest(event.event_id);
    setReport(null);
    setError("");
    setBusy(true);
    try {
      const before = await getIntegrationState();
      const result = await invoke<EngineEnvelope<AuditReport>>("get_event_audit", {
        eventId: event.event_id,
      });
      const after = await getIntegrationState();
      if (
        !live.current ||
        sequence !== requestId.current ||
        result.generation !== after.generation ||
        before.generation !== after.generation ||
        result.account_subject !== after.auth.account?.subject
      )
        return;
      setReport(result.data);
    } catch (e) {
      if (live.current && sequence === requestId.current)
        setError(
          managementError(
            e,
            "consultar auditoria Workspace; a identidade permanece não identificada",
          ),
        );
    } finally {
      if (live.current && sequence === requestId.current) setBusy(false);
    }
  }
  const active = request === event.event_id;
  return (
    <section aria-label="Auditoria Google Workspace">
      <Button
        variant="outline"
        size="sm"
        disabled={!desktop || (active && (busy || report !== null))}
        onClick={() => void load()}
      >
        {active && busy ? "Consultando auditoria…" : "Consultar auditoria Workspace"}
      </Button>
      {active && error && <p role="status">{error}</p>}
      {active && report && (
        <>
          <p>{report.note}</p>
          <ul>
            {report.records.map((record, index) => (
              <li key={index}>
                {formatLocalDateTime(record.occurred_at)} ·{" "}
                {record.actor ?? "Identidade omitida pelo Google"} · {record.action}
              </li>
            ))}
          </ul>
          {report.records.length === 0 && (
            <p>
              Nenhuma edição disponível nessa janela. Ausência de registro não comprova ausência de
              alteração.
            </p>
          )}
          {report.truncated && <p>Consulta parcial: há mais registros no Workspace.</p>}
        </>
      )}
    </section>
  );
}
