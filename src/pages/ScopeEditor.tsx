import { useId, useState } from "react";
import type { MonitorScope } from "@/types/integrity";
import type { SpreadsheetMetadata } from "@/types/google";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ClipboardPaste, Plus, ShieldAlert, ShieldCheck, BellRing, Trash2 } from "lucide-react";

type Sheet = MonitorScope["sheets"][number];
type Region = Sheet["regions"][number];
type Tab = SpreadsheetMetadata["sheets"][number];

const MAX_SCOPE_CELLS = 20_000;

export function columnLetter(zeroBasedIndex: number): string {
  if (!Number.isInteger(zeroBasedIndex) || zeroBasedIndex < 0) return "?";
  let n = zeroBasedIndex + 1;
  let letters = "";
  while (n > 0) {
    const rem = (n - 1) % 26;
    letters = String.fromCharCode(65 + rem) + letters;
    n = Math.floor((n - 1) / 26);
  }
  return letters;
}

export function rangeA1(range: Region["range"]): string {
  const startCol = columnLetter(range.start_col);
  const endCol = columnLetter(Math.max(range.start_col, range.end_col - 1));
  const startRow = range.start_row + 1;
  const endRow = Math.max(startRow, range.end_row);
  return `${startCol}${startRow}:${endCol}${endRow}`;
}

export const emptyScope = (defaultHistoryDays = 90): MonitorScope => ({
  schema_version: 1,
  sheets: [],
  scripts: [],
  named_range_ids: [],
  snapshot_retention_days: 7,
  history_retention_days: defaultHistoryDays,
});

function newRegion(id: string): Region {
  return {
    id,
    range: { start_row: 0, end_row: 100, start_col: 0, end_col: 4 },
    header_row: 0,
    columns: [{ id: "coluna_1", header: "", position: 0, protect_position: true, required: true }],
    key_column: null,
    critical: true,
    allow_record_deletion: true,
    formulas: [],
  };
}

export function scopeForTabs(
  scope: MonitorScope | null,
  ids: number[],
  defaultHistoryDays = 90,
): MonitorScope {
  const source = scope ?? emptyScope(defaultHistoryDays);
  return {
    ...structuredClone(source),
    sheets: ids.map((sheet_id) =>
      structuredClone(
        source.sheets.find((s) => s.sheet_id === sheet_id) ?? {
          sheet_id,
          protect_title: true,
          protect_position: false,
          regions: [newRegion("dados")],
        },
      ),
    ),
  };
}

export function countScopeStats(scope: MonitorScope) {
  let columns = 0;
  let formulas = 0;
  let cells = 0;
  for (const sheet of scope.sheets) {
    for (const region of sheet.regions) {
      columns += region.columns.length;
      formulas += region.formulas.length;
      const rows = Math.max(0, region.range.end_row - region.range.start_row);
      const cols = Math.max(0, region.range.end_col - region.range.start_col);
      cells += rows * cols;
    }
  }
  return { sheets: scope.sheets.length, columns, formulas, cells };
}

/**
 * Pendências de preenchimento que o formulário explica antes de enviar ao backend.
 * O motor Rust permanece como única autoridade de domínio e revalida todo o escopo ao salvar.
 */
