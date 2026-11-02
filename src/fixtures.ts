import type { AppSettings, FoundationDashboard } from "@/types/foundation";
export type { AppSettings, FoundationDashboard } from "@/types/foundation";
export const defaultSettings: AppSettings = {
  interval_seconds: 60,
  history_days: 90,
  start_with_windows: false,
};
export function demoDashboard(): FoundationDashboard {
  return {
    is_demo: true,
    database_version: 1,
    settings: { ...defaultSettings },
    spreadsheets: [
      {
        id: "demo-comercial",
        name: "Indicadores comerciais",
        url: "demo://planilhas/comercial",
        status: "online",
        selected_sheets: ["Resumo", "Vendas"],
        last_checked_at: "2026-11-01T18:30:00-03:00",
        is_demo: true,
      },
      {
        id: "demo-financeiro",
        name: "Planejamento financeiro",
        url: "demo://planilhas/financeiro",
        status: "offline",
        selected_sheets: ["Orçamento"],
        last_checked_at: "2026-11-01T18:30:00-03:00",
        is_demo: true,
      },
      {
        id: "demo-operacao",
        name: "Operação e entregas",
        url: "demo://planilhas/operacao",
        status: "checking",
        selected_sheets: [],
        last_checked_at: null,
        is_demo: true,
      },
      {
        id: "demo-equipe",
        name: "Acompanhamento da equipe",
        url: "demo://planilhas/equipe",
        status: "indeterminate",
        selected_sheets: ["Equipe"],
        last_checked_at: "2026-11-01T18:29:00-03:00",
        is_demo: true,
      },
    ],
    activities: [
      {
        id: "demo-event-1",
        spreadsheet_id: "demo-comercial",
        spreadsheet_name: "Indicadores comerciais",
        occurred_at: "2026-11-01T18:30:00-03:00",
        category: "content",
        diagnosis:
          "Simulação: valor comum alterado em Vendas!B2. Alteração permitida; integridade permanece Online. Notificação demonstrativa registrada.",
        is_demo: true,
      },
      {
        id: "demo-event-2",
        spreadsheet_id: "demo-financeiro",
        spreadsheet_name: "Planejamento financeiro",
        occurred_at: "2026-11-01T18:28:00-03:00",
        category: "structure",
        diagnosis:
          "Simulação: coluna protegida excluída na aba Orçamento. Falha estrutural confirmada no cenário fictício.",
        is_demo: true,
      },
      {
        id: "demo-event-3",
        spreadsheet_id: "demo-equipe",
        spreadsheet_name: "Acompanhamento da equipe",
        occurred_at: "2026-11-01T18:25:00-03:00",
        category: "operational",
        diagnosis:
          "Simulação: indisponibilidade temporária. Não foi possível confirmar a integridade no cenário fictício.",
        is_demo: true,
      },
    ],
  };
}
export const baselineExample = {
  is_demo: true,
  notice: "Exemplo didático. Nenhuma baseline real foi coletada ou aprovada.",
  spreadsheet: "Planilha fictícia",
  sheets: [{ name: "Resumo", protected_columns: ["A", "B"], critical_formula: "=SUM(B2:B10)" }],
  version: 1,
};
