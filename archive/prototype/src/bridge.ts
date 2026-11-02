import { invoke, isTauri } from "@tauri-apps/api/core";
export type Mode = "fixture" | "google";
export interface Settings {
  interval_seconds: number;
  mode: Mode;
}
export interface Sheet {
  id: string;
  spreadsheet_id?: string;
  name: string;
  script_ids: string[];
  status: "pending" | "healthy" | "changed" | "error";
  baseline_approved: boolean;
  last_checked: string | null;
  last_error: string | null;
}
export interface ChangeEvent {
  id: string;
  sheet_id: string;
  sheet_name: string;
  detected_at: string;
  path: string;
  kind: "added" | "removed" | "changed";
  before: unknown;
  after: unknown;
}
export interface Notification {
  id: string;
  event_id: string;
  status: "pending" | "sent" | "failed";
  attempts: number;
}
export interface ScriptFinding {
  sheet_id: string;
  sheet_name: string;
  script_id: string;
  rule: string;
  file: string;
  line: number;
  severity: string;
  message: string;
}
export interface Dashboard {
  sheets: Sheet[];
  events: ChangeEvent[];
  notifications: Notification[];
  settings: Settings;
  connected: boolean;
  client_id: string;
  script_findings?: ScriptFinding[];
}
export interface BaselinePreview {
  snapshot: unknown;
  fingerprint: string;
}
export interface AddSheet {
  id: string;
  name: string;
  script_ids: string[];
}
export const desktop = isTauri();
const stamp = () => new Date().toISOString();
const initial = (): Dashboard => ({
  sheets: [
    {
      id: "demo-1",
      spreadsheet_id: "fixture-sales",
      name: "Indicadores comerciais",
      script_ids: ["fixture-sales-script"],
      status: "changed",
      baseline_approved: true,
      last_checked: stamp(),
      last_error: null,
    },
    {
      id: "demo-2",
      spreadsheet_id: "fixture-ops",
      name: "Operação · acompanhamento",
      script_ids: [],
      status: "healthy",
      baseline_approved: true,
      last_checked: stamp(),
      last_error: null,
    },
    {
      id: "demo-3",
      spreadsheet_id: "fixture-budget",
      name: "Planejamento financeiro",
      script_ids: [],
      status: "pending",
      baseline_approved: false,
      last_checked: null,
      last_error: null,
    },
  ],
  events: [
    {
      id: "demo-event-1",
      sheet_id: "demo-1",
      sheet_name: "Indicadores comerciais",
      detected_at: stamp(),
      path: "/sheets/0/data/0/rowData/2/values/1/userEnteredValue/numberValue",
      kind: "changed",
      before: 12450,
      after: 13820,
    },
    {
      id: "demo-event-2",
      sheet_id: "demo-1",
      sheet_name: "Indicadores comerciais",
      detected_at: stamp(),
      path: "/scripts/fixture-sales-script/files/0/source",
      kind: "changed",
      before: "function atualizar() { return 1; }",
      after: "function atualizar() { MailApp.getRemainingDailyQuota(); return 2; }",
    },
  ],
  notifications: [
    {
      id: "demo-notice-1",
      event_id: "demo-event-1",
      status: "pending",
      attempts: 0,
    },
    {
      id: "demo-notice-2",
      event_id: "demo-event-2",
      status: "pending",
      attempts: 0,
    },
  ],
  settings: { interval_seconds: 300, mode: "fixture" },
  connected: false,
  client_id: "",
  script_findings: [{sheet_id: 'demo-1', sheet_name: 'Indicadores comerciais', script_id: 'fixture-sales-script', rule: 'email', file: 'Code', line: 1, severity: 'info', message: 'Indicador de acesso a envio de e-mail. (Fixture de demonstração.)'}],
});
let demo = initial();
const demoCandidates = new Map<string, BaselinePreview>();
let demoTick = 0;
export function resetDemo() {
  demo = initial();
  demoCandidates.clear();
  demoTick = 0;
}
function previewCommand<T>(command: string, args?: Record<string, unknown>): T {
  switch (command) {
    case "get_dashboard":
      return structuredClone(demo) as T;
    case "add_sheet": {
      const input = args?.input as AddSheet;
      if (demo.sheets.length >= 30)
        throw new Error("O limite é de 30 planilhas.");
      if (demo.sheets.some((s) => s.spreadsheet_id === input.id))
        throw new Error("Esta planilha já está cadastrada.");
      demo.sheets.push({
        ...input,
        id: crypto.randomUUID(),
        spreadsheet_id: input.id,
        status: "pending",
        baseline_approved: false,
        last_checked: null,
        last_error: null,
      });
      break;
    }
    case "remove_sheet":
      demo.sheets = demo.sheets.filter((s) => s.id !== args?.id);
      break;
    case "preview_baseline": {
      const candidate = {
        snapshot: {
          source: "fixture",
          spreadsheet_id: demo.sheets.find((s) => s.id === args?.id)
            ?.spreadsheet_id,
          sheets: [
            {
              title: "Resumo",
              cells: [
                ["Indicador", "Valor"],
                ["Receita", 13820],
              ],
            },
          ],
          drive: { name: "Fixture local", permissions: [] },
          scripts: [],
          captured_at: stamp(),
        },
        fingerprint: crypto.randomUUID(),
      };
      demoCandidates.set(String(args?.id), candidate);
      return candidate as T;
    }
    case "approve_baseline": {
      const candidate = demoCandidates.get(String(args?.id));
      if (!candidate || candidate.fingerprint !== args?.fingerprint)
        throw new Error("Prévia expirada. Carregue uma nova baseline.");
      const sheet = demo.sheets.find((s) => s.id === args?.id);
      if (sheet) {
        sheet.baseline_approved = true;
        sheet.status = "healthy";
        sheet.last_checked = stamp();
      }
      demoCandidates.delete(String(args?.id));
      break;
    }
    case "run_check": {
      demoTick++;
      for (const sheet of demo.sheets.filter(
        (s) => s.baseline_approved && (!args?.id || s.id === args.id),
      )) {
        const event: ChangeEvent = {
          id: crypto.randomUUID(),
          sheet_id: sheet.id,
          sheet_name: sheet.name,
          detected_at: stamp(),
          path: "/fixture/cells/B2",
          kind: "changed",
          before: 100 + demoTick - 1,
          after: 100 + demoTick,
        };
        demo.events.unshift(event);
        demo.notifications.unshift({
          id: crypto.randomUUID(),
          event_id: event.id,
          status: "pending",
          attempts: 0,
        });
        sheet.status = "changed";
        sheet.last_checked = stamp();
      }
      break;
    }
    case "update_settings": {
      const settings = args?.settings as Settings;
      if (settings.mode === "google")
        throw new Error(
          "A conexão Google está disponível somente no aplicativo desktop.",
        );
      demo.settings = settings;
      break;
    }
    case "connect_google":
      throw new Error(
        "Abra o aplicativo desktop para autorizar sua conta Google. Nenhuma conexão foi realizada.",
      );
    case "disconnect_google":
      demo.connected = false;
      break;
    case "retry_notifications":
      throw new Error(
        "Notificações do Windows estão disponíveis no aplicativo desktop. A fila de demonstração permanece pendente.",
      );
    case "export_report":
      throw new Error(
        "A exportação para arquivo está disponível no aplicativo desktop.",
      );
    default:
      throw new Error(`Comando indisponível na demonstração: ${command}`);
  }
  return undefined as T;
}
export async function call<T = void>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (desktop) return invoke<T>(command, args);
  return previewCommand<T>(command, args);
}
