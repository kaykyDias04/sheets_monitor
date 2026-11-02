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

const safeMessages: Record<string, string> = {
  invalid_configuration: "Revise o Client ID Desktop e a configuração informada.",

  auth_busy: "Outra operação de autenticação está em andamento.",

  invalid_grant: "A autorização expirou ou foi revogada. Reconecte a conta.",

  vault_corrupt:
    "A credencial protegida não pôde ser recuperada. Desconecte e configure novamente.",

  operation_superseded: "A sessão mudou. O resultado anterior foi descartado.",

  insufficient_scope: "A permissão necessária não foi concedida. Autorize novamente.",

  browser_unavailable: "Não foi possível abrir o navegador do sistema.",

  local_storage: "Não foi possível salvar os metadados locais.",

  identity_mismatch: "A identidade retornada diverge da conta. Reconecte a conta.",

  desktop_required: "Abra o aplicativo Windows para configurar a conta e consultar as APIs Google.",

  unconfigured: "Configure um cliente OAuth Desktop antes de conectar.",

  oauth_not_configured: "Configure um cliente OAuth Desktop antes de conectar.",

  not_configured: "Configure um cliente OAuth Desktop antes de conectar.",

  auth_required: "Conecte sua conta Google para realizar esta consulta.",

  unauthorized: "A sessão não autoriza esta leitura. Reconecte a conta Google.",

  reconnect_required: "A sessão precisa de uma nova autorização no navegador.",

  consent_denied: "O consentimento foi negado. Nenhuma conexão foi concluída.",

  oauth_denied: "O consentimento foi negado. Nenhuma conexão foi concluída.",

  access_denied: "O consentimento foi negado. Nenhuma conexão foi concluída.",

  forbidden:
    "Acesso negado. Confirme os escopos, o compartilhamento e a habilitação da API no Google Cloud. Para Apps Script, verifique também a autorização do painel do serviço.",

  not_found: "O recurso não foi encontrado ou não está disponível para esta conta.",

  bad_request: "O Google recusou a consulta. Revise o link, os IDs e os intervalos A1.",

  invalid_input: "Revise o link, os IDs e os intervalos informados.",

  rate_limited: "A cota de consultas foi atingida. Aguarde e tente novamente.",

  network: "Não foi possível acessar o Google. Verifique a conexão com a internet.",

  timeout: "A operação excedeu o tempo limite. Tente novamente.",

  server: "O serviço Google está temporariamente indisponível. Tente novamente.",

  invalid_response: "O serviço retornou uma resposta que não pôde ser validada.",

  response_too_large: "A resposta excedeu o limite de leitura. Reduza os intervalos consultados.",

  credential_store_unavailable:
    "O armazenamento seguro do Windows está indisponível. A sessão não foi salva.",

  vault_unavailable: "O armazenamento seguro do Windows está indisponível. A sessão não foi salva.",

  browser_open_failed: "Não foi possível abrir o navegador do sistema. Tente novamente.",

  operation_in_progress: "Outra operação está em andamento. Aguarde sua conclusão.",

  stale_session:
    "A conta mudou durante a consulta. O resultado anterior foi descartado; tente novamente.",

  configuration_invalid: "Revise o Client ID do tipo Desktop e a configuração informada.",
};

export function integrationErrorMessage(error: unknown): string {
  let code: unknown;

  if (typeof error === "string") code = error;
  else if (error && typeof error === "object") {
    const record = error as Record<string, unknown>;

    code = record.code ?? record.kind;
  }

  return typeof code === "string" && Object.hasOwn(safeMessages, code)
    ? safeMessages[code]
    : "Não foi possível concluir a operação. Revise a configuração, as permissões e a conexão; tente novamente.";
}
