import { invoke } from "@tauri-apps/api/core";
import { desktop } from "./platform";
import { integrationErrorMessage } from "./google";
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
    "Revise o escopo: regiões, cabeçalhos, chaves e limites devem ser explícitos e válidos.",
  integrity_invalid_snapshot:
    "A captura não é suficiente para esta operação. Confira as regiões, fórmulas e a cobertura.",
  integrity_missing_baseline: "Ainda não há baseline aprovado para esta conta, planilha e escopo.",
  integrity_baseline_conflict:
    "A referência ou observação mudou. Capture novamente antes de continuar.",
  integrity_approval_required: "Aprovar uma referência exige confirmação explícita.",
  integrity_limit:
    "O escopo excedeu o limite de captura ou análise. Reduza regiões, arquivos ou tamanho.",
  integrity_forbidden: "Esta operação não pertence à conta conectada.",
  integrity_not_found: "O snapshot ou referência não está disponível nesta conta.",
  integrity_busy: "Há três capturas em andamento. Aguarde uma delas terminar.",
  integrity_storage: "Não foi possível concluir a transação local.",
  invalid_json: "O escopo precisa ser um JSON válido.",
  stale_session: "A conta mudou durante a operação. O resultado foi descartado.",
};
export function engineError(error: unknown): string {
  const code =
    error && typeof error === "object" ? (error as Record<string, unknown>).code : undefined;
  return typeof code === "string" && Object.hasOwn(messages, code)
    ? messages[code]
    : integrationErrorMessage(error);
}
