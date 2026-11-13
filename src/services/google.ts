import { invoke } from "@tauri-apps/api/core";

import { desktop } from "@/services/platform";

import type {
  IntegrationState,
  OAuthConfiguration,
  SheetAccessResult,
  ScriptAccessResult,
} from "@/types/google";

function requireDesktop() {
  if (!desktop) throw { code: "desktop_required" };
}

export async function getIntegrationState(): Promise<IntegrationState> {
  requireDesktop();

  return invoke("integration_get_state");
}

export async function configureIntegration(input: OAuthConfiguration): Promise<IntegrationState> {
  requireDesktop();

  return invoke("integration_configure", { input });
}

export async function connectIntegration(): Promise<IntegrationState> {
  requireDesktop();

  return invoke("integration_connect");
}

export async function disconnectIntegration(): Promise<IntegrationState> {
  requireDesktop();

  return invoke("integration_disconnect");
}

export async function revokeIntegration(): Promise<IntegrationState> {
  requireDesktop();

  return invoke("integration_revoke", { confirmed: true });
}

export async function testSheetAccess(input: {
  url: string;

  ranges: string[];
}): Promise<SheetAccessResult> {
  requireDesktop();

  return invoke("integration_test_sheet", { input });
}

export async function testScriptAccess(input: {
  script_id: string;

  spreadsheet_id?: string;
}): Promise<ScriptAccessResult> {
  requireDesktop();

  return invoke("integration_test_script", { input });
}

export interface ErrorDiagnostic {
  code: string;
  category: "auth" | "permission" | "connectivity" | "quota" | "sync" | "validation" | "internal";
  retryable: boolean;
  context?: string;
}

const errorCategories: Record<string, ErrorDiagnostic["category"]> = {
  auth_required: "auth",
  cloud_unauthorized: "auth",
  authentication_required: "auth",
  unauthorized: "auth",
  reconnect_required: "auth",
  token_expired: "auth",
  invalid_grant: "auth",
  identity_mismatch: "auth",
  stale_session: "auth",
  operation_superseded: "auth",
  auth_busy: "auth",
  cancelled: "auth",
  consent_denied: "auth",
  oauth_denied: "auth",
  access_denied: "auth",
  callback_invalid: "auth",
  authorization_failed: "auth",
  revocation_confirmation_required: "auth",
  unconfigured: "auth",
  oauth_not_configured: "auth",
  not_configured: "auth",
  invalid_configuration: "auth",
  configuration_invalid: "auth",
  invalid_desktop_client_id: "auth",
  vault_corrupt: "auth",
  vault_unavailable: "auth",
  credential_store_unavailable: "auth",
  browser_unavailable: "auth",
  browser_open_failed: "auth",

  forbidden: "permission",
  cloud_forbidden: "permission",
  cloud_opt_in_required: "permission",
  permission_denied: "permission",
  not_found: "permission",
  insufficient_scope: "permission",
  integrity_forbidden: "permission",
  script_not_authorized: "permission",
  native_permission_blocked: "permission",

  network: "connectivity",
  cloud_network: "connectivity",
  cloud_server: "connectivity",
  cloud_rate_limited: "quota",
  sync_busy: "sync",
  cloud_local_changed: "sync",
  cloud_stale_resolution: "sync",
  cloud_storage: "sync",
  timeout: "connectivity",
  rate_limited: "quota",
  quota_exceeded: "quota",
  network_failed: "connectivity",
  server_error: "connectivity",
  service_unavailable: "connectivity",
  configuration_conflict: "sync",
  monitoring_conflict: "sync",
  integrity_baseline_conflict: "sync",
  sync_conflict: "sync",
  sync_failed: "sync",
  server: "connectivity",
  invalid_response: "connectivity",

  bad_request: "validation",
  cloud_invalid_data: "validation",
  cloud_invalid_commit: "validation",
  cloud_invalid_baseline: "validation",
  cloud_hash_mismatch: "validation",
  cloud_missing_parent: "validation",
  cloud_invalid_graph: "validation",
  cloud_listing_limit: "validation",
  cloud_response_limit: "validation",
  cloud_account: "validation",
  invalid_input: "validation",
  invalid_json: "validation",
  duplicate: "validation",
  capacity: "validation",
  invalid_tabs: "validation",
  tabs_scope_mismatch: "validation",
  integrity_invalid_scope: "validation",
  integrity_invalid_snapshot: "validation",
  integrity_limit: "validation",
  response_too_large: "validation",
  confirmation_required: "validation",
  integrity_approval_required: "validation",
  invalid_history_choice: "validation",
  invalid_preferences: "validation",
  invalid_filter: "validation",
  invalid_page: "validation",
  configuration_pending: "validation",
  integrity_missing_baseline: "validation",
};

