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
