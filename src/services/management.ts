import { invoke } from "@tauri-apps/api/core";
import { desktop } from "./platform";
import { errorGuidance, extractErrorCode, logErrorDiagnostic } from "./google";
import { engineError } from "./integrity";
import type {
  EngineEnvelope,
  BaselineSummary,
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
  ExportEventsInput,
  ExportEventsResult,
  Mutation,
  OperationalPreview,
  Page,
  Preferences,
  ProposeRulesInput,
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
export const proposeRules = (input: ProposeRulesInput) =>
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
export const exportEvents = (input: ExportEventsInput) =>
  command<ExportEventsResult>("export_change_events", input);
export const scripts = (spreadsheet_id: string) =>
  command<ScriptSummary[]>("list_associated_scripts", { spreadsheet_id });
export const check = (input: Mutation) => command<number>("monitoring_check_file", input);
export const getPreferences = () => command<Preferences>("get_management_preferences");
export const savePreferences = (preferences: Preferences, expected_generation: number) => {
  if (
    !Number.isInteger(preferences.interval_seconds) ||
    preferences.interval_seconds < 60 ||
    preferences.interval_seconds > 600
  )
    return Promise.reject({ code: "invalid_preferences" });
  return command<Preferences>("save_management_preferences", { preferences, expected_generation });
};
export const openSpreadsheet = (spreadsheet_id: string) =>
  command<void>("open_spreadsheet", { spreadsheet_id });
export const setDriveOptIn = (enabled: boolean): Promise<IntegrationState> => {
  if (!desktop) return Promise.reject({ code: "desktop_required" });
  return invoke("integration_set_drive_opt_in", { enabled });
};

const messages: Record<string, string> = {
  duplicate:
    "Esta planilha já está cadastrada nesta conta. Feche a janela de adição para acessá-la na lista principal.",
  capacity:
    "O limite de 30 planilhas cadastradas foi atingido nesta conta. Remova um cadastro existente antes de adicionar outro arquivo.",
  configuration_conflict:
    "Os dados da planilha foram atualizados por outra operação. Revise as informações em tela e tente salvar novamente.",
  monitoring_conflict:
    "O estado do monitoramento mudou enquanto a ação era processada. Atualize a tela e tente novamente.",
  not_registered:
    "Esta planilha não está mais cadastrada nesta conta. Volte para Planilhas e atualize os dados.",
  invalid_tabs:
    "Selecione ao menos uma aba de grade disponível (até 1.024 abas) e desmarque abas removidas antes de salvar.",
  tabs_scope_mismatch:
    "As regras de proteção não correspondem às abas selecionadas. Salve a seleção atual na etapa Abas antes de salvar as regras.",
  script_not_authorized:
    "O projeto Apps Script informado ainda não foi autorizado nesta conta. Acesse a seção Apps Script da planilha, consulte o projeto e tente salvar novamente.",
  confirmation_required:
    "Marque a confirmação de remoção e escolha como tratar o histórico de atividades antes de continuar.",
  invalid_history_choice:
    "Selecione se deseja preservar ou excluir o histórico local de atividades antes de confirmar a remoção.",
  invalid_preferences:
    "Os valores de preferências são inválidos. Informe um intervalo entre 60 e 600 segundos e retenção entre 1 e 3.650 dias.",
  invalid_filter:
    "Os filtros informados são inválidos. Verifique o período de datas selecionado e tente aplicar os filtros novamente.",
  invalid_page:
    "A página solicitada não está mais disponível para esta consulta. Volte para a primeira página e tente novamente.",
  baseline_not_found:
    "A versão de referência selecionada não foi encontrada para esta planilha. Atualize os dados e escolha outra versão.",
  invalid_anchor_row:
    "Informe uma linha âncora válida (entre 1 e 1.000.000) antes de sugerir as regras.",
  anchor_row_out_of_range:
    "A linha âncora informada ultrapassa o total de linhas de uma ou mais abas selecionadas. Informe uma linha existente na planilha ou atualize as abas.",
  configuration_pending:
    "O cadastro desta planilha ainda está pendente. Selecione abas, salve regras de proteção e aprove uma referência antes de verificar.",
  check_failed:
    "Não foi possível agendar a verificação desta planilha no momento. Atualize os dados e tente novamente.",
  native_permission_blocked:
    "As notificações deste aplicativo estão bloqueadas no Windows. Ative as notificações nas configurações do sistema antes de reenviar.",
  native_prepare_failed:
    "Não foi possível preparar o canal de notificações do Windows neste momento. Tente novamente mais tarde.",
  native_setting_unknown:
    "Não foi possível consultar o estado das notificações no Windows. Verifique as configurações de notificações do sistema.",
  unsupported: "Este recurso nativo não é suportado no ambiente atual do Windows.",
  startup_entry_conflict:
    "Já existe um registro diferente de inicialização no Windows. Remova a entrada conflitante nas configurações do sistema antes de ativar.",
  startup_path_too_long:
    "O caminho de instalação atual excede o limite aceito pelo Windows para inicialização automática.",
  startup_path_unsupported:
    "O local atual do executável não permite registro seguro de inicialização automática no Windows.",
  startup_read_failed:
    "Não foi possível consultar o registro de inicialização automática do Windows. Verifique as permissões do usuário.",
  startup_write_failed:
    "Não foi possível gravar a preferência de inicialização automática no Windows. Verifique as permissões do usuário.",
  startup_verification_failed:
    "O Windows não confirmou a gravação da inicialização automática com segurança. Tente novamente.",
  invalid_format:
    "Formato de exportação inválido. Selecione CSV (Excel) ou JSON antes de exportar.",
  export_io_error:
    "Não foi possível gravar o arquivo de exportação na pasta local. Verifique o espaço disponível e as permissões do usuário.",
  manual_update_required:
    "Esta execução utiliza um binário portátil ou instalação administrativa. Baixe o instalador oficial na página da release para atualizar.",
  update_not_staged:
    "Nenhum pacote de atualização verificado está pronto para instalação. Clique em Verificar atualizações.",
  already_up_to_date: "A versão instalada já está atualizada.",
  checks_in_progress:
    "Há verificações de planilhas em andamento. Aguarde a conclusão da análise atual e tente reiniciar novamente.",
  integrity_mismatch:
    "A verificação de integridade (SHA-256) do instalador falhou. Execute uma nova verificação de atualizações.",
  invalid_version: "A versão informada para adiamento é inválida.",
};

export function managementError(error: unknown, context?: string) {
  const code = extractErrorCode(error);
  if (code === "configuration_pending" && context === "ler os cabeçalhos da planilha") {
    logErrorDiagnostic(error, context);
    return "Não foi possível identificar cabeçalhos únicos na linha âncora escolhida em todas as abas selecionadas. Confira o número da linha âncora, atualize as abas ou preencha os cabeçalhos manualmente.";
  }
  if (code && Object.hasOwn(messages, code)) {
    logErrorDiagnostic(error, context);
    const guidance = errorGuidance(error);
    return guidance
      ? `${messages[code]} Impacto: ${guidance.impact} Solução: ${guidance.solution}`
      : messages[code];
  }
  return engineError(error, context);
}