const safeMessages: Record<string, string> = {
  sync_busy:
    "Uma sincronização já está em andamento. Aguarde sua conclusão antes de repetir a ação.",
  cloud_network:
    "Não foi possível acessar o Drive por falha de rede. Confira a conexão; nenhuma configuração local deve ser descartada.",
  cloud_server:
    "O serviço Drive está temporariamente indisponível. Aguarde e tente sincronizar novamente.",
  cloud_rate_limited:
    "A sincronização atingiu a cota de consultas do Drive. Aguarde o prazo de nova tentativa e evite repetir a ação.",
  cloud_local_changed:
    "A configuração local mudou durante a sincronização. Atualize os dados e revise as versões antes de repetir.",
  cloud_unauthorized:
    "A sessão Google não autorizou a sincronização. Reautorize a conta em Configurações.",
  cloud_forbidden:
    "O Google negou acesso aos dados de sincronização. Confira as permissões concedidas à conta conectada.",
  cloud_opt_in_required:
    "A sincronização não foi autorizada neste dispositivo. Revise a opção em Configurações e só reconecte se desejar conceder esse acesso.",
  cloud_storage:
    "Não foi possível persistir a sincronização no dispositivo. Verifique o espaço em disco e as permissões antes de repetir.",
  cloud_stale_resolution:
    "A versão mudou após a escolha de resolução. Atualize os dados e revise o conflito novamente; não sobrescreva a configuração local às cegas.",
  cloud_invalid_data: "Os dados remotos de sincronização têm formato inválido e foram recusados.",
  cloud_invalid_commit: "O registro remoto de sincronização é inválido e foi recusado.",
  cloud_invalid_baseline:
    "A referência remota não passou pela validação e foi recusada; nenhuma referência é aprovada automaticamente.",
  cloud_hash_mismatch:
    "A integridade dos dados remotos diverge do hash informado; a importação foi recusada.",
  cloud_missing_parent:
    "Falta uma versão ancestral para comprovar o histórico remoto; a importação foi recusada.",
  cloud_invalid_graph: "O histórico remoto é inconsistente e não permite uma resolução segura.",
  cloud_listing_limit:
    "A listagem remota excedeu o limite seguro; não é possível afirmar que o histórico está completo.",
  cloud_response_limit: "A resposta remota excedeu o limite seguro de leitura e não foi importada.",
  cloud_account: "Os dados remotos não correspondem à conta conectada e foram recusados.",
  // Autenticação e sessão
  invalid_configuration:
    "A configuração do cliente OAuth Desktop foi recusada pelo Google ou está incompleta nesta instalação. Solicite um instalador atualizado ao responsável.",
  configuration_invalid:
    "A configuração do cliente OAuth Desktop foi recusada pelo Google ou está incompleta nesta instalação. Solicite um instalador atualizado ao responsável.",
  invalid_desktop_client_id:
    "O Client ID informado não corresponde a um cliente OAuth Desktop válido. Revise a configuração e tente novamente.",
  unconfigured:
    "Esta instalação ainda não possui configuração OAuth Desktop. Solicite o pacote configurado ao responsável antes de conectar.",
  oauth_not_configured:
    "Esta instalação ainda não possui configuração OAuth Desktop. Solicite o pacote configurado ao responsável antes de conectar.",
  not_configured:
    "Esta instalação ainda não possui configuração OAuth Desktop. Solicite o pacote configurado ao responsável antes de conectar.",
  auth_required:
    "Sua conta Google não está conectada. Acesse Configurações e clique em Entrar com Google para continuar.",
  unauthorized:
    "Sua autorização Google não está mais ativa para esta leitura. Acesse Configurações e clique em Reautorizar permissões.",
  reconnect_required:
    "Sua sessão Google precisa ser renovada. Acesse Configurações e clique em Reautorizar permissões no navegador.",
  token_expired:
    "A sessão de acesso Google expirou. Acesse Configurações e clique em Reautorizar permissões no navegador.",
  invalid_grant:
    "A autorização da conta expirou ou foi revogada no Google. Acesse Configurações e conecte a conta novamente.",
  auth_busy:
    "Já existe uma autenticação Google em andamento. Conclua ou feche a autorização aberta no navegador antes de tentar novamente.",
  cancelled:
    "O login com Google foi cancelado antes da conclusão. Clique em Entrar com Google quando desejar tentar novamente.",
  consent_denied:
    "O consentimento foi negado no navegador ou a conta não está liberada na tela OAuth do Google Cloud. Conclua a autorização com uma conta permitida.",
  oauth_denied:
    "O consentimento foi negado no navegador ou a conta não está liberada na tela OAuth do Google Cloud. Conclua a autorização com uma conta permitida.",
  access_denied:
    "O consentimento foi negado no navegador ou a conta não está liberada na tela OAuth do Google Cloud. Conclua a autorização com uma conta permitida.",
  callback_invalid:
    "A resposta de login recebida do navegador não pôde ser validada com segurança. Feche abas antigas de login e tente novamente.",
  authorization_failed:
    "O Google recusou a autorização desta conta. Verifique se a conta está liberada no projeto Google Cloud (organização Workspace ou lista de usuários de teste) e tente novamente.",
  revocation_confirmation_required:
    "Confirme explicitamente a revogação antes de encerrar o acesso concedido no Google.",
  identity_mismatch:
    "A conta autenticada no navegador diverge da sessão atual. Desconecte a conta em Configurações e entre novamente.",
  operation_superseded:
    "A sessão da conta mudou enquanto a operação estava em andamento. Confira os dados atualizados e tente novamente.",
  stale_session:
    "A conta mudou durante a consulta e o resultado anterior foi descartado por segurança. Confira a conta ativa e tente novamente.",
  vault_corrupt:
    "As credenciais protegidas no Windows não puderam ser recuperadas. Desconecte a conta em Configurações e entre novamente.",
  credential_store_unavailable:
    "O Gerenciador de Credenciais do Windows está indisponível e a sessão não pôde ser salva. Verifique as permissões do usuário no Windows e tente novamente.",
  vault_unavailable:
    "O Gerenciador de Credenciais do Windows está indisponível e a sessão não pôde ser salva. Verifique as permissões do usuário no Windows e tente novamente.",
  browser_unavailable:
    "Não foi possível abrir o navegador padrão do Windows. Defina um navegador padrão no sistema e tente novamente.",
  browser_open_failed:
    "Não foi possível abrir o navegador do sistema para concluir a ação. Verifique o navegador padrão do Windows e tente novamente.",

  // Permissões
  insufficient_scope:
    "Falta permissão de leitura na conta Google para concluir esta consulta. Acesse Configurações, clique em Reautorizar permissões e aprove os escopos solicitados.",
  forbidden:
    "Acesso negado pelo Google. Verifique se a planilha ou o projeto está compartilhado com sua conta e se as permissões de leitura foram concedidas em Configurações.",

  // Conectividade e disponibilidade externa
  network: "Sem conexão com os serviços do Google. Verifique sua internet e tente novamente.",
  timeout:
    "A comunicação com o Google excedeu o tempo limite. Verifique sua conexão e tente novamente em instantes.",
  rate_limited:
    "O limite temporário de consultas da API Google foi atingido. Aguarde alguns instantes e tente novamente.",
  server:
    "Os serviços do Google estão temporariamente indisponíveis. Aguarde alguns instantes e tente novamente.",
  invalid_response:
    "O Google retornou uma resposta incompleta ou inesperada. Aguarde alguns instantes e tente atualizar os dados.",

  // Validação e limites de leitura
  bad_request:
    "O Google recusou os parâmetros informados. Confira se o link da planilha, o ID do projeto e os intervalos A1 estão corretos.",
  invalid_input:
    "Os dados informados são inválidos. Confira o link completo do Google Sheets ou os identificadores preenchidos e tente novamente.",
  response_too_large:
    "Os dados consultados excederam o limite de leitura permitido. Reduza o intervalo de linhas ou colunas monitoradas e tente novamente.",

  // Operações internas e recursos
  not_found:
    "O arquivo ou projeto não foi encontrado ou não está compartilhado com esta conta Google. Confira o link e as permissões de acesso.",
  local_storage:
    "Não foi possível salvar ou consultar os dados locais neste dispositivo. Verifique o espaço em disco e tente novamente.",
  desktop_required:
    "Esta ação exige o aplicativo desktop no Windows. Abra o Sheets Monitor instalado para acessar sua conta e seus dados.",
  operation_in_progress:
    "Outra operação já está em andamento. Aguarde sua conclusão antes de tentar novamente.",
};

