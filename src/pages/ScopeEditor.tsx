import type { MonitorScope } from "@/types/integrity";
import type { SpreadsheetMetadata } from "@/types/google";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Plus, Trash2 } from "lucide-react";

type Sheet = MonitorScope["sheets"][number];
type Region = Sheet["regions"][number];
type Tab = SpreadsheetMetadata["sheets"][number];
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
function nextId(prefix: string, existing: string[]) {
  let n = 1;
  while (existing.includes(`${prefix}_${n}`)) n++;
  return `${prefix}_${n}`;
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
  value,
  onChange,
  min = 1,
  max = 1000000,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
}) {
  return (
    <label className="op-field">
      {label}
      <Input
        type="number"
        value={value}
        min={min}
        max={max}
        required
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
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
  function update(work: (draft: MonitorScope) => void) {
    const draft = structuredClone(value);
    work(draft);
    onChange(draft);
  }
  return (
    <fieldset className="op-scope-editor" disabled={disabled}>
      <legend className="sr-only">Regras de integridade</legend>
      <p className="op-hint">
        Configure somente os dados que precisam de proteção. Linhas e colunas começam em 1. As
        regras serão validadas pelo motor antes da captura; salvar regras não aprova uma referência.
      </p>
      {value.sheets.length === 0 && (
        <p className="op-empty">Selecione e salve ao menos uma aba para configurar a proteção.</p>
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
          <p className="op-hint">
            A aba selecionada é obrigatória. Desmarcar a proteção de nome permite renomeá-la; a
            identidade continua vinculada ao sheetId.
          </p>
          {sheet.regions.map((region, ri) => (
            <fieldset className="op-region" key={region.id}>
              <legend>
                Região {ri + 1} · {region.id}
              </legend>
              <div className="op-form-grid">
                <Numeric
                  label="Primeira linha"
                  value={region.range.start_row + 1}
                  onChange={(v) =>
                    update((d) => {
                      d.sheets[si].regions[ri].range.start_row = v - 1;
                    })
                  }
                />
                <Numeric
                  label="Última linha"
                  value={region.range.end_row}
                  onChange={(v) =>
                    update((d) => {
                      d.sheets[si].regions[ri].range.end_row = v;
                    })
                  }
                />
                <Numeric
                  label="Primeira coluna"
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
                  value={region.range.end_col}
                  max={18278}
                  onChange={(v) =>
                    update((d) => {
                      d.sheets[si].regions[ri].range.end_col = v;
                    })
                  }
                />
                <Numeric
                  label="Linha dos cabeçalhos"
                  value={region.header_row + 1}
                  onChange={(v) =>
                    update((d) => {
                      d.sheets[si].regions[ri].header_row = v - 1;
                    })
                  }
                />
                <label className="op-field">
                  Coluna-chave
                  <select
                    value={region.key_column ?? ""}
                    onChange={(e) =>
                      update((d) => {
                        d.sheets[si].regions[ri].key_column = e.target.value || null;
                      })
                    }
                  >
                    <option value="">Sem chave: movimento pode ser inconclusivo</option>
                    {region.columns.map((column) => (
                      <option key={column.id} value={column.id}>
                        {column.header || column.id}
                      </option>
                    ))}
                  </select>
                </label>
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
              <p className="op-hint">
                Falhas críticas podem deixar a planilha Offline. Regiões opcionais geram avisos.
                Exclusões permitidas continuam registradas como alterações.
              </p>
              <h4>Colunas e cabeçalhos protegidos</h4>
              {region.columns.map((column, ci) => (
                <div className="op-column-rule" key={column.id}>
                  <label className="op-field">
                    Cabeçalho protegido
                    <Input
                      required
                      value={column.header}
                      placeholder="Texto exato do cabeçalho"
                      maxLength={512}
                      onChange={(e) =>
                        update((d) => {
                          d.sheets[si].regions[ri].columns[ci].header = e.target.value;
                        })
                      }
                    />
                    <small>Identidade lógica: {column.id}</small>
                  </label>
                  <Numeric
                    label="Posição da coluna"
                    value={column.position + 1}
                    max={18278}
                    onChange={(v) =>
                      update((d) => {
                        d.sheets[si].regions[ri].columns[ci].position = v - 1;
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
                    r.columns.push({
                      id: nextId(
                        "coluna",
                        r.columns.map((c) => c.id),
                      ),
                      header: "",
                      position:
                        Math.max(r.range.start_col - 1, ...r.columns.map((c) => c.position)) + 1,
                      protect_position: true,
                      required: true,
                    });
                  })
                }
              >
                <Plus />
                Adicionar coluna protegida
              </Button>
              <h4>Fórmulas críticas</h4>
              <p className="op-hint">
                A fórmula é lida da célula na captura. Comparação estrutural aceita apenas
                movimentos que o motor consegue provar.
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
                          {c.header || c.id}
                        </option>
                      ))}
                    </select>
                  </label>
                  <Numeric
                    label="Linha da fórmula"
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
              </div>
            </fieldset>
          ))}
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
            Adicionar região
          </Button>
        </section>
      ))}
      <section className="op-rule-sheet">
        <h3>Dependências de Apps Script</h3>
        <p className="op-hint">
          Projetos precisam estar autorizados e associados à planilha. A análise deriva referências
          estáticas; referências dinâmicas são sinalizadas como não verificáveis. Nenhum script é
          executado.
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
        <h3>Dependências e retenção</h3>
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
        <summary>Contrato avançado das regras (somente leitura)</summary>
        <pre>{JSON.stringify(value, null, 2)}</pre>
      </details>
    </fieldset>
  );
}
