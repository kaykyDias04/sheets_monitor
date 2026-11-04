import { invoke } from "@tauri-apps/api/core";
import { desktop } from "./platform";
import { extractErrorCode, integrationErrorMessage, logErrorDiagnostic } from "./google";
import type {
  EngineEnvelope,
  ScopeProposal,
  MonitorScope,
  CapturePreview,
  BaselineSummary,
  ValidationResult,
  ScopeQuery,
  IntegrityEvent,
} from "@/types/integrity";

function command<T>(name: string, args: Record<string, unknown>) {
  if (!desktop) return Promise.reject({ code: "desktop_required" });
  return invoke<EngineEnvelope<T>>(name, args);
}

export const proposeScope = (url: string) =>
  command<ScopeProposal>("integrity_propose_scope", { url });
export const captureSnapshot = (url: string, scope: MonitorScope) =>
  command<CapturePreview>("integrity_capture_snapshot", { input: { url, scope } });
export const approveBaseline = (input: {
  expected_generation: number;
  snapshot_id: string;
  snapshot_fingerprint: string;
  expected_version: number;
  confirmed: boolean;
}) => command<BaselineSummary>("integrity_create_baseline", { input });
export const validateIntegrity = (snapshot_id: string) =>
  command<ValidationResult>("integrity_validate_integrity", { snapshotId: snapshot_id });
export const listBaselines = (input: ScopeQuery, offset = 0) =>
  command<BaselineSummary[]>("integrity_list_baselines", { input, offset });
export const listEvents = (input: ScopeQuery) =>
  command<IntegrityEvent[]>("integrity_list_integrity_events", { input });
export const pruneHistory = (input: ScopeQuery) =>
  command<number>("integrity_prune_history", { input });

const messages: Record<string, string> = {
  integrity_invalid_scope:
    "As regras de proteção possuem campos inválidos ou incompletos. Revise cabeçalhos, linhas, colunas e o limite de 20.000 células e tente novamente.",
  integrity_invalid_snapshot:
    "A captura atual não cobre todas as regras configuradas. Confira se as abas e colunas existem na planilha e capture a estrutura novamente.",
  integrity_missing_baseline:
    "Esta planilha ainda não possui uma referência aprovada para as regras atuais. Capture a estrutura na aba Referência e aprove-a primeiro.",
  integrity_baseline_conflict:
    "A estrutura ou a configuração da planilha mudou desde a última captura. Capture a estrutura novamente na aba Referência antes de aprovar.",
  integrity_approval_required:
    "Marque a confirmação de revisão do escopo antes de aprovar a nova referência estrutural.",
  integrity_limit:
    "O escopo configurado excedeu o limite permitido de captura ou análise (até 20.000 células). Reduza o intervalo de linhas ou colunas nas regras de proteção.",
  integrity_forbidden:
    "Este recurso pertence a outra conta Google. Conecte a conta correspondente em Configurações para acessá-lo.",
  integrity_not_found:
    "A captura ou referência solicitada não foi encontrada nesta conta. Realize uma nova captura na aba Referência.",
  integrity_busy:
    "Já existem três verificações simultâneas em execução. Aguarde alguns segundos para que uma termine e tente novamente.",
  integrity_storage:
    "Não foi possível gravar ou ler os dados no banco local deste dispositivo. Verifique o espaço em disco e tente novamente.",
  invalid_json:
    "Os dados de configuração do escopo não estão em um formato válido. Revise os campos preenchidos e tente novamente.",
  stale_session:
    "A conta mudou durante a operação e o resultado anterior foi descartado por segurança. Confira a conta conectada e tente novamente.",
};

export function engineError(error: unknown, context?: string): string {
  logErrorDiagnostic(error, context);
  const code = extractErrorCode(error);
  return code && Object.hasOwn(messages, code)
    ? messages[code]
    : integrationErrorMessage(error, context);
}