export function extractErrorCode(error: unknown): string | undefined {
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const record = error as Record<string, unknown>;
    const raw = record.code ?? record.kind;
    if (typeof raw === "string" && raw.trim() !== "") return raw;
  }
  return undefined;
}

export function extractErrorDiagnostic(error: unknown, context?: string): ErrorDiagnostic {
  const code = (extractErrorCode(error) ?? "unknown_error").split("|", 1)[0];
  const retryable =
    Boolean(
      error && typeof error === "object" && (error as Record<string, unknown>).retryable === true,
    ) ||
    [
      "network",
      "timeout",
      "rate_limited",
      "server",
      "auth_busy",
      "cloud_network",
      "cloud_rate_limited",
      "cloud_server",
      "sync_busy",
    ].includes(code);
  const category = errorCategories[code] ?? "internal";
  return context ? { code, category, retryable, context } : { code, category, retryable };
}

export function logErrorDiagnostic(error: unknown, context?: string): ErrorDiagnostic {
  const diagnostic = extractErrorDiagnostic(error, context);
  const mode = (import.meta as { env?: { MODE?: string } }).env?.MODE;
  if (mode !== "test") {
    console.warn("[SheetsMonitor]", diagnostic);
  }
  return diagnostic;
}

export function errorGuidance(error: unknown) {
  const { category, code } = extractErrorDiagnostic(error);
  if (category === "validation" && code.startsWith("cloud_")) {
    return {
      category,
      label: "Dados de sincronização recusados",
      impact: "A importação foi interrompida; a configuração local é preservada.",
      solution:
        "Confira a conta e a integridade das versões remotas. Resolva o histórico incompleto ou inválido antes de repetir; não substitua a referência local automaticamente.",
    };
  }
  if (code === "cloud_opt_in_required") {
    return {
      category,
      label: "Sincronização opcional desativada",
      impact: "A sincronização não foi iniciada; o monitoramento local permanece separado.",
      solution:
        "Revise a opção em Configurações. Só habilite e reconecte se desejar conceder o acesso adicional.",
    };
  }
  switch (category) {
    case "auth":
      return {
        category,
        label: "Autenticação",
        impact: "A leitura pode estar suspensa; o último resultado não confirma o estado atual.",
        solution: "Confira a conta em Configurações e reautorize o acesso Google.",
      };
    case "permission":
      return {
        category,
        label: "Permissão",
        impact: "O recurso não pôde ser lido; isso não comprova uma violação estrutural.",
        solution: "Confira o compartilhamento com a conta conectada e as permissões de leitura.",
      };
    case "connectivity":
      return {
        category,
        label: "Rede e disponibilidade",
        impact:
          "A verificação atual não foi concluída; os dados anteriores podem estar desatualizados.",
        solution:
          "Confira a conexão. Aguarde a próxima tentativa do serviço ou tente novamente após restabelecer o acesso.",
      };
    case "quota":
      return {
        category,
        label: "Limite de consultas Google",
        impact: "As consultas estão adiadas; o estado atual ainda não pôde ser confirmado.",
        solution:
          "Aguarde o prazo de nova tentativa. Evite verificações manuais repetidas e aumente o intervalo se necessário.",
      };
    case "sync":
      return {
        category,
        label: "Sincronização",
        impact:
          "A operação não confirmou a atualização; os dados locais podem estar desatualizados.",
        solution:
          "Atualize os dados e revise a versão atual antes de repetir a ação. Não aprove uma referência automaticamente.",
      };
    default:
      return null;
  }
}

