import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MonitoringPanel, NotificationHistory } from "./MonitoringPanel";
import type { MonitoringStatus, NotificationRecord } from "@/types/monitoring";
afterEach(cleanup);

const status: MonitoringStatus = {
  control: { enabled: false, generation: 1 },
  schedules: [],
  eligible_count: 0,
  interval_seconds: 60,
  service: { running: true, active: 0, queued: 0, warning: null },
};
describe("continuous monitoring controls", () => {
  it("requires ready protection before checks and an explicit start action", async () => {
    const toggle = vi.fn();
    render(
      <MonitoringPanel
        status={status}
        rows={[]}
        disabled={false}
        onToggle={toggle}
        onCheck={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "Verificar elegíveis agora" })).toBeDisabled();
    expect(screen.getByText(/Nenhuma referência é aprovada automaticamente/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Iniciar monitoramento" }));
    expect(toggle).toHaveBeenCalledWith(true);
  });
  it("offers pause, queues checks and explains infeasible interval without overlap", async () => {
    const check = vi.fn();
    const toggle = vi.fn();
    render(
      <MonitoringPanel
        status={{
          ...status,
          control: { enabled: true, generation: 2 },
          eligible_count: 1,
          interval_seconds: 300,
          service: { ...status.service, warning: "interval_capacity_exceeded" },
        }}
        rows={[]}
        disabled={false}
        onToggle={toggle}
        onCheck={check}
      />,
    );
    expect(screen.getByText(/intervalo de 300s/)).toBeInTheDocument();
    expect(screen.getByText(/verificações não são sobrepostas/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Pausar monitoramento" }));
    await userEvent.click(screen.getByRole("button", { name: "Verificar elegíveis agora" }));
    expect(toggle).toHaveBeenCalledWith(false);
    expect(check).toHaveBeenCalledOnce();
  });
});
describe("notification event journal", () => {
  it("distingue visualmente causas operacionais com impacto e solução", () => {
    const causes = [
      ["auth_required", "auth", "Autenticação"],
      ["network", "connectivity", "Rede e disponibilidade"],
      ["forbidden", "permission", "Permissão"],
      ["sync_failed", "sync", "Sincronização"],
      [
        "rate_limited|Sheets API / spreadsheets.get: cota temporária",
        "quota",
        "Limite de consultas Google",
      ],
    ];
    const items: NotificationRecord[] = causes.map(([diagnostic_code], index) => ({
      id: `notice-${index}`,
      event_id: `event-${index}`,
      source: "operational",
      spreadsheet_id: "fixture_file",
      spreadsheet_name: "Synthetic sheet",
      sheet_id: null,
      sheet_title: null,
      severity: "warning",
      diagnostic_code,
      diagnosis: `Causa: consulta interrompida. Impacto: verificação incompleta. Solução: aguarde a recuperação. ${diagnostic_code.startsWith("rate_limited|") ? diagnostic_code.split("|")[1] : ""}`,
      detected_at: "2026-11-02T12:00:00Z",
      state: "retry",
      attempt_count: 1,
      last_attempt_ms: 1,
      submitted_ms: null,
      group_id: `group-${index}`,
    }));
    render(
      <NotificationHistory
        page={{ items, total: 5, offset: 0, limit: 25 }}
        disabled={false}
        onPage={vi.fn()}
        onOpen={vi.fn()}
      />,
    );
    for (const [, category, label] of causes) {
      expect(screen.getByText(label).closest("li")).toHaveAttribute(
        "data-error-category",
        category,
      );
    }
    expect(screen.getAllByText(/Impacto:.*Solução:/)).toHaveLength(5);
    expect(screen.getByText(/Sheets API \/ spreadsheets.get: cota temporária/)).toBeInTheDocument();
  });
  it("keeps distinct events in the same group and never claims display or reading", async () => {
    const base: NotificationRecord = {
      id: "notice-a",
      event_id: "event-a",
      source: "integrity",
      spreadsheet_id: "fixture_file",
      spreadsheet_name: "Synthetic sheet",
      sheet_id: 7,
      sheet_title: "Data",
      severity: "info",
      diagnosis: "Conteúdo alterado",
      diagnostic_code: "value_changed",
      detected_at: "2026-11-02T12:00:00Z",
      state: "dispatch_submitted",
      attempt_count: 1,
      last_attempt_ms: 1,
      submitted_ms: 1,
      group_id: "same-group",
    };
    const second = {
      ...base,
      id: "notice-b",
      event_id: "event-b",
      diagnosis: "Estrutura alterada",
      state: "suppressed_by_preference",
      attempt_count: 0,
    };
    const open = vi.fn();
    render(
      <NotificationHistory
        page={{ items: [base, second], total: 2, offset: 0, limit: 25 }}
        disabled={false}
        onPage={vi.fn()}
        onOpen={open}
      />,
    );
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText(/Submissão nativa registrada/)).toBeInTheDocument();
    expect(screen.getByText(/Suprimida pela preferência/)).toBeInTheDocument();
    expect(screen.getByText(/não confirma exibição nem leitura/)).toBeInTheDocument();
    await userEvent.click(screen.getAllByRole("button", { name: "Abrir evento" })[1]);
    expect(open).toHaveBeenCalledWith(second);
  });
});
