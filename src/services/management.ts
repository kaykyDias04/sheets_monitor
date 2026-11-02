import { invoke } from "@tauri-apps/api/core";
import { desktop } from "./platform";
import { engineError } from "./integrity";
import type {
  EngineEnvelope,
  BaselineSummary,
  ValidationResult,
  MonitorScope,
  BaselineVersion,
} from "@/types/integrity";
import type {
  Activity,
  ApprovalInput,
  BaselineComparison,
  DashboardSummary,
  Diagnostics,
  EventFilter,
  Mutation,
  OperationalPreview,
  Page,
  Preferences,
  RegisteredSpreadsheet,
  RemoveInput,
  RulesInput,
  ScriptSummary,
  TabsInput,
} from "@/types/management";
import type { DiscoveryPage, DiscoveryQuery } from "@/types/discovery";
import type { IntegrationState } from "@/types/google";

function command<T>(name: string, input?: unknown): Promise<EngineEnvelope<T>> {
  if (!desktop) return Promise.reject({ code: "desktop_required" });
  return invoke(name, input === undefined ? {} : { input });
}
export const getDashboard = () => command<DashboardSummary>("get_dashboard_summary");
export const listRegistered = () =>
  command<RegisteredSpreadsheet[]>("list_registered_spreadsheets");
export const discover = (input: DiscoveryQuery) =>
  command<DiscoveryPage>("discover_spreadsheets", input);
export const register = (url: string, expected_generation: number) =>
  command<RegisteredSpreadsheet>("register_spreadsheet", { url, expected_generation });
export const remove = (input: RemoveInput) => command<void>("remove_spreadsheet", input);
export const pause = (input: Mutation) =>
  command<RegisteredSpreadsheet>("pause_spreadsheet", input);
export const resume = (input: Mutation) =>
  command<RegisteredSpreadsheet>("resume_spreadsheet", input);
export const refreshTabs = (spreadsheet_id: string) =>
  command<RegisteredSpreadsheet>("list_spreadsheet_tabs", { spreadsheet_id });
export const saveTabs = (input: TabsInput) =>
  command<RegisteredSpreadsheet>("update_monitored_tabs", input);
export const saveRules = (input: RulesInput) =>
  command<RegisteredSpreadsheet>("update_integrity_rules", input);
export const proposeRules = (input: Mutation) =>
  command<MonitorScope>("propose_integrity_rules", input);
export const previewBaseline = (input: Mutation) =>
  command<OperationalPreview>("preview_baseline", input);
export const approveBaseline = (input: ApprovalInput) =>
  command<BaselineVersion>("approve_baseline", input);
export const listBaselines = (spreadsheet_id: string, offset = 0) =>
  command<Page<BaselineSummary>>("list_baseline_versions", { spreadsheet_id, offset, limit: 10 });
export const compareBaselines = (spreadsheet_id: string, before_id: string, after_id: string) =>
  command<BaselineComparison>("compare_baseline_versions", { spreadsheet_id, before_id, after_id });
export const diagnostics = (spreadsheet_id: string) =>
  command<Diagnostics>("get_spreadsheet_diagnostics", { spreadsheet_id });
export const events = (input: EventFilter) => command<Page<Activity>>("list_change_events", input);
export const scripts = (spreadsheet_id: string) =>
  command<ScriptSummary[]>("list_associated_scripts", { spreadsheet_id });
export const check = (input: Mutation) => command<ValidationResult>("run_spreadsheet_check", input);
export const getPreferences = () => command<Preferences>("get_management_preferences");
export const savePreferences = (preferences: Preferences, expected_generation: number) =>
  command<Preferences>("save_management_preferences", { preferences, expected_generation });
export const openSpreadsheet = (spreadsheet_id: string) =>
  command<void>("open_spreadsheet", { spreadsheet_id });
export const setDriveOptIn = (enabled: boolean): Promise<IntegrationState> => {
  if (!desktop) return Promise.reject({ code: "desktop_required" });
  return invoke("integration_set_drive_opt_in", { enabled });
};
const messages: Record<string, string> = {
  duplicate: "Esta planilha j\u00e1 est\u00e1 cadastrada nesta conta.",
  capacity:
    "O limite de 30 planilhas cadastradas foi atingido. Remova um cadastro para selecionar outro arquivo.",
  configuration_conflict: "A configura\u00e7\u00e3o mudou. Atualize os dados e revise novamente.",
  not_registered: "O cadastro n\u00e3o est\u00e1 dispon\u00edvel nesta conta.",
  invalid_tabs:
    "Selecione abas de grade dispon\u00edveis. O limite \u00e9 de 1.024 abas e 20 mil c\u00e9lulas por escopo; a capacidade de 30 se aplica aos arquivos cadastrados.",
  tabs_scope_mismatch:
    "Salve regras para todas as abas selecionadas, usando suas identidades atuais.",
  script_not_authorized:
    "Consulte e associe o projeto Apps Script nesta conta antes de inclu\u00ed-lo nas regras.",
  confirmation_required:
    "Confirme a remo\u00e7\u00e3o do cadastro e escolha o tratamento do hist\u00f3rico.",
  invalid_history_choice: "Escolha se deseja preservar ou excluir as atividades.",
  invalid_preferences:
    "Revise as prefer\u00eancias. Inicializa\u00e7\u00e3o autom\u00e1tica e execu\u00e7\u00e3o minimizada estar\u00e3o dispon\u00edveis posteriormente.",
  invalid_filter: "Revise os filtros, datas e pagina\u00e7\u00e3o.",
  invalid_page: "A p\u00e1gina solicitada n\u00e3o est\u00e1 dispon\u00edvel.",
  baseline_not_found: "A refer\u00eancia solicitada n\u00e3o pertence a esta planilha.",
  configuration_pending:
    "Conclua a sele\u00e7\u00e3o de abas, salve regras v\u00e1lidas e aprove uma refer\u00eancia antes de verificar.",
};
export function managementError(error: unknown) {
  const code =
    error && typeof error === "object" ? (error as Record<string, unknown>).code : undefined;
  return typeof code === "string" && messages[code] ? messages[code] : engineError(error);
}