export function integrationErrorMessage(error: unknown, context?: string): string {
  const code = extractErrorCode(error);
  if (
    (code === "rate_limited" || code === "cloud_rate_limited") &&
    error &&
    typeof error === "object"
  ) {
    const message = (error as Record<string, unknown>).message;
    if (
      typeof message === "string" &&
      /^(Sheets|Drive|Apps Script|Admin SDK Reports) API \//.test(message)
    ) {
      const guidance = errorGuidance(error)!;
      return `${safeMessages.rate_limited} Impacto: ${guidance.impact} Solução: ${guidance.solution} Detalhes: ${message.slice(0, 512)}`;
    }
  }
  if (code && Object.hasOwn(safeMessages, code)) {
    const guidance = errorGuidance(error);
    return guidance
      ? `${safeMessages[code]} Impacto: ${guidance.impact} Solução: ${guidance.solution}`
      : safeMessages[code];
  }
  const guidance = errorGuidance(error);
  if (guidance)
    return `${guidance.label}. Impacto: ${guidance.impact} Solução: ${guidance.solution}`;
  return context
    ? `Não foi possível ${context}. Verifique sua conexão com a internet e as permissões da conta Google e tente novamente.`
    : "Não foi possível concluir a operação. Verifique sua conexão com a internet e as permissões da conta Google e tente novamente.";
}
