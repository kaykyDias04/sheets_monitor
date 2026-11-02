import { useState } from "react";
import { Camera, ShieldCheck, GitCompareArrows } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { BaselineSummary } from "@/types/integrity";
import type {
  BaselineComparison,
  OperationalPreview,
  RegisteredSpreadsheet,
} from "@/types/management";
import { dateLabel } from "./OperationalShared";

function versionLabel(version: BaselineSummary) {
  return `Versão ${version.version} · regras ${version.scope_hash?.slice(0, 8) ?? "históricas"} · ${dateLabel(version.approved_at)}`;
}

function ApprovalReview({
  preview,
  onApprove,
  disabled,
}: {
  preview: OperationalPreview;
  onApprove: (preview: OperationalPreview) => void;
  disabled: boolean;
}) {
  const [confirmed, setConfirmed] = useState(false);
  const snapshot = preview.preview.snapshot;
  const cells = snapshot.scope.sheets.reduce(
    (total, sheet) =>
      total +
      sheet.regions.reduce(
        (count, region) =>
          count +
          (region.range.end_row - region.range.start_row) *
            (region.range.end_col - region.range.start_col),
        0,
      ),
    0,
  );
  const formulas = snapshot.scope.sheets.reduce(
    (total, sheet) =>
      total + sheet.regions.reduce((count, region) => count + region.formulas.length, 0),
    0,
  );
  return (
    <section className="op-baseline-review" aria-label="Revisão da referência">
      <div className="op-section-heading">
        <h3>Revise antes de aprovar</h3>
        <Badge variant="outline">
          {snapshot.coverage.complete ? "Captura completa no escopo" : "Captura incompleta"}
        </Badge>
      </div>
      <p className="op-hint">
        Capturada em {dateLabel(snapshot.captured_at)}. A referência protegerá somente o escopo
        abaixo; não representa toda a planilha.
      </p>
      <div className="op-baseline-counts">
        <span>
          <strong>{snapshot.scope.sheets.length}</strong> aba(s)
        </span>
        <span>
          <strong>{cells.toLocaleString("pt-BR")}</strong> células delimitadas
        </span>
        <span>
          <strong>{formulas}</strong> fórmula(s) protegida(s)
        </span>
        <span>
          <strong>{snapshot.scope.scripts.length}</strong> projeto(s) de script
        </span>
      </div>
      <div className="op-table-scroll">
        <table className="op-table">
          <caption className="sr-only">Estrutura identificada na captura</caption>
          <thead>
            <tr>
              <th>Aba</th>
              <th>Identidade</th>
              <th>Regiões</th>
              <th>Regras de colunas</th>
            </tr>
          </thead>
          <tbody>
            {snapshot.scope.sheets.map((rule) => (
              <tr key={rule.sheet_id}>
                <td>
                  {snapshot.sheets.find((s) => s.sheet_id === rule.sheet_id)?.title ??
                    "Aba indisponível"}
                </td>
                <td>{rule.sheet_id}</td>
                <td>
                  {rule.regions
                    .map(
                      (region) =>
                        `${region.id}: linhas ${region.range.start_row + 1}–${region.range.end_row}, colunas ${region.range.start_col + 1}–${region.range.end_col}`,
                    )
                    .join("; ")}
                </td>
                <td>{rule.regions.flatMap((r) => r.columns.map((c) => c.header)).join(", ")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {snapshot.coverage.issues.length > 0 && (
        <div className="op-notice op-warning">
          <strong>Avisos de cobertura</strong>
          <ul>
            {snapshot.coverage.issues.map((issue, i) => (
              <li key={i}>
                {issue.code}
                {issue.critical ? " · crítico" : " · opcional"}
              </li>
            ))}
          </ul>
        </div>
      )}
      <details className="op-advanced">
        <summary>Estrutura capturada e regras propostas</summary>
        <pre>{JSON.stringify(snapshot, null, 2)}</pre>
      </details>
      <p className="op-hint">
        Para ajustar a proteção, edite e salve as regras. Uma alteração nas regras exige nova
        captura. Referências anteriores permanecem versionadas.
      </p>
      <label className="op-check op-approval-check">
        <input
          type="checkbox"
          checked={confirmed}
          disabled={disabled || !snapshot.coverage.complete}
          onChange={(e) => setConfirmed(e.target.checked)}
        />
        Revisei o escopo e aprovo esta estrutura como referência de integridade.
      </label>
      <Button
        disabled={disabled || !confirmed || !snapshot.coverage.complete}
        onClick={() => onApprove(preview)}
      >
        <ShieldCheck />
        Aprovar referência versão {(preview.preview.active_baseline?.version ?? 0) + 1}
      </Button>
    </section>
  );
}
export function BaselineFlow({
  row,
  preview,
  versions,
  comparison,
  offset,
  disabled,
  dirty,
  onCapture,
  onApprove,
  onPage,
  onCompare,
  onComparisonClear,
}: {
  row: RegisteredSpreadsheet;
  preview: OperationalPreview | null;
  versions: BaselineSummary[];
  comparison: BaselineComparison | null;
  offset: number;
  disabled: boolean;
  dirty: boolean;
  onCapture: () => void;
  onApprove: (preview: OperationalPreview) => void;
  onPage: (offset: number) => void;
  onCompare: (before: string, after: string) => void;
  onComparisonClear: () => void;
}) {
  const [before, setBefore] = useState("");
  const [after, setAfter] = useState("");
  const [selectedLabels, setSelectedLabels] = useState<Record<string, string>>({});
  function remember(id: string) {
    const version = versions.find((v) => v.id === id);
    if (version) setSelectedLabels((labels) => ({ ...labels, [id]: versionLabel(version) }));
  }
  return (
    <div className="op-stack">
      <Card>
        <CardHeader>
          <CardTitle>Referência estrutural</CardTitle>
          <CardDescription>
            Capture a estrutura, revise as regras e aprove explicitamente. Nenhuma referência é
            substituída automaticamente.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="op-section-heading">
            <div>
              <Badge variant="outline">
                {row.baseline_version
                  ? `Referência ativa: versão ${row.baseline_version}`
                  : "Configuração pendente: sem referência aprovada"}
              </Badge>
              <p className="op-hint">
                A captura lê dados autorizados; a aprovação salva apenas uma referência local.
              </p>
            </div>
            <Button disabled={disabled || dirty || !row.scope} onClick={onCapture}>
              <Camera />
              Capturar estrutura para revisão
            </Button>
          </div>
          {dirty && (
            <p className="op-notice op-warning">
              Há regras ou abas não salvas. Salve a configuração antes de capturar uma nova
              referência.
            </p>
          )}
          {preview && (
            <ApprovalReview
              key={`${preview.preview.snapshot.id}:${preview.revision}`}
              preview={preview}
              disabled={disabled || dirty || preview.revision !== row.revision}
              onApprove={onApprove}
            />
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Versões e comparação</CardTitle>
          <CardDescription>
            Compare referências aprovadas. As versões são numeradas por conjunto de regras; uma
            mudança de regras inicia outra sequência. A comparação é calculada pelo backend.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {versions.length === 0 ? (
            <p className="op-empty">Nenhuma referência aprovada nesta seleção.</p>
          ) : (
            <>
              <div className="op-table-scroll">
                <table className="op-table">
                  <caption className="sr-only">Versões de referências aprovadas</caption>
                  <thead>
                    <tr>
                      <th>Versão</th>
                      <th>Aprovação</th>
                      <th>Estado</th>
                    </tr>
                  </thead>
                  <tbody>
                    {versions.map((version) => (
                      <tr key={version.id}>
                        <td>
                          Versão {version.version}
                          <small>Regras {version.scope_hash?.slice(0, 8) ?? "históricas"}</small>
                        </td>
                        <td>{dateLabel(version.approved_at)}</td>
                        <td>
                          {version.active && version.scope_hash === row.scope_hash
                            ? "Ativa na configuração atual"
                            : "Histórica"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="op-pagination">
                <span>Até 10 versões por página</span>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={disabled || offset === 0}
                  onClick={() => onPage(Math.max(0, offset - 10))}
                >
                  Versões anteriores
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={disabled || versions.length < 10}
                  onClick={() => onPage(offset + 10)}
                >
                  Mais versões
                </Button>
              </div>
              <div className="op-form-grid op-baseline-compare">
                <label className="op-field">
                  Versão anterior
                  <select
                    value={before}
                    onChange={(e) => {
                      setBefore(e.target.value);
                      remember(e.target.value);
                      onComparisonClear();
                    }}
                  >
                    <option value="">Selecionar referência</option>
                    {before && !versions.some((v) => v.id === before) && (
                      <option value={before}>{selectedLabels[before]}</option>
                    )}
                    {versions.map((v) => (
                      <option key={v.id} value={v.id}>
                        {versionLabel(v)}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="op-field">
                  Versão posterior
                  <select
                    value={after}
                    onChange={(e) => {
                      setAfter(e.target.value);
                      remember(e.target.value);
                      onComparisonClear();
                    }}
                  >
                    <option value="">Selecionar referência</option>
                    {after && !versions.some((v) => v.id === after) && (
                      <option value={after}>{selectedLabels[after]}</option>
                    )}
                    {versions.map((v) => (
                      <option key={v.id} value={v.id}>
                        {versionLabel(v)}
                      </option>
                    ))}
                  </select>
                </label>
                <Button
                  disabled={disabled || !before || !after || before === after}
                  onClick={() => onCompare(before, after)}
                >
                  <GitCompareArrows />
                  Comparar versões
                </Button>
              </div>
            </>
          )}
          {comparison && (
            <div className="op-baseline-comparison">
              <p className="op-notice">
                {comparison.scope_changed
                  ? "As versões protegem escopos diferentes. Confira as regras antes de interpretar a comparação."
                  : "As versões pertencem ao mesmo escopo de proteção."}
              </p>
              {comparison.truncated && (
                <p className="op-notice op-warning">
                  A comparação atingiu o limite de detalhes. O resultado exibido é parcial.
                </p>
              )}
              {comparison.differences.length === 0 ? (
                <p className="op-empty">
                  Nenhuma diferença estrutural identificada entre as versões.
                </p>
              ) : (
                <div className="op-table-scroll">
                  <table className="op-table">
                    <caption className="sr-only">Diferenças entre referências</caption>
                    <thead>
                      <tr>
                        <th>Estrutura</th>
                        <th>Anterior</th>
                        <th>Posterior</th>
                      </tr>
                    </thead>
                    <tbody>
                      {comparison.differences.map((diff, i) => (
                        <tr key={i}>
                          <td>{diff.path}</td>
                          <td>{diff.before ?? "Ausente"}</td>
                          <td>{diff.after ?? "Ausente"}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
