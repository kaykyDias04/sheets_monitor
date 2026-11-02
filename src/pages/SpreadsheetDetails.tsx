import { useState } from "react";
import {
  Pause,
  Play,
  ExternalLink,
  RefreshCw,
  Trash2,
  SearchCheck,
  History,
  Save,
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
import type { MonitorScope, BaselineSummary } from "@/types/integrity";
import type { BaselineComparison, Diagnostics, OperationalPreview } from "@/types/management";
import { ScopeEditor, scopeForTabs } from "./ScopeEditor";
import { BaselineFlow } from "./BaselineFlow";
import { ScriptCoverage } from "./ScriptCoverage";
import { dateLabel, diagnosticText, StatusBadge } from "./OperationalShared";

interface Props {
  data: Diagnostics;
  preview: OperationalPreview | null;
  versions: BaselineSummary[];
  comparison: BaselineComparison | null;
  versionOffset: number;
  disabled: boolean;
  defaultHistoryDays?: number;
  initialTab?: "tabs" | "rules";
  onPreview: () => void;
  onInvalidate: () => void;
  onApprove: (preview: OperationalPreview) => void;
  onVersionPage: (offset: number) => void;
  onCompare: (before: string, after: string) => void;
  onComparisonClear: () => void;
  onSaveTabs: (ids: number[]) => void;
  onSaveRules: (scope: MonitorScope) => void;
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
  defaultHistoryDays = 90,
  initialTab = "tabs",
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
  const [tab, setTab] = useState<string>(initialTab);
  const [selected, setSelected] = useState(row.selected_sheet_ids);
  const [scope, setScope] = useState(() =>
    scopeForTabs(row.scope, row.selected_sheet_ids, defaultHistoryDays),
  );
  const [removeOpen, setRemoveOpen] = useState(false);
  const [removeConfirmed, setRemoveConfirmed] = useState(false);
  const [historyChoice, setHistoryChoice] = useState<"preserve" | "delete_activity">("preserve");
  const [scriptId, setScriptId] = useState("");
  const tabsDirty =
    JSON.stringify([...selected].sort((a, b) => a - b)) !==
    JSON.stringify([...row.selected_sheet_ids].sort((a, b) => a - b));
  const rulesDirty = JSON.stringify(scope) !== JSON.stringify(row.scope);
  const tabNames = [
    ["tabs", "Abas"],
    ["rules", "Regras de proteção"],
    ["baselines", "Referências"],
    ["diagnostics", "Diagnósticos"],
    ["scripts", "Apps Script"],
  ];
  function select(ids: number[]) {
    setSelected(ids);
    setScope(scopeForTabs(scope, ids, defaultHistoryDays));
    onInvalidate();
  }
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
              <span className="op-hint">
                Diagnóstico: {row.access_error ?? "acesso indisponível"}
              </span>
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
              Selecione as abas que compõem a proteção. As preferências usam a identidade da aba,
              mesmo quando seu nome muda.
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
              <Button disabled={disabled || !tabsDirty} onClick={() => onSaveTabs(selected)}>
                <Save />
                Salvar seleção de abas
              </Button>
            </div>
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
            <div className="op-actions">
              <Button variant="outline" onClick={() => setTab("rules")}>
                Continuar para regras de proteção
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
      {tab === "rules" && (
        <Card>
          <CardHeader>
            <CardTitle>Regras de proteção</CardTitle>
            <CardDescription>
              Proteja cabeçalhos, ordem, fórmulas, registros e dependências. A referência deve ser
              aprovada novamente após uma alteração legítima de regras.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="op-notice">
              <strong>Sugestão inicial a partir dos cabeçalhos reais</strong>
              <p>
                Leia somente a primeira linha, até oito colunas de cada aba selecionada. A sugestão
                delimita até 50 linhas por região; revise e amplie se necessário. Fórmulas não são
                protegidas automaticamente.
              </p>
              <Button
                type="button"
                variant="outline"
                disabled={disabled || tabsDirty || selected.length === 0 || selected.length > 30}
                onClick={() => {
                  void onSuggestRules().then((proposal) => {
                    if (proposal) {
                      setScope(proposal);
                      onInvalidate();
                    }
                  });
                }}
              >
                Sugerir regras para abas selecionadas
              </Button>
              <p className="op-hint">
                Substitui o rascunho local de regiões e colunas. Nenhuma regra é salva e nenhuma
                referência é aprovada automaticamente. Cabeçalhos vazios ou ambíguos exigem
                configuração manual. A sugestão é limitada a 30 abas por consulta; seleções maiores
                podem ser configuradas manualmente, respeitando o limite de células do escopo.
              </p>
            </div>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                onSaveRules(scope);
              }}
            >
              <ScopeEditor
                value={scope}
                tabs={row.tabs}
                disabled={disabled}
                onChange={(next) => {
                  setScope(next);
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
                <Button variant="outline" type="button" onClick={() => setTab("baselines")}>
                  Revisar referência
                </Button>
              </div>
              {tabsDirty && <p className="op-hint">Salve primeiro a seleção de abas.</p>}
              <p className="op-hint">
                A severidade é definida pela criticidade da região ou do script. Dependências de
                fórmulas e scripts são derivadas pelo motor, sem comparações no frontend.
              </p>
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
          onCapture={onPreview}
          onApprove={onApprove}
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
            {data.report ? (
              <>
                <div className="op-section-heading">
                  <StatusBadge status={data.report.status} />
                  <span className="op-hint">Detecção em {dateLabel(data.report.detected_at)}</span>
                </div>
                <p className="op-notice">
                  Conectividade:{" "}
                  {data.report.connectivity === "available" || data.report.connectivity === "online"
                    ? "Disponível na captura"
                    : data.report.connectivity}
                  . Cobertura: {data.report.coverage.complete ? "completa no escopo" : "parcial"}. A
                  integridade depende das regras configuradas e das fontes disponíveis.
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
                          {issue.code} · {issue.critical ? "crítica" : "opcional"}
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
                <details className="op-advanced">
                  <summary>Relatório avançado da verificação</summary>
                  <pre>{JSON.stringify(data.report, null, 2)}</pre>
                </details>
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
