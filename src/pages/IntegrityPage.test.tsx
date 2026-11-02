import { beforeEach, afterEach, it, expect, vi } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { IntegrityPage } from "./IntegrityPage";
import type {
  CapturePreview,
  MonitorScope,
  EngineEnvelope,
  ValidationResult,
} from "@/types/integrity";
import type { IntegrationState } from "@/types/google";
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true, invoke: vi.fn() }));
const mock = vi.mocked(invoke);
let auth: IntegrationState;
let preview: CapturePreview;
const scope: MonitorScope = {
  schema_version: 1,
  sheets: [
    {
      sheet_id: 7,
      protect_title: true,
      protect_position: false,
      regions: [
        {
          id: "data",
          range: { start_row: 0, end_row: 10, start_col: 0, end_col: 2 },
          header_row: 0,
          columns: [
            { id: "id", header: "ID", position: 0, required: true, protect_position: true },
          ],
          key_column: "id",
          critical: true,
          allow_record_deletion: true,
          formulas: [],
        },
      ],
    },
  ],
  scripts: [],
  named_range_ids: [],
  snapshot_retention_days: 7,
  history_retention_days: 90,
};
function envelope<T>(data: T): EngineEnvelope<T> {
  return {
    generation: auth.generation,
    account_subject: auth.auth.account!.subject,
    data: structuredClone(data),
  };
}
beforeEach(() => {
  auth = {
    configured: true,
    client_id: "fixture.apps.googleusercontent.com",
    generation: 1,
    auth: {
      status: "connected",
      account: { subject: "fixture_subject", email: null, name: null },
      granted_scopes: [],
      expires_at: 1,
    },
    drive_metadata_opt_in: false,
  };
  preview = {
    session_generation: 1,
    snapshot_fingerprint: "a".repeat(64),
    active_baseline: null,
    snapshot: {
      schema_version: 1,
      id: "snapshot_fixture",
      account_subject: "fixture_subject",
      spreadsheet_id: "sheet_fixture",
      capture_started_at: "2026-01-01T00:00:00Z",
      captured_at: "2026-01-01T00:00:01Z",
      scope_hash: "f".repeat(64),
      scope,
      sheets: [
        {
          sheet_id: 7,
          title: "Synthetic data",
          index: 0,
          row_count: 10,
          column_count: 2,
          regions: [],
        },
      ],
      coverage: {
        metadata_complete: true,
        complete: true,
        issues: [],
        atomic_across_services: false,
      },
      scripts: [],
      connectivity: "reachable",
    },
  };
  mock.mockReset();
  mock.mockImplementation(async (command) => {
    if (command === "integration_get_state") return structuredClone(auth);
    if (command === "integrity_propose_scope")
      return envelope({
        scope,
        available_sheets: [{ sheet_id: 7, title: "Synthetic data", index: 0 }],
      });
    if (command === "integrity_capture_snapshot") return envelope(preview);
    if (command === "integrity_create_baseline")
      return envelope({
        id: "baseline_fixture",
        version: 1,
        active: true,
        approved_at: "2026-01-01T00:01:00Z",
        approval_origin: "human:native",
      });
    if (command === "integrity_list_baselines") return envelope([]);
    throw { code: "integrity_invalid_state" };
  });
});
afterEach(cleanup);
async function ready() {
  render(<IntegrityPage />);
  await screen.findByText("Conta conectada");
  return userEvent.setup();
}
async function capture() {
  const user = await ready();
  await user.type(
    screen.getByLabelText("URL da planilha autorizada"),
    "https://docs.google.com/spreadsheets/d/sheet_fixture/edit",
  );
  await user.click(screen.getByRole("button", { name: "Preparar proposta de escopo" }));
  await screen.findByText(/Proposta preparada/);
  await user.click(screen.getByRole("button", { name: "Capturar snapshot" }));
  await screen.findByText("snapshot_fixture");
  return user;
}
it("does not call capture or approval while disconnected", async () => {
  auth = { ...auth, auth: { ...auth.auth, status: "disconnected", account: null } };
  render(<IntegrityPage />);
  await screen.findByText("Conecte a conta na área Google");
  expect(screen.getByRole("button", { name: "Capturar snapshot" })).toBeDisabled();
  expect(mock.mock.calls.every((c) => c[0] === "integration_get_state")).toBe(true);
});
it("binds human approval to the displayed snapshot hash and expected version", async () => {
  const user = await capture();
  const approve = screen.getByRole("button", { name: "Aprovar nova versão de baseline" });
  expect(approve).toBeDisabled();
  expect(mock.mock.calls.some((c) => c[0] === "integrity_create_baseline")).toBe(false);
  await user.click(screen.getByRole("checkbox", { name: /Revisei o conteúdo/ }));
  await user.click(approve);
  await screen.findByText(/Referência versão 1 aprovada/);
  expect(mock).toHaveBeenCalledWith("integrity_create_baseline", {
    input: {
      expected_generation: 1,
      snapshot_id: "snapshot_fixture",
      snapshot_fingerprint: "a".repeat(64),
      expected_version: 0,
      confirmed: true,
    },
  });
  expect(screen.getByRole("checkbox")).not.toBeChecked();
});
it("requires another preview after editing the scope and never approves partial capture", async () => {
  preview.snapshot.coverage.complete = false;
  preview.snapshot.coverage.issues = [
    { sheet_id: 7, rule_id: null, critical: true, code: "network" },
  ];
  const user = await capture();
  expect(screen.getByRole("checkbox", { name: /Revisei o conteúdo/ })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Aprovar nova versão de baseline" })).toBeDisabled();
  await user.type(screen.getByLabelText("Escopo técnico em JSON"), " ");
  expect(screen.queryByText("snapshot_fixture")).not.toBeInTheDocument();
});
it("rejects a captured result after the authenticated session changes", async () => {
  const user = await ready();
  await user.type(
    screen.getByLabelText("URL da planilha autorizada"),
    "https://docs.google.com/spreadsheets/d/sheet_fixture/edit",
  );
  mock.mockImplementation(async (command) => {
    if (command === "integration_get_state") return structuredClone(auth);
    if (command === "integrity_capture_snapshot") {
      const stale = envelope(preview);
      auth = {
        ...auth,
        generation: 2,
        auth: { ...auth.auth, account: { subject: "other_subject", email: null, name: null } },
      };
      return stale;
    }
    throw { code: "integrity_invalid_state" };
  });
  await user.click(screen.getByRole("button", { name: "Capturar snapshot" }));
  await screen.findByRole("alert");
  expect(screen.getByRole("alert")).toHaveTextContent("conta mudou");
  expect(screen.queryByText("snapshot_fixture")).not.toBeInTheDocument();
  expect(screen.getByLabelText("URL da planilha autorizada")).toHaveValue("");
});
it("a JSON syntax error never reaches native capture", async () => {
  const user = await ready();
  await user.type(
    screen.getByLabelText("URL da planilha autorizada"),
    "https://docs.google.com/spreadsheets/d/sheet_fixture/edit",
  );
  await user.clear(screen.getByLabelText("Escopo técnico em JSON"));
  await user.type(screen.getByLabelText("Escopo técnico em JSON"), "invalid");
  await user.click(screen.getByRole("button", { name: "Capturar snapshot" }));
  await screen.findByRole("alert");
  await waitFor(() =>
    expect(mock.mock.calls.some((c) => c[0] === "integrity_capture_snapshot")).toBe(false),
  );
});
it("shows classification produced by Rust separately from connectivity", async () => {
  preview.active_baseline = {
    id: "baseline_fixture",
    version: 1,
    active: true,
    approved_at: "fixture",
    approval_origin: "human:native",
  };
  const original = mock.getMockImplementation()!;
  mock.mockImplementation(async (command, args) => {
    if (command === "integrity_validate_integrity") {
      const data: ValidationResult = {
        report: {
          check_id: "check_fixture",
          snapshot_id: "snapshot_fixture",
          baseline_id: "baseline_fixture",
          account_subject: "fixture_subject",
          spreadsheet_id: "sheet_fixture",
          detected_at: "2026-01-01T01:00:00Z",
          status: "offline",
          connectivity: "reachable",
          sheets: [{ sheet_id: 7, status: "offline" }],
          findings: [
            {
              key: "column",
              sheet_id: 7,
              rule_id: "amount",
              code: "protected_column_missing",
              severity: "critical",
              outcome: "violation",
              expected: "Amount",
              found: null,
              location: "B1",
            },
          ],
          differences: [],
          coverage: preview.snapshot.coverage,
        },
        events: [],
      };
      return envelope(data);
    }
    return original(command, args);
  });
  const user = await capture();
  await user.click(screen.getByRole("button", { name: "Validar integridade da captura" }));
  await screen.findByText("Offline");
  expect(screen.getByText("protected_column_missing")).toBeInTheDocument();
  expect(screen.getByText(/Conectividade: reachable/)).toBeInTheDocument();
});

it("rejects a new-session result even when its envelope matches the final session", async () => {
  const user = await ready();
  await user.type(
    screen.getByLabelText("URL da planilha autorizada"),
    "https://docs.google.com/spreadsheets/d/sheet_fixture/edit",
  );
  mock.mockImplementation(async (command) => {
    if (command === "integration_get_state") return structuredClone(auth);
    if (command === "integrity_capture_snapshot") {
      auth = { ...auth, generation: 2 };
      return envelope({ ...preview, session_generation: 2 });
    }
    throw { code: "integrity_invalid_state" };
  });
  await user.click(screen.getByRole("button", { name: "Capturar snapshot" }));
  await screen.findByRole("alert");
  expect(screen.getByRole("alert")).toHaveTextContent("conta mudou");
  expect(screen.queryByText("snapshot_fixture")).not.toBeInTheDocument();
});
