import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { ScriptSummary } from "@/types/management";
import type { StructuralSnapshot } from "@/types/integrity";
import { dateLabel } from "./OperationalShared";

function record(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" ? (value as Record<string, unknown>) : {};
}
function text(value: unknown, fallback = "Não disponível") {
  return typeof value === "string" || typeof value === "number" ? String(value) : fallback;
}
function DependencyRows({ values }: { values: unknown[] }) {
  const [offset, setOffset] = useState(0);
  return (
    <>
      <div className="op-table-scroll">
        <table className="op-table">
          <caption className="sr-only">Dependências reconhecidas pelo analisador</caption>
          <thead>
            <tr>
              <th>Arquivo e linha</th>
              <th>Referência</th>
              <th>Cobertura</th>
              <th>Diagnóstico</th>
            </tr>
          </thead>
          <tbody>
            {values.slice(offset, offset + 20).map((value, i) => {
              const dep = record(value);
              return (
                <tr key={offset + i}>
                  <td>
                    {text(dep.file)}
                    <small>Linha {text(dep.line)}</small>
                  </td>
                  <td>
                    {text(dep.sheet_name, "Sem aba determinada")}
                    <small>{text(dep.kind)}</small>
                  </td>
                  <td>
                    {dep.coverage === "static"
                      ? "Estática"
                      : dep.coverage === "partial"
                        ? "Parcial"
                        : "Não verificável"}
                  </td>
                  <td>{text(dep.detail_code, "Sem detalhe adicional")}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <div className="op-pagination">
        <span>
          {values.length === 0
            ? "Nenhuma dependência estática reconhecida"
            : `${offset + 1}–${Math.min(offset + 20, values.length)} de ${values.length} dependências`}
        </span>
        <Button
          variant="outline"
          size="sm"
          disabled={offset === 0}
          onClick={() => setOffset(Math.max(0, offset - 20))}
        >
          Dependências anteriores
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={offset + 20 >= values.length}
          onClick={() => setOffset(offset + 20)}
        >
          Próximas dependências
        </Button>
      </div>
    </>
  );
}
export function ScriptCoverage({
  scripts,
  previewScripts,
}: {
  scripts: ScriptSummary[];
  previewScripts?: StructuralSnapshot["scripts"];
}) {
  const displayScripts = scripts.map((script) => ({
    ...script,
    analysis:
      previewScripts === undefined
        ? script.analysis
        : (previewScripts.find((captured) => captured.script_id === script.script_id) ?? null),
  }));
  return (
    <div className="op-stack">
      <div className="op-notice">
        <strong>Análise de dependências, sem executar código</strong>
        <p>
          Somente projetos explicitamente autorizados e associados são consultados. A descoberta
          automática de todos os projetos vinculados não é garantida. Use o ID do projeto quando
          necessário.
        </p>
      </div>
      {previewScripts !== undefined && (
        <div className="op-notice">
          <strong>Análise da captura em revisão — ainda não aprovada</strong>
          <p>
            A cobertura abaixo pertence à captura em revisão. Projetos ausentes dessa captura não
            reutilizam uma análise anterior.
          </p>
        </div>
      )}
      {scripts.length === 0 && (
        <p className="op-empty">
          Nenhum projeto Apps Script associado. Cadastre um projeto autorizado para consultar a
          cobertura.
        </p>
      )}
      {displayScripts.map((script) => (
        <Card key={script.script_id}>
          <CardHeader>
            <div className="op-section-heading">
              <CardTitle>{script.name ?? "Projeto Apps Script"}</CardTitle>
              <Badge variant="outline">
                {script.critical ? "Dependência crítica" : "Dependência opcional"}
              </Badge>
            </div>
            <p className="op-hint">
              <code>{script.script_id}</code> · Última consulta:{" "}
              {script.last_queried_at ? dateLabel(script.last_queried_at) : "Ainda não consultado"}
            </p>
            <p className="op-hint">
              Autorização:{" "}
              {script.authorization === "success" || script.authorization === "available"
                ? "Acesso disponível na última consulta"
                : script.authorization === "error"
                  ? "Consulta recusada; revise a autorização"
                  : "Ainda não confirmada"}
            </p>
          </CardHeader>
          <CardContent>
            {script.analysis ? (
              <>
                <div className="op-notice">
                  <strong>
                    {script.analysis.analysis.complete
                      ? "Cobertura estática completa no código analisado"
                      : "Cobertura parcial — há dependências não verificáveis"}
                  </strong>
                  <p>
                    Uma referência estática reconhecida não comprova o comportamento de caminhos
                    dinâmicos em execução.
                  </p>
                </div>
                <h4 className="op-small-heading">Arquivos consultados</h4>
                <ul className="op-file-list">
                  {script.analysis.files.map((file) => (
                    <li key={`${file.file_type}:${file.name}`}>
                      <span>{file.name}</span>
                      <Badge variant="outline">{file.file_type}</Badge>
                    </li>
                  ))}
                </ul>
                {script.analysis.analysis.limitations.length > 0 && (
                  <details className="op-advanced">
                    <summary>
                      Limitações da análise ({script.analysis.analysis.limitations.length})
                    </summary>
                    <ul>
                      {script.analysis.analysis.limitations.map((limitation, i) => (
                        <li key={i}>{limitation}</li>
                      ))}
                    </ul>
                  </details>
                )}
                <h4 className="op-small-heading">Dependências reconhecidas e não verificáveis</h4>
                <DependencyRows
                  key={`${script.script_id}:${script.analysis.source_hash}`}
                  values={script.analysis.analysis.dependencies}
                />
              </>
            ) : (
              <p className="op-hint">
                Ainda não há análise para este projeto no escopo atual. Inclua o projeto nas regras,
                capture a estrutura e consulte o diagnóstico.
              </p>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
