import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BaselineFlow } from "./BaselineFlow";
import { scopeForTabs } from "./ScopeEditor";
import type { OperationalPreview, RegisteredSpreadsheet } from "@/types/management";

afterEach(cleanup);

const scope = scopeForTabs(null, [17]);
const row: RegisteredSpreadsheet = {
  spreadsheet_id: "fixture-file",
  name: "Teste",
  url: "https://docs.google.com/spreadsheets/d/fixture-file/edit",
  account_email: null,
  paused: false,
  removed: false,
  status: "configuration_pending",
  authorization: "available",
  access_error: null,
  last_checked_at: null,
  baseline_version: null,
  scope,
  scope_hash: "fixture",
  revision: 1,
  tabs: [],
  selected_sheet_ids: [17],
  tabs_refreshed_at: null,
};

const preview: OperationalPreview = {
  revision: 1,
  preview: {
    session_generation: 1,
    snapshot_fingerprint: "fixturehash",
    active_baseline: null,
    snapshot: {
      schema_version: 1,
      id: "capture-one",
      account_subject: "fixture-account",
      spreadsheet_id: "fixture-file",
      capture_started_at: "2026-11-02T12:00:00Z",
      captured_at: "2026-11-02T12:00:00Z",
      scope_hash: "fixture",
      scope,
      sheets: [],
      scripts: [],
      connectivity: "available",
      coverage: {
        complete: true,
        metadata_complete: true,
        atomic_across_services: false,
        issues: [],
      },
    },
  },
};

function view(p = preview, dirty = false) {
  const approve = vi.fn();
  const props = {
    row,
    preview: p,
    versions: [],
    comparison: null,
    offset: 0,
    disabled: false,
    dirty,
    onCapture: vi.fn(),
    onApprove: approve,
    onPage: vi.fn(),
    onCompare: vi.fn(),
    onComparisonClear: vi.fn(),
  };
  return { ...render(<BaselineFlow {...props} />), approve, props };
}

describe("Aprovação visual", () => {
  it("exige confirmação explícita, oculta JSON bruto e encaminha exatamente a prévia revisada", async () => {
    const user = userEvent.setup();
    const { approve } = view();
    const button = screen.getByRole("button", { name: "Aprovar referência versão 1" });
    expect(button).toBeDisabled();
    expect(screen.getByText(/não representa toda a planilha/)).toBeInTheDocument();
    expect(screen.queryByText(/Estrutura capturada e regras propostas/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/"schema_version"/)).not.toBeInTheDocument();
    expect(screen.queryByText(/"capture-one"/)).not.toBeInTheDocument();
    await user.click(screen.getByRole("checkbox", { name: /Revisei o escopo/ }));
    await user.click(button);
    expect(approve).toHaveBeenCalledWith(preview);
  });

  it("recusa aprovação de captura incompleta ou configuração editada e traduz códigos internos", () => {
    const partial = structuredClone(preview);
    partial.preview.snapshot.coverage.complete = false;
    partial.preview.snapshot.coverage.issues = [
      { sheet_id: 17, rule_id: null, critical: true, code: "forbidden" },
    ];
    const { unmount } = view(partial);
    expect(screen.getByRole("checkbox", { name: /Revisei o escopo/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Aprovar referência versão 1" })).toBeDisabled();
    unmount();
    view(preview, true);
    expect(screen.getByRole("button", { name: "Capturar estrutura para revisão" })).toBeDisabled();
    expect(screen.getByText(/Há regras ou abas não salvas/)).toBeInTheDocument();
  });

  it("descarta confirmação quando uma nova captura substitui a prévia", async () => {
    const user = userEvent.setup();
    const { rerender, props } = view();
    await user.click(screen.getByRole("checkbox", { name: /Revisei o escopo/ }));
    expect(screen.getByRole("button", { name: "Aprovar referência versão 1" })).toBeEnabled();
    const next = structuredClone(preview);
    next.preview.snapshot.id = "capture-two";
    rerender(<BaselineFlow {...props} preview={next} />);
    expect(screen.getByRole("button", { name: "Aprovar referência versão 1" })).toBeDisabled();
  });
});
