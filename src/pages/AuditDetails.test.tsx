import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AuditDetails } from "./AuditDetails";
import type { Activity } from "@/types/management";
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), state: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@/services/platform", () => ({ desktop: true }));
vi.mock("@/services/google", async (original) => ({
  ...(await original<Record<string, unknown>>()),
  getIntegrationState: mocks.state,
}));
afterEach(cleanup);
const event: Activity = {
  event_id: "one",
  check_id: "check",
  spreadsheet_id: "file",
  sheet_id: 0,
  spreadsheet_name: "Fixture",
  sheet_title: "Dados",
  kind: "violation",
  category: "integrity",
  severity: "critical",
  status: "offline",
  removed: false,
  rule_id: "header",
  location: null,
  before: null,
  after: null,
  detected_at: "2026-11-01T12:00:00Z",
  fingerprint: "fixture",
  diagnosis: "protected_header_changed_or_column_missing",
};
it("não revela auditoria de uma sessão substituída", async () => {
  const session = { generation: 1, auth: { account: { subject: "account-a" } } };
  mocks.state
    .mockResolvedValueOnce(session)
    .mockResolvedValue({ generation: 2, auth: { account: { subject: "account-b" } } });
  mocks.invoke.mockResolvedValue({
    generation: 1,
    account_subject: "account-a",
    data: {
      responsible: "Responsável não identificado",
      note: "PRIVATE_NOTE",
      records: [{ actor: "private@example.test", occurred_at: event.detected_at, action: "edit" }],
      truncated: false,
    },
  });
  render(<AuditDetails event={event} />);
  expect(mocks.invoke).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", { name: "Consultar auditoria Workspace" }));
  await waitFor(() => expect(mocks.state).toHaveBeenCalledTimes(2));
  expect(screen.queryByText(/private@example/)).not.toBeInTheDocument();
  expect(screen.queryByText("PRIVATE_NOTE")).not.toBeInTheDocument();
});
describe("Contexto da auditoria", () => {
  it("separa editor do arquivo de responsabilidade pela regra", async () => {
    mocks.state.mockResolvedValue({ generation: 1, auth: { account: { subject: "account-a" } } });
    mocks.invoke.mockResolvedValue({
      generation: 1,
      account_subject: "account-a",
      data: {
        responsible: "Responsável não identificado",
        note: "Proximidade temporal não comprova responsabilidade.",
        records: [
          {
            actor: "editor@example.test",
            occurred_at: event.detected_at,
            action: "Edição do arquivo (sem vínculo comprovado com esta regra)",
          },
        ],
        truncated: false,
      },
    });
    render(<AuditDetails event={event} />);
    await userEvent.click(screen.getByRole("button", { name: "Consultar auditoria Workspace" }));
    await screen.findByText(/editor@example.test/);
    expect(screen.getByText(/Proximidade temporal não comprova/)).toBeInTheDocument();
    expect(screen.getByText(/sem vínculo comprovado/)).toBeInTheDocument();
  });
});