export function scopeFormProblems(scope: MonitorScope, tabs: Tab[]): string[] {
  const problems: string[] = [];
  const title = (id: number) => tabs.find((tab) => tab.sheet_id === id)?.title ?? `ID ${id}`;
  let totalCells = 0;

  for (const sheet of scope.sheets) {
    if (sheet.regions.length === 0) {
      problems.push(`Aba "${title(sheet.sheet_id)}": adicione ao menos uma região protegida.`);
    }
    sheet.regions.forEach((region, ri) => {
      const where = `Aba "${title(sheet.sheet_id)}", região ${ri + 1}`;
      const numbers = [
        region.range.start_row,
        region.range.end_row,
        region.range.start_col,
        region.range.end_col,
        region.header_row,
        ...region.columns.map((c) => c.position),
        ...region.formulas.map((f) => f.baseline_row),
      ];
      if (numbers.some((n) => !Number.isInteger(n) || n < 0)) {
        problems.push(`${where}: preencha linhas e colunas com números inteiros a partir de 1.`);
        return;
      }
      if (region.range.start_row >= region.range.end_row || region.range.end_row > 1_000_000) {
        problems.push(
          `${where}: a "Última linha" (${region.range.end_row}) precisa ser maior ou igual à "Primeira linha" (${region.range.start_row + 1}) e até 1.000.000.`,
        );
      }
      if (region.range.start_col >= region.range.end_col || region.range.end_col > 18_278) {
        problems.push(
          `${where}: a "Última coluna" (${region.range.end_col}) precisa ser maior ou igual à "Primeira coluna" (${region.range.start_col + 1}) e até 18.278.`,
        );
      }
      if (region.header_row < region.range.start_row || region.header_row >= region.range.end_row) {
        problems.push(
          `${where}: a "Linha dos cabeçalhos" (${region.header_row + 1}) precisa estar dentro do intervalo de linhas (${region.range.start_row + 1} a ${region.range.end_row}).`,
        );
      }
      if (region.columns.length === 0) {
        problems.push(`${where}: adicione ao menos uma coluna protegida ou remova a região.`);
      }
      const seenHeaders = new Set<string>();
      const seenPositions = new Set<number>();
      region.columns.forEach((column, ci) => {
        const colLabel = `Coluna ${columnLetter(column.position)} (${column.position + 1})`;
        if (column.header.trim() === "") {
          problems.push(
            `${where}: informe o cabeçalho protegido da coluna ${ci + 1} (${colLabel}) ou remova a coluna.`,
          );
        } else {
          if (seenHeaders.has(column.header)) {
            problems.push(
              `${where}: o cabeçalho "${column.header}" está repetido. Cada coluna protegida precisa ter um texto único na região.`,
            );
          }
          seenHeaders.add(column.header);
        }
        if (seenPositions.has(column.position)) {
          problems.push(
            `${where}: mais de uma regra usa a ${colLabel}. Escolha posições diferentes para cada coluna.`,
          );
        }
        seenPositions.add(column.position);
        if (column.position < region.range.start_col || column.position >= region.range.end_col) {
          problems.push(
            `${where}: a coluna "${column.header || ci + 1}" está na ${colLabel}, fora da área monitorada (colunas ${region.range.start_col + 1} a ${region.range.end_col}). Aumente a "Última coluna" para pelo menos ${column.position + 1}.`,
          );
        }
      });
      region.formulas.forEach((formula, fi) => {
        if (
          formula.baseline_row <= region.header_row ||
          formula.baseline_row >= region.range.end_row
        ) {
          problems.push(
            `${where}: a fórmula ${fi + 1} (linha ${formula.baseline_row + 1}) precisa ficar abaixo da linha dos cabeçalhos (${region.header_row + 1}) e até a última linha (${region.range.end_row}).`,
          );
        }
      });
      const rows = Math.max(0, region.range.end_row - region.range.start_row);
      const cols = Math.max(0, region.range.end_col - region.range.start_col);
      totalCells += rows * cols;
    });
  }

  if (totalCells > MAX_SCOPE_CELLS) {
    problems.push(
      `O escopo soma ${totalCells.toLocaleString("pt-BR")} células e ultrapassa o limite de ${MAX_SCOPE_CELLS.toLocaleString("pt-BR")}. Reduza a "Última linha" ou a "Última coluna" nas abas selecionadas.`,
    );
  }
  scope.scripts.forEach((script, i) => {
    if (script.script_id.trim() === "") {
      problems.push(`Dependência de Apps Script ${i + 1}: informe o ID do projeto ou remova-a.`);
    }
  });
  for (const field of ["snapshot_retention_days", "history_retention_days"] as const) {
    if (!Number.isInteger(scope[field]) || scope[field] < 1) {
      problems.push("Retenção: informe a quantidade de dias com um número inteiro positivo.");
    }
  }
  return [...new Set(problems)];
}

function nextId(prefix: string, existing: string[]) {
  let n = 1;
  while (existing.includes(`${prefix}_${n}`)) n++;
  return `${prefix}_${n}`;
}

