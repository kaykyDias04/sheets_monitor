import { useEffect, useRef, useState } from "react";
import {
  Pause,
  Play,
  ExternalLink,
  RefreshCw,
  Trash2,
  SearchCheck,
  History,
  Save,
  Sparkles,
  LoaderCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import type { SpreadsheetMetadata } from "@/types/google";
import type { MonitorScope, BaselineSummary } from "@/types/integrity";
import type { BaselineComparison, Diagnostics, OperationalPreview } from "@/types/management";
import { ScopeEditor, scopeForTabs, scopeFormProblems } from "./ScopeEditor";
import { BaselineFlow } from "./BaselineFlow";
import { ScriptCoverage } from "./ScriptCoverage";
import {
  accessErrorText,
  connectivityText,
  dateLabel,
  diagnosticText,
  StatusBadge,
} from "./OperationalShared";

const MAX_MONITORED_TABS = 1024;

export function tabSelectionProblems(
  selectedIds: number[],
  tabs: SpreadsheetMetadata["sheets"],
): string[] {
  const problems: string[] = [];
  if (selectedIds.length === 0) {
    problems.push(
      "Selecione ao menos uma aba de grade antes de salvar e avançar para as regras de proteção.",
    );
  }
  if (selectedIds.length > MAX_MONITORED_TABS) {
    problems.push(`Selecione no máximo ${MAX_MONITORED_TABS.toLocaleString("pt-BR")} abas.`);
  }
  const invalidSelected = selectedIds.filter((id) => {
    const found = tabs.find((item) => item.sheet_id === id);
    return !found || (!!found.sheet_type && found.sheet_type !== "GRID");
  });
  if (invalidSelected.length > 0) {
    problems.push("Desmarque abas removidas ou de tipo não suportado antes de salvar a seleção.");
  }
  return problems;
}

interface Props {
  data: Diagnostics;
  preview: OperationalPreview | null;
  versions: BaselineSummary[];
  comparison: BaselineComparison | null;
  versionOffset: number;
  disabled: boolean;
  busy?: string | null;
  error?: string;
  notice?: string;
  defaultHistoryDays?: number;
  initialTab?: string;
  onTabChange?: (tab: string) => void;
  onClearFeedback?: () => void;
  onPreview: () => void;
  onInvalidate: () => void;
  onApprove: (preview: OperationalPreview) => void;
  onVersionPage: (offset: number) => void;
  onCompare: (before: string, after: string) => void;
  onComparisonClear: () => void;
  onSaveTabs: (ids: number[]) => Promise<boolean>;
  onSaveRules: (scope: MonitorScope) => Promise<boolean>;
  onSuggestRules: () => Promise<MonitorScope | null>;
  onPause: () => void;
  onCheck: () => void;
  onRefresh: () => void;
  onOpen: () => void;
  onHistory: () => void;
  onRemove: (history: "preserve" | "delete_activity") => void;
  onAssociateScript: (scriptId: string) => void;
}

export function SpreadsheetDetails({
  data,
  preview,
  versions,
  comparison,
  versionOffset,
  disabled,
  busy = null,
  error = "",
  notice = "",
  defaultHistoryDays = 90,
  initialTab = "tabs",
  onTabChange,
  onClearFeedback,
  onPreview,
  onInvalidate,
  onApprove,
  onVersionPage,
  onCompare,
  onComparisonClear,
  onSaveTabs,
  onSaveRules,
  onSuggestRules,
  onPause,
  onCheck,
  onRefresh,
  onOpen,
  onHistory,
  onRemove,
  onAssociateScript,
}: Props) {
  const row = data.spreadsheet;
  const [tab, setTabState] = useState<string>(initialTab);
  const setTab = (next: string) => {
    setTabState(next);
    onTabChange?.(next);
  };
  const [selected, setSelected] = useState(row.selected_sheet_ids);
  const [scope, setScope] = useState(() =>
    scopeForTabs(row.scope, row.selected_sheet_ids, defaultHistoryDays),
  );
  const [tabProblems, setTabProblems] = useState<string[]>([]);
  const [ruleProblems, setRuleProblems] = useState<string[]>([]);
  const tabProblemsRef = useRef<HTMLDivElement>(null);
  const problemsRef = useRef<HTMLDivElement>(null);
  const inlineFeedbackRef = useRef<HTMLDivElement>(null);
  const [removeOpen, setRemoveOpen] = useState(false);
  const [removeConfirmed, setRemoveConfirmed] = useState(false);
  const [historyChoice, setHistoryChoice] = useState<"preserve" | "delete_activity">("preserve");
  const [scriptId, setScriptId] = useState("");

  useEffect(() => {
    if (tabProblems.length === 0) return;
    tabProblemsRef.current?.scrollIntoView?.({ block: "nearest" });
    tabProblemsRef.current?.focus();
  }, [tabProblems]);

  useEffect(() => {
    if (ruleProblems.length === 0) return;
    problemsRef.current?.scrollIntoView?.({ block: "nearest" });
    problemsRef.current?.focus();
  }, [ruleProblems]);

  useEffect(() => {
    if (!error && !notice) return;
    inlineFeedbackRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [error, notice]);

  const tabsDirty =
    JSON.stringify([...selected].sort((a, b) => a - b)) !==
    JSON.stringify([...row.selected_sheet_ids].sort((a, b) => a - b));
  const rulesDirty = JSON.stringify(scope) !== JSON.stringify(row.scope);

  const tabNames = [
    ["tabs", "Abas"],
    ["rules", "Regras de proteção"],
    ["baselines", "Referência"],
    ["diagnostics", "Diagnósticos"],
    ["scripts", "Apps Script"],
  ];

  function select(ids: number[]) {
    setSelected(ids);
    setScope(scopeForTabs(scope, ids, defaultHistoryDays));
    setTabProblems([]);
    setRuleProblems([]);
    onInvalidate();
  }

  async function handleSaveTabs() {
    const found = tabSelectionProblems(selected, row.tabs);
    setTabProblems(found);
    if (found.length > 0) {
      onClearFeedback?.();
      return;
    }
    const ok = await onSaveTabs(selected);
    if (ok) {
      setScope((currentScope) => scopeForTabs(currentScope, selected, defaultHistoryDays));
      setTabProblems([]);
      setRuleProblems([]);
      setTab("rules");
    }
  }

  async function handleSaveRules() {
    const found = scopeFormProblems(scope, row.tabs);
    setRuleProblems(found);
    if (found.length > 0) {
      onClearFeedback?.();
      return;
    }
    const ok = await onSaveRules(scope);
    if (ok) {
      setRuleProblems([]);
      setTab("baselines");
    }
  }

  const inlineFeedback =
    busy || error || notice ? (
      <div className="op-inline-feedback" ref={inlineFeedbackRef}>
        {error && (
          <div className="feedback error" role="alert">
            {error}
          </div>
        )}
        {notice && (
          <div className="feedback success" role="status">
            {notice}
          </div>
        )}
        {busy && (
          <div className="op-progress" role="status">
            <LoaderCircle className="animate-spin motion-reduce:animate-none" size={16} />
            {busy}…
          </div>
        )}
      </div>
    ) : null;

  return (
    <div className="op-stack">
      <Card className="op-detail-heading">
        <CardContent>
          <div className="op-section-heading">
            <div>
              <div className="op-detail-status">
                <StatusBadge status={row.status} />
                <span>
                  {row.baseline_version
                    ? `Referência v${row.baseline_version}`
                    : "Sem referência aprovada"}
                </span>
              </div>
              <h2>{row.name}</h2>
              <p>
                {row.account_email ?? "Conta Google conectada"} · Última verificação:{" "}
                {dateLabel(row.last_checked_at)}
              </p>
            </div>
            <Button
              onClick={onCheck}
              disabled={
                disabled ||
                row.paused ||
                !row.baseline_version ||
                row.status === "configuration_pending" ||
                tabsDirty ||
                rulesDirty
              }
            >
              <SearchCheck />
              Verificar agora
            </Button>
          </div>
          <div className="op-actions">
            <Button variant="outline" size="sm" disabled={disabled} onClick={onPause}>
              {row.paused ? <Play /> : <Pause />}
              {row.paused ? "Retomar" : "Pausar"}
            </Button>
            <Button variant="outline" size="sm" disabled={disabled} onClick={onRefresh}>
              <RefreshCw />
              Atualizar abas e nome
            </Button>
            <Button variant="outline" size="sm" disabled={disabled} onClick={onOpen}>
              <ExternalLink />
              Abrir no Google Sheets
            </Button>
            <Button variant="outline" size="sm" disabled={disabled} onClick={onHistory}>
              <History />
              Ver atividades
            </Button>
            <Button
              variant="ghost"
              size="sm"
              disabled={disabled}
              onClick={() => {
                setRemoveConfirmed(false);
                setRemoveOpen(true);
              }}
            >
              <Trash2 />
              Remover cadastro
            </Button>
          </div>
          {row.authorization === "error" && (
            <p className="op-notice op-warning">
              Não foi possível acessar o arquivo na última consulta. Revise as permissões Google e o
              compartilhamento.
              <br />
              <span className="op-hint">{accessErrorText(row.access_error)}</span>
            </p>
          )}
          {(tabsDirty || rulesDirty) && (
            <p className="op-hint">
              Há ajustes locais não salvos. Atualizar os dados descarta esses ajustes.
            </p>
          )}
        </CardContent>
      </Card>

      <nav className="op-detail-nav" aria-label="Seções da planilha">
        {tabNames.map(([key, label]) => (
          <button
            key={key}
            aria-current={tab === key ? "page" : undefined}
            onClick={() => setTab(key)}
          >
            {label}
          </button>
        ))}
      </nav>

      {tab === "tabs" && (
        <Card>
          <CardHeader>
            <CardTitle>Abas monitoradas</CardTitle>
            <CardDescription>
              Marque apenas as abas que precisam de proteção e clique em &ldquo;Salvar seleção de
              abas&rdquo; para avançar às regras de proteção.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="op-section-heading">
              <label className="op-check">
                <input
                  type="checkbox"
                  disabled={disabled}
                  checked={
                    row.tabs.length > 0 &&
                    row.tabs
                      .filter((t) => !t.sheet_type || t.sheet_type === "GRID")
                      .every((t) => selected.includes(t.sheet_id))
                  }
                  onChange={(e) =>
                    select(
                      e.target.checked
                        ? row.tabs
                            .filter((t) => !t.sheet_type || t.sheet_type === "GRID")
                            .map((t) => t.sheet_id)
                        : [],
                    )
                  }
                />
                Selecionar todas as abas de grade
              </label>
              <Button
                disabled={disabled || (!tabsDirty && row.selected_sheet_ids.length > 0)}
                onClick={() => void handleSaveTabs()}
              >
                <Save />
                Salvar seleção de abas
              </Button>
            </div>
            {tabProblems.length > 0 && (
              <div className="feedback error" role="alert" ref={tabProblemsRef} tabIndex={-1}>
                <div>
                  <strong>A seleção de abas ainda não foi salva:</strong>
                  <ul>
                    {tabProblems.map((problem) => (
                      <li key={problem}>{problem}</li>
                    ))}
                  </ul>
                </div>
              </div>
            )}
            {inlineFeedback}
            <div className="op-tab-grid">
              {row.tabs.map((item) => (
                <label className="op-tab-option" key={item.sheet_id}>
                  <input
                    type="checkbox"
                    checked={selected.includes(item.sheet_id)}
                    aria-label={`Monitorar aba ${item.title} (ID ${item.sheet_id})`}
                    disabled={disabled || (!!item.sheet_type && item.sheet_type !== "GRID")}
                    onChange={(e) =>
                      select(
                        e.target.checked
                          ? [...selected, item.sheet_id]
                          : selected.filter((id) => id !== item.sheet_id),
                      )
                    }
                  />
                  <span>
                    <strong>{item.title}</strong>
                    <small>
                      ID {item.sheet_id} · {item.row_count ?? "?"} linhas ·{" "}
                      {item.column_count ?? "?"} colunas{item.hidden ? " · Oculta no Google" : ""}
                      {item.sheet_type && item.sheet_type !== "GRID"
                        ? " · Tipo não suportado para captura"
                        : ""}
                    </small>
                  </span>
                </label>
              ))}
            </div>
            {selected
              .filter((id) => !row.tabs.some((item) => item.sheet_id === id))
              .map((id) => (
                <label className="op-tab-option op-warning" key={id}>
                  <input
                    type="checkbox"
                    checked
                    disabled={disabled}
                    onChange={() => select(selected.filter((v) => v !== id))}
                  />
                  <span>
                    Aba removida ou não disponível · ID {id}
                    <small>Desmarque para retirar do escopo ou restaure o acesso.</small>
                  </span>
                </label>
              ))}
            <p className="op-notice">
              Dependências podem atravessar abas. Excluir uma aba do monitoramento pode impedir a
              verificação completa de fórmulas ou scripts que dependem dela. Salvar a seleção não
              aprova a integridade.
            </p>
          </CardContent>
        </Card>
      )}

      {tab === "rules" && (
        <Card>
          <CardHeader>
            <CardTitle>Regras de proteção</CardTitle>
            <CardDescription>
              Escolha de forma simples o que não pode quebrar nas suas abas: use a leitura
              automática de cabeçalhos ou preencha apenas as colunas principais.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {inlineFeedback}
            <div className="op-suggest-banner">
              <div>
                <strong>Jeito mais fácil: buscar os cabeçalhos reais da planilha</strong>
                <p>
                  Lê a primeira linha das abas selecionadas no Google Sheets (até 8 colunas e 50
                  linhas iniciais por aba, para até 30 abas) e monta o rascunho automaticamente.
                  Depois basta conferir e clicar em &ldquo;Salvar regras de proteção&rdquo;.
                </p>
              </div>
              <Button
                type="button"
                variant="outline"
                disabled={disabled || tabsDirty || selected.length === 0 || selected.length > 30}
                onClick={() => {
                  void onSuggestRules().then((proposal) => {
                    if (proposal) {
                      setScope(proposal);
                      setRuleProblems([]);
                      onInvalidate();
                    }
                  });
                }}
              >
                <Sparkles size={16} />
                Sugerir regras para abas selecionadas
              </Button>
            </div>

            <form
              noValidate
              onSubmit={(e) => {
                e.preventDefault();
                void handleSaveRules();
              }}
            >
              <ScopeEditor
                value={scope}
                tabs={row.tabs}
                disabled={disabled}
                onChange={(next) => {
                  setScope(next);
                  setRuleProblems([]);
                  onInvalidate();
                }}
              />
              <div className="op-actions op-save-bar">
                <Button
                  type="submit"
                  disabled={disabled || tabsDirty || scope.sheets.length === 0 || !rulesDirty}
                >
                  <Save />
                  Salvar regras de proteção
                </Button>
              </div>
              {ruleProblems.length > 0 && (
                <div className="feedback error" role="alert" ref={problemsRef} tabIndex={-1}>
                  <div>
                    <strong>As regras ainda não foram salvas. Ajuste os itens abaixo:</strong>
                    <ul>
                      {ruleProblems.slice(0, 10).map((problem) => (
                        <li key={problem}>{problem}</li>
                      ))}
                    </ul>
                    {ruleProblems.length > 10 && (
                      <p>Há mais {ruleProblems.length - 10} pendências semelhantes.</p>
                    )}
                  </div>
                </div>
              )}
              {tabsDirty && (
                <p className="op-notice op-warning">
                  Você alterou as abas selecionadas, mas ainda não salvou. Volte em
                  &ldquo;Abas&rdquo; e clique em &ldquo;Salvar seleção de abas&rdquo; antes de
                  salvar as regras.
                </p>
              )}
              {!tabsDirty && scope.sheets.length > 0 && !rulesDirty && row.scope !== null && (
                <p className="op-hint">
                  As regras exibidas já estão salvas. Edite qualquer campo para habilitar um novo
                  salvamento ou acesse &ldquo;Referência&rdquo; para capturar e aprovar.
                </p>
              )}
            </form>
          </CardContent>
        </Card>
      )}

      {tab === "baselines" && (
        <BaselineFlow
          row={row}
          preview={preview}
          versions={versions}
          comparison={comparison}
          offset={versionOffset}
          disabled={disabled}
          dirty={tabsDirty || rulesDirty}
          feedback={inlineFeedback}
          onCapture={onPreview}
          onApprove={onApprove}
          onEditRules={() => setTab("rules")}
          onPage={onVersionPage}
          onCompare={onCompare}
          onComparisonClear={onComparisonClear}
        />
      )}

      {tab === "diagnostics" && (
        <Card>
          <CardHeader>
            <CardTitle>Diagnóstico da integridade</CardTitle>
            <CardDescription>
              Resultados do motor para o escopo e referência atuais.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {inlineFeedback}
            {data.report ? (
              <>
                <div className="op-section-heading">
                  <StatusBadge status={data.report.status} />
                  <span className="op-hint">Detecção em {dateLabel(data.report.detected_at)}</span>
                </div>
                <p className="op-notice">
                  Conectividade: {connectivityText(data.report.connectivity)}. Cobertura:{" "}
                  {data.report.coverage.complete ? "completa no escopo" : "parcial"}. A integridade
                  depende das regras configuradas e das fontes disponíveis.
                </p>
                {data.report.findings.length === 0 ? (
                  <p className="op-empty">
                    Nenhuma violação ou incerteza de regra encontrada nesta verificação.
                  </p>
                ) : (
                  <ul className="op-findings">
                    {data.report.findings.map((finding) => (
                      <li key={finding.key}>
                        <strong>
                          {finding.outcome === "inconclusive"
                            ? "Não foi possível determinar"
                            : diagnosticText(finding.code)}
                        </strong>
                        <p>{diagnosticText(finding.code)}</p>
                        <dl>
                          <div>
                            <dt>Regra</dt>
                            <dd>{finding.rule_id}</dd>
                          </div>
                          <div>
                            <dt>Local</dt>
                            <dd>
                              {finding.location ?? `Aba ${finding.sheet_id ?? "não determinada"}`}
                            </dd>
                          </div>
                          <div>
                            <dt>Esperado</dt>
                            <dd>{finding.expected ?? "Não disponível"}</dd>
                          </div>
                          <div>
                            <dt>Encontrado</dt>
                            <dd>{finding.found ?? "Não disponível"}</dd>
                          </div>
                        </dl>
                        <p className="op-hint">
                          {finding.outcome === "inconclusive"
                            ? "Revise a cobertura e as permissões antes de concluir que há uma falha."
                            : "Confira a regra e a estrutura. Se a alteração for legítima, revise e aprove uma nova referência."}
                        </p>
                      </li>
                    ))}
                  </ul>
                )}
                {data.report.coverage.issues.length > 0 && (
                  <details className="op-advanced">
                    <summary>Limitações de cobertura</summary>
                    <ul>
                      {data.report.coverage.issues.map((issue, i) => (
                        <li key={i}>
                          {diagnosticText(issue.code)} (
                          {issue.critical ? "regra crítica" : "regra opcional"})
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
              </>
            ) : (
              <p className="op-empty">
                Ainda não há verificação para a configuração atual. Selecione abas, salve regras
                válidas, aprove a referência e escolha Verificar agora.
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {tab === "scripts" && (
        <div className="op-stack">
          <Card>
            <CardHeader>
              <CardTitle>Associar projeto autorizado</CardTitle>
              <CardDescription>
                Se a descoberta automática não for viável, informe o ID do projeto Apps Script. O
                aplicativo apenas consulta o código.
              </CardDescription>
            </CardHeader>
            <CardContent>
              {inlineFeedback}
              <form
                className="op-script-association"
                onSubmit={(e) => {
                  e.preventDefault();
                  onAssociateScript(scriptId.trim());
                }}
              >
                <label className="op-field">
                  ID do projeto Apps Script
                  <Input
                    value={scriptId}
                    required
                    maxLength={256}
                    onChange={(e) => setScriptId(e.target.value)}
                    autoComplete="off"
                  />
                </label>
                <Button type="submit" disabled={disabled || !scriptId.trim()}>
                  Consultar e associar projeto
                </Button>
              </form>
              <p className="op-hint">
                Após associar, inclua o projeto nas regras para proteger suas dependências. Arquivos
                e cobertura aparecem após captura e análise do escopo.
              </p>
            </CardContent>
          </Card>
          <ScriptCoverage
            scripts={data.scripts}
            previewScripts={preview?.preview.snapshot.scripts}
          />
        </div>
      )}

      <Dialog open={removeOpen} onOpenChange={setRemoveOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Remover cadastro de {row.name}?</DialogTitle>
            <DialogDescription>
              Apenas o cadastro local será removido. O arquivo no Google Drive permanece intacto.
            </DialogDescription>
          </DialogHeader>
          <div className="op-stack">
            <label className="op-field">
              Histórico de atividades
              <select
                value={historyChoice}
                onChange={(e) => setHistoryChoice(e.target.value as "preserve" | "delete_activity")}
              >
                <option value="preserve">Preservar atividades no histórico</option>
                <option value="delete_activity">Excluir atividades e relatórios locais</option>
              </select>
            </label>
            <p className="op-hint">
              Referências aprovadas e seus snapshots permanecem preservados para rastreabilidade. A
              exclusão de atividades é local e não pode ser desfeita.
            </p>
            <label className="op-check">
              <input
                type="checkbox"
                checked={removeConfirmed}
                onChange={(e) => setRemoveConfirmed(e.target.checked)}
              />
              Confirmo a remoção deste cadastro.
            </label>
            <div className="op-actions">
              <Button variant="outline" onClick={() => setRemoveOpen(false)}>
                Cancelar
              </Button>
              <Button
                disabled={disabled || !removeConfirmed}
                onClick={() => onRemove(historyChoice)}
              >
                Confirmar remoção
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
