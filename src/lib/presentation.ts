import type { IntegrityStatus } from "@/types/foundation";
export const stateLabels: Record<IntegrityStatus, string> = {
  online: "Online",
  offline: "Offline",
  checking: "Verificando",
  indeterminate: "Indeterminado",
};
export const stateHelp: Record<IntegrityStatus, string> = {
  online: "Planilha verificada e estruturalmente íntegra no cenário demonstrativo.",
  offline:
    "Falha confirmada de estrutura, fórmula protegida ou dependência crítica no cenário demonstrativo.",
  checking: "Avaliação em andamento no cenário demonstrativo.",
  indeterminate:
    "Integridade não confirmada por indisponibilidade, permissão ou falha técnica no cenário demonstrativo.",
};
export const categoryNames: Record<string, string> = {
  content: "Conteúdo",
  structure: "Estrutura",
  operational: "Operacional",
  script: "Apps Script",
};
export function formatDate(value: string | null) {
  return value
    ? new Intl.DateTimeFormat("pt-BR", {
        day: "2-digit",
        month: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      }).format(new Date(value))
    : "Sem avaliação simulada";
}
