import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { GooglePage } from "./GooglePage";
import { integrationErrorMessage } from "@/services/google";
import type { IntegrationState, SheetAccessResult } from "@/types/google";
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true, invoke: vi.fn() }));
const mock = vi.mocked(invoke);
let state: IntegrationState;
const connected = (): IntegrationState => ({
  ...state,
  configured: true,
  client_id: "fixture.apps.googleusercontent.com",
  generation: 2,
  auth: {
    status: "connected",
    account: { subject: "fixture_1", email: "fixture@example.invalid", name: "Synthetic account" },
    granted_scopes: [
      "openid",
      "https://www.googleapis.com/auth/spreadsheets.readonly",
      "https://www.googleapis.com/auth/script.projects.readonly",
    ],
    expires_at: 12345,
  },
});
beforeEach(() => {
  state = {
    generation: 0,
    configured: false,
    client_id: null,
    auth: { status: "unconfigured", account: null, granted_scopes: [], expires_at: null },
    drive_metadata_opt_in: false,
    real_validation: "pending",
  };
  mock.mockReset();
  mock.mockImplementation(async (command) => {
    if (command === "integration_get_state") return structuredClone(state);
    throw { code: "unconfigured" };
  });
});
afterEach(cleanup);
it("does not enable Google reads before configuration and authorization", async () => {
  render(<GooglePage />);
  await screen.findByText("Cliente não configurado");
  expect(screen.getByRole("button", { name: "Conectar conta Google" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Testar acesso" })).toBeDisabled();
  expect(screen.queryByText("Synthetic account")).not.toBeInTheDocument();
  expect(mock.mock.calls.map((c) => c[0])).toEqual(["integration_get_state"]);
});
it("clears the secret field immediately and displays only a safe consent error", async () => {
  render(<GooglePage />);
  await screen.findByText("Cliente não configurado");
  const user = userEvent.setup();
  await user.type(
    screen.getByLabelText("Client ID do aplicativo Desktop"),
    "fixture.apps.googleusercontent.com",
  );
  const secret = screen.getByLabelText(/Client secret do Desktop/);
  await user.type(secret, "synthetic-secret");
  mock.mockImplementation(async (command) => {
    if (command === "integration_configure") {
      state = {
        ...state,
        configured: true,
        client_id: "fixture.apps.googleusercontent.com",
        generation: 1,
        auth: { ...state.auth, status: "disconnected" },
      };
      return structuredClone(state);
    }
    if (command === "integration_connect")
      throw { code: "consent_denied", message: "synthetic-secret" };
    return structuredClone(state);
  });
  await user.click(screen.getByRole("button", { name: "Salvar cliente Desktop" }));
  await screen.findByText("Conta desconectada");
  expect(secret).toHaveValue("");
  await user.click(screen.getByRole("button", { name: "Conectar conta Google" }));
  await screen.findByRole("alert");
  expect(screen.getByRole("alert")).not.toHaveTextContent("synthetic-secret");
  expect(screen.getByRole("alert")).toHaveTextContent("consentimento foi negado");
});
function sheetResult(): SheetAccessResult {
  return {
    generation: 2,
    account_subject: "fixture_1",
    spreadsheet_id: "sheet_1",
    sheets: {
      status: "success",
      error: null,
      data: {
        spreadsheet_id: "sheet_1",
        title: "Synthetic sheet",
        sheets: [{ sheet_id: 0, title: "Dados", index: 0, hidden: false }],
      },
    },
    values: {
      status: "success",
      error: null,
      data: {
        spreadsheet_id: "sheet_1",
        values: [{ range: "A1", major_dimension: "ROWS", values: [[2]] }],
        formulas: [{ range: "A1", major_dimension: "ROWS", values: [["=1+1"]] }],
      },
    },
    drive: { status: "not_requested", data: null, error: null },
  };
}
it("renders independent real-command results and removes them on logout", async () => {
  state = connected();
  mock.mockImplementation(async (command) => {
    if (command === "integration_test_sheet") return sheetResult();
    if (command === "integration_disconnect") {
      state = {
        ...state,
        generation: 3,
        auth: { status: "disconnected", account: null, granted_scopes: [], expires_at: null },
      };
    }
    return structuredClone(state);
  });
  render(<GooglePage />);
  await screen.findByText("Synthetic account");
  const user = userEvent.setup();
  await user.type(
    screen.getByLabelText("URL da planilha Google Sheets"),
    "https://docs.google.com/spreadsheets/d/sheet_1/edit",
  );
  await user.click(screen.getByRole("button", { name: "Testar acesso" }));
  await screen.findByText("Synthetic sheet");
  expect(screen.getByText("=1+1")).toBeInTheDocument();
  expect(screen.getByText("Não solicitado")).toBeInTheDocument();
  expect(mock).toHaveBeenCalledWith("integration_test_sheet", {
    input: { url: "https://docs.google.com/spreadsheets/d/sheet_1/edit", ranges: ["A1:D10"] },
  });
  await user.click(screen.getByRole("button", { name: "Desconectar" }));
  await screen.findByText("Conta desconectada");
  expect(screen.queryByText("Synthetic sheet")).not.toBeInTheDocument();
  expect(screen.queryByText("=1+1")).not.toBeInTheDocument();
  expect(mock.mock.calls.some((c) => c[0] === "integration_revoke")).toBe(false);
});
it("rejects a result from another account and requires explicit revocation confirmation", async () => {
  state = connected();
  mock.mockImplementation(async (command) => {
    if (command === "integration_test_sheet")
      return { ...sheetResult(), account_subject: "other_account" };
    if (command === "integration_revoke") {
      state = {
        ...state,
        generation: 3,
        auth: { status: "disconnected", account: null, granted_scopes: [], expires_at: null },
      };
    }
    return structuredClone(state);
  });
  render(<GooglePage />);
  await screen.findByText("Synthetic account");
  const user = userEvent.setup();
  await user.type(
    screen.getByLabelText("URL da planilha Google Sheets"),
    "https://docs.google.com/spreadsheets/d/sheet_1/edit",
  );
  await user.click(screen.getByRole("button", { name: "Testar acesso" }));
  await screen.findByRole("alert");
  expect(screen.queryByText("Synthetic sheet")).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Revogar autorização no Google" }));
  const dialog = screen.getByRole("dialog");
  const confirm = within(dialog).getByRole("button", { name: "Confirmar revogação" });
  expect(confirm).toBeDisabled();
  await user.click(within(dialog).getByRole("checkbox"));
  await user.click(confirm);
  await waitFor(() => expect(mock).toHaveBeenCalledWith("integration_revoke", { confirmed: true }));
});
it("never renders an arbitrary provider error body", () => {
  expect(integrationErrorMessage({ code: "unknown", message: "synthetic-secret" })).not.toContain(
    "synthetic-secret",
  );
  expect(integrationErrorMessage({ kind: "forbidden", message: "synthetic-secret" })).toContain(
    "Acesso negado",
  );
});