export function parsePastedHeaders(raw: string): string[] {
  const firstLine =
    raw
      .split(/\r?\n/)
      .map((line) => line.trim())
      .find(Boolean) ?? "";
  if (!firstLine) return [];
  const separator = firstLine.includes("\t")
    ? "\t"
    : firstLine.includes(";")
      ? ";"
      : firstLine.includes(",")
        ? ","
        : null;
  const parts = separator ? firstLine.split(separator) : [firstLine];
  return parts.map((item) => item.trim()).filter(Boolean);
}

function Check({
  label,
  value,
  onChange,
  disabled,
}: {
  label: string;
  value: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className="op-check">
      <input
        type="checkbox"
        checked={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      {label}
    </label>
  );
}

function Numeric({
  label,
  hint,
  value,
  onChange,
  min = 1,
  max = 1000000,
}: {
  label: string;
  hint?: string;
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
}) {
  const id = useId();
  return (
    <div className="op-field">
      <label htmlFor={id}>{label}</label>
      <Input
        id={id}
        type="number"
        value={value}
        min={min}
        max={max}
        required
        aria-describedby={hint ? `${id}-hint` : undefined}
        onChange={(e) => onChange(Number(e.target.value))}
      />
      {hint && <small id={`${id}-hint`}>{hint}</small>}
    </div>
  );
}

type PresetId = "structure" | "strict_rows" | "alert_only";

function activePreset(region: Region): PresetId {
  if (!region.critical) return "alert_only";
  if (!region.allow_record_deletion) return "strict_rows";
  return "structure";
}

function PasteHeadersBox({
  onApply,
  disabled,
}: {
  onApply: (headers: string[]) => void;
  disabled: boolean;
}) {
  const [text, setText] = useState("");
  const parsed = parsePastedHeaders(text);
  return (
    <details className="op-paste-helper">
      <summary>
        <ClipboardPaste size={14} />
        Atalho: colar cabeçalhos copiados da planilha
      </summary>
      <div className="op-paste-body">
        <p className="op-hint">
          Copie a linha de títulos no Google Sheets (Ctrl+C) e cole abaixo, ou separe os nomes por
          vírgula ou ponto e vírgula.
        </p>
        <div className="op-paste-row">
          <Input
            value={text}
            disabled={disabled}
            placeholder="Ex.: Código, Cliente, Data, Valor"
            aria-label="Cabeçalhos copiados da planilha"
            onChange={(e) => setText(e.target.value)}
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={disabled || parsed.length === 0}
            onClick={() => {
              onApply(parsed);
              setText("");
            }}
          >
            Preencher {parsed.length > 0 ? `${parsed.length} coluna(s)` : "colunas"}
          </Button>
        </div>
      </div>
    </details>
  );
}

export function ScopeEditor({
  value,
  tabs,
  onChange,
  disabled = false,
}: {
  value: MonitorScope;
  tabs: Tab[];
  onChange: (scope: MonitorScope) => void;
  disabled?: boolean;
}) {
  const stats = countScopeStats(value);
  const overBudget = stats.cells > MAX_SCOPE_CELLS;

  function update(work: (draft: MonitorScope) => void) {
    const draft = structuredClone(value);
    work(draft);
    onChange(draft);
  }

  return (
    <fieldset className="op-scope-editor" disabled={disabled}>
      <legend className="sr-only">Regras de integridade</legend>

      {value.sheets.length === 0 ? (
        <p className="op-empty">Selecione e salve ao menos uma aba para configurar a proteção.</p>
      ) : (
        <div className={`op-scope-summary ${overBudget ? "over-budget" : ""}`}>
          <div>
            <strong>Resumo do que será protegido</strong>
            <p className="op-hint">
              Informe apenas os títulos das colunas importantes e a área da tabela. Salvar as regras
              guarda sua configuração; em seguida, você captura e aprova a referência.
            </p>
          </div>
          <div className="op-scope-metrics">
            <span>
              <strong>{stats.sheets}</strong> aba(s)
            </span>
            <span>
              <strong>{stats.columns}</strong> coluna(s)
            </span>
            <span className={overBudget ? "metric-danger" : ""}>
              <strong>{stats.cells.toLocaleString("pt-BR")}</strong> /{" "}
              {MAX_SCOPE_CELLS.toLocaleString("pt-BR")} células
            </span>
          </div>
        </div>
      )}

      {value.sheets.map((sheet, si) => (
        <section className="op-rule-sheet" key={sheet.sheet_id}>
          <div className="op-section-heading">
            <div>
              <h3>
                {tabs.find((tab) => tab.sheet_id === sheet.sheet_id)?.title ?? "Aba não encontrada"}
              </h3>
              <p className="op-hint">Identidade estável: {sheet.sheet_id}</p>
            </div>
            <div className="op-inline-checks">
              <Check
                label="Proteger nome da aba"
                value={sheet.protect_title}
                onChange={(v) =>
                  update((d) => {
                    d.sheets[si].protect_title = v;
                  })
                }
              />
              <Check
                label="Proteger posição da aba"
                value={sheet.protect_position}
                onChange={(v) =>
                  update((d) => {
                    d.sheets[si].protect_position = v;
                  })
                }
              />
            </div>
          </div>
          <p className="op-hint">
            Mesmo que alguém renomeie a aba no Google Sheets, o monitor continua identificando-a
            pelo ID acima. Desmarque &ldquo;Proteger nome da aba&rdquo; se renomeá-la for permitido.
          </p>

          {sheet.regions.map((region, ri) => {
            const preset = activePreset(region);
            return (
              <fieldset className="op-region" key={region.id}>
                <legend>
                  Tabela {ri + 1} ({region.id}) · Intervalo {rangeA1(region.range)}
                </legend>

                <div className="op-step-block">
                  <h4>1. Como reagir a mudanças nesta tabela?</h4>
                  <div
                    className="op-preset-grid"
                    role="group"
                    aria-label="Modo de proteção da região"
                  >
                    <button
                      type="button"
                      className={`op-preset-card ${preset === "structure" ? "active" : ""}`}
                      aria-pressed={preset === "structure"}
                      onClick={() =>
                        update((d) => {
                          const r = d.sheets[si].regions[ri];
                          r.critical = true;
                          r.allow_record_deletion = true;
                        })
                      }
                    >
                      <ShieldCheck size={16} />
                      <div>
                        <strong>Cabeçalhos e estrutura (Recomendado)</strong>
                        <small>
                          Fica Offline se apagarem, trocarem ou renomearem colunas. Edições comuns
                          nas linhas geram aviso sem derrubar o status.
                        </small>
                      </div>
                    </button>
                    <button
                      type="button"
                      className={`op-preset-card ${preset === "strict_rows" ? "active" : ""}`}
                      aria-pressed={preset === "strict_rows"}
                      onClick={() =>
                        update((d) => {
                          const r = d.sheets[si].regions[ri];
                          r.critical = true;
                          r.allow_record_deletion = false;
                        })
                      }
                    >
                      <ShieldAlert size={16} />
                      <div>
                        <strong>Bloquear também exclusão de linhas</strong>
                        <small>
                          Ideal para históricos e cadastros onde nenhuma linha existente deve ser
                          excluída.
                        </small>
                      </div>
                    </button>
                    <button
                      type="button"
                      className={`op-preset-card ${preset === "alert_only" ? "active" : ""}`}
                      aria-pressed={preset === "alert_only"}
                      onClick={() =>
                        update((d) => {
                          const r = d.sheets[si].regions[ri];
                          r.critical = false;
                          r.allow_record_deletion = true;
                        })
                      }
                    >
                      <BellRing size={16} />
                      <div>
                        <strong>Apenas alertar (sem ficar Offline)</strong>
                        <small>
                          Registra mudanças e envia alerta no Windows, mantendo a planilha Online.
                        </small>
                      </div>
                    </button>
                  </div>
                  <div className="op-inline-checks">
                    <Check
                      label="Região crítica"
                      value={region.critical}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].critical = v;
                        })
                      }
                    />
                    <Check
                      label="Permitir exclusão de registros"
                      value={region.allow_record_deletion}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].allow_record_deletion = v;
                        })
                      }
                    />
                  </div>
                </div>

                <div className="op-step-block">
                  <h4>2. Quais colunas e cabeçalhos devem existir?</h4>
                  <p className="op-hint">
                    Digite o nome exato do título que aparece na linha de cabeçalho da planilha
                    (diferenciando maiúsculas e acentos).
                  </p>
                  <PasteHeadersBox
                    disabled={disabled}
                    onApply={(headers) =>
                      update((d) => {
                        const r = d.sheets[si].regions[ri];
                        const unique = [...new Set(headers)].slice(0, 128);
                        r.columns = unique.map((header, idx) => ({
                          id: `coluna_${idx + 1}`,
                          header,
                          position: r.range.start_col + idx,
                          protect_position: true,
                          required: true,
                        }));
                        r.range.end_col = Math.max(
                          r.range.end_col,
                          Math.min(18278, r.range.start_col + unique.length),
                        );
                        r.formulas = [];
                        r.key_column = null;
                      })
                    }
                  />
                  {region.columns.map((column, ci) => (
                    <div className="op-column-rule" key={column.id}>
                      <label className="op-field">
                        Cabeçalho protegido
                        <Input
                          required
                          value={column.header}
                          placeholder={`Ex.: título exato na Coluna ${columnLetter(column.position)}`}
                          maxLength={512}
                          onChange={(e) =>
                            update((d) => {
                              d.sheets[si].regions[ri].columns[ci].header = e.target.value;
                            })
                          }
                        />
                        <small>
                          Coluna {columnLetter(column.position)} · ID interno: {column.id}
                        </small>
                      </label>
                      <Numeric
                        label="Posição da coluna"
                        hint={`Coluna ${columnLetter(column.position)} na planilha`}
                        value={column.position + 1}
                        max={18278}
                        onChange={(v) =>
                          update((d) => {
                            const r = d.sheets[si].regions[ri];
                            const pos = v - 1;
                            r.columns[ci].position = pos;
                            if (pos >= r.range.end_col && pos < 18278) {
                              r.range.end_col = pos + 1;
                            }
                          })
                        }
                      />
                      <div>
                        <Check
                          label="Obrigatória"
                          value={column.required}
                          onChange={(v) =>
                            update((d) => {
                              d.sheets[si].regions[ri].columns[ci].required = v;
                            })
                          }
                        />
                        <Check
                          label="Manter ordem"
                          value={column.protect_position}
                          onChange={(v) =>
                            update((d) => {
                              d.sheets[si].regions[ri].columns[ci].protect_position = v;
                            })
                          }
                        />
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        aria-label={`Remover coluna ${column.header || column.id}`}
                        onClick={() =>
                          update((d) => {
                            const r = d.sheets[si].regions[ri];
                            r.columns.splice(ci, 1);
                            r.formulas = r.formulas.filter((f) => f.column_id !== column.id);
                            if (r.key_column === column.id) r.key_column = null;
                          })
                        }
                      >
                        <Trash2 />
                      </Button>
                    </div>
                  ))}
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() =>
                      update((d) => {
                        const r = d.sheets[si].regions[ri];
                        const nextPos = Math.min(
                          18277,
                          Math.max(r.range.start_col - 1, ...r.columns.map((c) => c.position)) + 1,
                        );
                        r.columns.push({
                          id: nextId(
                            "coluna",
                            r.columns.map((c) => c.id),
                          ),
                          header: "",
                          position: nextPos,
                          protect_position: true,
                          required: true,
                        });
                        if (nextPos >= r.range.end_col) {
                          r.range.end_col = nextPos + 1;
                        }
                      })
                    }
                  >
                    <Plus />
                    Adicionar coluna protegida
                  </Button>
                </div>

                <div className="op-step-block">
                  <h4>3. Qual área da aba faz parte desta tabela? ({rangeA1(region.range)})</h4>
                  <p className="op-hint">
                    Delimitamos apenas as linhas e colunas necessárias para respeitar o orçamento de
                    20 mil células por planilha.
                  </p>
                  <div className="op-form-grid">
                    <Numeric
                      label="Primeira linha"
                      hint="Onde começa a tabela"
                      value={region.range.start_row + 1}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].range.start_row = v - 1;
                        })
                      }
                    />
                    <Numeric
                      label="Última linha"
                      hint="Até qual linha acompanhar"
                      value={region.range.end_row}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].range.end_row = v;
                        })
                      }
                    />
                    <Numeric
                      label="Linha dos cabeçalhos"
                      hint="Geralmente linha 1"
                      value={region.header_row + 1}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].header_row = v - 1;
                        })
                      }
                    />
                    <Numeric
                      label="Primeira coluna"
                      hint={`Coluna ${columnLetter(region.range.start_col)}`}
                      value={region.range.start_col + 1}
                      max={18278}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].range.start_col = v - 1;
                        })
                      }
                    />
                    <Numeric
                      label="Última coluna"
                      hint={`Coluna ${columnLetter(Math.max(0, region.range.end_col - 1))}`}
                      value={region.range.end_col}
                      max={18278}
                      onChange={(v) =>
                        update((d) => {
                          d.sheets[si].regions[ri].range.end_col = v;
                        })
                      }
                    />
                    <div className="op-field">
                      <label htmlFor={`key-col-${sheet.sheet_id}-${region.id}`}>Coluna-chave</label>
                      <select
                        id={`key-col-${sheet.sheet_id}-${region.id}`}
                        aria-describedby={`key-col-hint-${sheet.sheet_id}-${region.id}`}
                        value={region.key_column ?? ""}
                        onChange={(e) =>
                          update((d) => {
                            d.sheets[si].regions[ri].key_column = e.target.value || null;
                          })
                        }
                      >
                        <option value="">Sem chave (opcional)</option>
                        {region.columns.map((column) => (
                          <option key={column.id} value={column.id}>
                            {column.header || column.id}
                          </option>
                        ))}
                      </select>
                      <small id={`key-col-hint-${sheet.sheet_id}-${region.id}`}>
                        Coluna com ID único da linha (ex.: Código, CPF, Pedido)
                      </small>
                    </div>
                  </div>
                </div>

                <div className="op-step-block">
                  <h4>4. Fórmulas críticas (opcional)</h4>
                  <p className="op-hint">
                    Use quando uma célula específica contiver uma fórmula que não pode ser apagada
                    ou trocada por número fixo.
                  </p>
                  {region.formulas.map((formula, fi) => (
                    <div className="op-formula-rule" key={formula.id}>
                      <label className="op-field">
                        Coluna da fórmula
                        <select
                          value={formula.column_id}
                          onChange={(e) =>
                            update((d) => {
                              d.sheets[si].regions[ri].formulas[fi].column_id = e.target.value;
                            })
                          }
                        >
                          {region.columns.map((c) => (
                            <option key={c.id} value={c.id}>
                              {c.header || c.id} ({columnLetter(c.position)})
                            </option>
                          ))}
                        </select>
                      </label>
                      <Numeric
                        label="Linha da fórmula"
                        hint="Deve ser maior que o cabeçalho"
                        value={formula.baseline_row + 1}
                        onChange={(v) =>
                          update((d) => {
                            d.sheets[si].regions[ri].formulas[fi].baseline_row = v - 1;
                          })
                        }
                      />
                      <label className="op-field">
                        Comparação
                        <select
                          value={formula.mode}
                          onChange={(e) =>
                            update((d) => {
                              d.sheets[si].regions[ri].formulas[fi].mode = e.target.value as
                                "exact" | "structural";
                            })
                          }
                        >
                          <option value="exact">Exata</option>
                          <option value="structural">Estrutural</option>
                        </select>
                      </label>
                      <Check
                        label="Permitir movimento de linha"
                        value={formula.allow_row_shift}
                        onChange={(v) =>
                          update((d) => {
                            d.sheets[si].regions[ri].formulas[fi].allow_row_shift = v;
                          })
                        }
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        aria-label={`Remover fórmula ${fi + 1} da região ${region.id}`}
                        onClick={() =>
                          update((d) => {
                            d.sheets[si].regions[ri].formulas.splice(fi, 1);
                          })
                        }
                      >
                        <Trash2 />
                      </Button>
                    </div>
                  ))}
                  <div className="op-actions">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={region.columns.length === 0}
                      onClick={() =>
                        update((d) => {
                          const r = d.sheets[si].regions[ri];
                          r.formulas.push({
                            id: nextId(
                              "formula",
                              r.formulas.map((f) => f.id),
                            ),
                            column_id: r.columns[0].id,
                            baseline_row: r.header_row + 1,
                            mode: "exact",
                            allow_row_shift: false,
                          });
                        })
                      }
                    >
                      <Plus />
                      Proteger fórmula
                    </Button>
                    {sheet.regions.length > 1 && (
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          update((d) => {
                            d.sheets[si].regions.splice(ri, 1);
                          })
                        }
                      >
                        <Trash2 />
                        Remover região
                      </Button>
                    )}
                  </div>
                </div>
              </fieldset>
            );
          })}

          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() =>
              update((d) => {
                const s = d.sheets[si];
                s.regions.push(
                  newRegion(
                    nextId(
                      "regiao",
                      s.regions.map((r) => r.id),
                    ),
                  ),
                );
              })
            }
          >
            <Plus />
            Adicionar outra tabela nesta aba
          </Button>
        </section>
      ))}

      <details
        className="op-advanced"
        open={value.scripts.length > 0 || value.named_range_ids.length > 0}
      >
        <summary>
          Opções avançadas (Apps Script, intervalos nomeados e retenção de histórico)
        </summary>
        <div className="op-advanced-sections">
          <section className="op-rule-sheet">
            <h3>Dependências de Apps Script</h3>
            <p className="op-hint">
              Projetos precisam estar autorizados e associados à planilha. A análise deriva
              referências estáticas; referências dinâmicas são sinalizadas como não verificáveis.
              Nenhum script é executado.
            </p>
            {value.scripts.map((script, i) => (
              <div className="op-script-rule" key={i}>
                <label className="op-field">
                  ID do projeto Apps Script
                  <Input
                    value={script.script_id}
                    required
                    onChange={(e) =>
                      update((d) => {
                        d.scripts[i].script_id = e.target.value.trim();
                      })
                    }
                  />
                </label>
                <Check
                  label="Dependência crítica"
                  value={script.critical}
                  onChange={(v) =>
                    update((d) => {
                      d.scripts[i].critical = v;
                    })
                  }
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={`Remover dependência de script ${i + 1}`}
                  onClick={() =>
                    update((d) => {
                      d.scripts.splice(i, 1);
                    })
                  }
                >
                  <Trash2 />
                </Button>
              </div>
            ))}
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={value.scripts.length >= 4}
              onClick={() =>
                update((d) => {
                  d.scripts.push({ script_id: "", critical: false });
                })
              }
            >
              <Plus />
              Adicionar dependência de script
            </Button>
          </section>

          <section className="op-rule-sheet">
            <h3>Intervalos nomeados e retenção</h3>
            <p className="op-hint">
              Referências entre abas nas fórmulas e scripts são verificadas pelo motor. Abas fora do
              escopo podem tornar a análise inconclusiva. Intervalos nomeados protegidos exigem seu
              identificador Google.
            </p>
            <label className="op-field">
              IDs de intervalos nomeados (um por linha)
              <textarea
                rows={3}
                value={value.named_range_ids.join("\n")}
                onChange={(e) =>
                  update((d) => {
                    d.named_range_ids = e.target.value
                      .split(/\r?\n/)
                      .map((id) => id.trim())
                      .filter(Boolean);
                  })
                }
              />
            </label>
            <div className="op-form-grid">
              <Numeric
                label="Retenção de snapshots (dias)"
                value={value.snapshot_retention_days}
                max={365}
                onChange={(v) =>
                  update((d) => {
                    d.snapshot_retention_days = v;
                  })
                }
              />
              <Numeric
                label="Retenção do histórico (dias)"
                value={value.history_retention_days}
                max={3650}
                onChange={(v) =>
                  update((d) => {
                    d.history_retention_days = v;
                  })
                }
              />
            </div>
          </section>

          <details className="op-advanced">
            <summary>JSON técnico das regras (somente leitura)</summary>
            <pre>{JSON.stringify(value, null, 2)}</pre>
          </details>
        </div>
      </details>
    </fieldset>
  );
}
