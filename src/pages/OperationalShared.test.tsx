import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import {
  EventDiagnostic,
  EventList,
  StatusBadge,
  activityTone,
  dateLabel,
  diagnosticText,
  formatLocalDateTime,
} from "./OperationalShared";
import { ScriptCoverage } from "./ScriptCoverage";
import type { Activity } from "@/types/management";
afterEach(cleanup);
const event: Activity = {
  event_id: "test-event",
  check_id: "test-check",
  spreadsheet_id: "fixture-file",
  sheet_id: 17,
  spreadsheet_name: "Planilha de teste",
  sheet_title: "Receitas",
  kind: "inconclusive",
  category: "operational",
  severity: "warning",
  status: "indeterminate",
  removed: false,
  rule_id: "key",
  location: "A2",
  before: null,
  after: null,
  detected_at: "2026-11-02T12:00:00Z",
  fingerprint: "fixture",
  diagnosis: "Movimento não comprovado",
};
describe("Diagnóstico operacional", () => {
  it("exibe estados reais distintos de configuração pendente e não presume Online", () => {
    render(
      <>
        <StatusBadge status="configuration_pending" />
        <StatusBadge status="not_checked" />
        <StatusBadge status="online" />
        <StatusBadge status="offline" />
        <StatusBadge status="indeterminate" />
      </>,
    );
    for (const name of [
      "Configuração pendente",
      "Ainda não verificada",
      "Online",
      "Offline",
      "Indeterminado",
    ])
      expect(screen.getByText(name)).toBeInTheDocument();
    expect(dateLabel(null)).toBe("Ainda não verificada");
  });
  it("expõe incerteza, ausência de antes/depois e autoria desconhecida", () => {
    render(<EventDiagnostic event={event} onClose={() => undefined} />);
    const dialog = screen.getByRole("dialog", { name: "Diagnóstico da alteração" });
    expect(dialog).toHaveTextContent("Não foi possível determinar");
    expect(dialog).toHaveTextContent("Não disponível neste evento");
    expect(dialog).toHaveTextContent("A autoria da alteração não é determinada");
    expect(dialog).toHaveTextContent("Nenhuma correção é executada automaticamente");
  });
  it("mostra dependências dinâmicas como não verificáveis e não promete descoberta total", () => {
    render(
      <ScriptCoverage
        scripts={[
          {
            script_id: "fixture-script",
            name: "Automação de teste",
            authorization: "success",
            critical: false,
            last_queried_at: null,
            analysis: {
              script_id: "fixture-script",
              source_hash: "fixture-hash",
              files: [{ name: "Main", file_type: "SERVER_JS", hash: "fixture" }],
              analysis: {
                complete: false,
                parser: "fixture-parser",
                limitations: ["dynamic sheet name"],
                dependencies: [
                  {
                    file: "Main",
                    line: 12,
                    kind: "sheet_by_name",
                    coverage: "dynamic",
                    detail_code: "dynamic_argument",
                    sheet_name: null,
                  },
                ],
              },
            },
          },
        ]}
      />,
    );
    expect(
      screen.getByText("Cobertura parcial — há dependências não verificáveis"),
    ).toBeInTheDocument();
    expect(screen.getByText("Não verificável")).toBeInTheDocument();
    expect(
      screen.getByText(/descoberta automática de todos os projetos vinculados não é garantida/),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Main")).toHaveLength(2);
  });
  it("mostra arquivos e dependências da captura não aprovada sem reutilizar análise de projeto ausente", () => {
    const captured = {
      script_id: "fixture-script",
      source_hash: "preview-hash",
      files: [{ name: "Main", file_type: "SERVER_JS", hash: "fixture" }],
      analysis: {
        complete: false,
        parser: "fixture-parser",
        limitations: ["dynamic reference"],
        dependencies: [
          {
            file: "Main",
            line: 12,
            kind: "sheet_by_name",
            coverage: "dynamic",
            detail_code: "dynamic_argument",
            sheet_name: null,
          },
        ],
      },
    };
    const associated = {
      script_id: "fixture-script",
      name: "Projeto de teste",
      authorization: "success",
      critical: false,
      last_queried_at: null,
      analysis: null,
    };
    const { rerender } = render(
      <ScriptCoverage scripts={[associated]} previewScripts={[captured]} />,
    );
    expect(
      screen.getByText("Análise da captura em revisão — ainda não aprovada"),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Main")).toHaveLength(2);
    expect(screen.getByText("Não verificável")).toBeInTheDocument();
    rerender(
      <ScriptCoverage scripts={[{ ...associated, analysis: captured }]} previewScripts={[]} />,
    );
    expect(screen.queryByText("Main")).not.toBeInTheDocument();
    expect(
      screen.getByText(/Ainda não há análise para este projeto no escopo atual/),
    ).toBeInTheDocument();
  });
  it("diferencia indicadores vermelhos, verdes e neutros, consolida contagens e formata datas locais", () => {
    const formatted = formatLocalDateTime("2026-11-04T15:09:07Z");
    expect(formatted).toMatch(/^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}:\d{2}$/);
    expect(diagnosticText("value_changed:count=4")).toBe(
      "4 valores foram alterados nesta verificação.",
    );
    const items: Activity[] = [
      {
        ...event,
        event_id: "ev-fail",
        kind: "violation",
        category: "structure",
        severity: "critical",
        status: "offline",
        issue_status: "active",
        diagnosis: "protected_header_changed_or_column_missing",
      },
      {
        ...event,
        event_id: "ev-res",
        kind: "recovered",
        category: "integrity",
        severity: "info",
        status: "online",
        issue_status: "resolved",
        diagnosis: "critical_rule_recovered",
      },
      {
        ...event,
        event_id: "ev-mod",
        kind: "value_changed",
        category: "content",
        severity: "info",
        status: "online",
        issue_status: null,
        diagnosis: "value_changed:count=3",
      },
    ];
    expect(items.map(activityTone)).toEqual(["failure", "resolved", "neutral"]);
    const { container } = render(<EventList events={items} onOpen={() => undefined} />);
    const dots = container.querySelectorAll(".op-event-indicator > .op-event-dot");
    expect(dots).toHaveLength(3);
    expect(dots[0]).toHaveClass("op-tone-failure");
    expect(dots[1]).toHaveClass("op-tone-resolved");
    expect(dots[2]).toHaveClass("op-tone-neutral");
    expect(screen.getByText(/Ativo/)).toBeInTheDocument();
    expect(screen.getByText(/Resolvido/)).toBeInTheDocument();
    expect(screen.getByText("3 valores foram alterados nesta verificação.")).toBeInTheDocument();
  });
});
