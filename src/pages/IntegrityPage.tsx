import { useEffect, useRef, useState } from "react";
import {
  ShieldCheck,
  Camera,
  CheckCircle2,
  LoaderCircle,
  FileSearch,
  History,
  AlertCircle,
} from "lucide-react";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { desktop } from "@/services/platform";
import { getIntegrationState } from "@/services/google";
import * as api from "@/services/integrity";
import type { IntegrationState } from "@/types/google";
import type {
  MonitorScope,
  EngineEnvelope,
  CapturePreview,
  BaselineSummary,
  ValidationResult,
  IntegrityEvent,
  ScopeProposal,
} from "@/types/integrity";
const statusNames = {
  online: "Online",
  offline: "Offline",
  indeterminate: "Indeterminado",
  checking: "Verificando",
};
const example: MonitorScope = {
  schema_version: 1,
  sheets: [
    {
      sheet_id: 0,
      protect_title: true,
      protect_position: false,
      regions: [
        {
          id: "data",
          range: { start_row: 0, end_row: 100, start_col: 0, end_col: 4 },
          header_row: 0,
          columns: [
            { id: "record_id", header: "ID", position: 0, protect_position: true, required: true },
            { id: "amount", header: "Valor", position: 1, protect_position: true, required: true },
          ],
          key_column: "record_id",
          critical: true,
          allow_record_deletion: true,
          formulas: [],
        },
      ],
    },
  ],
  scripts: [],
  named_range_ids: [],
  snapshot_retention_days: 7,
  history_retention_days: 90,
};
export function IntegrityPage() {
  const [session, setSession] = useState<IntegrationState | null>(null);
  const [url, setUrl] = useState("");
  const [scopeJson, setScopeJson] = useState(JSON.stringify(example, null, 2));
  const [preview, setPreview] = useState<CapturePreview | null>(null);
  const [proposal, setProposal] = useState<ScopeProposal | null>(null);
  const [result, setResult] = useState<ValidationResult | null>(null);
  const [history, setHistory] = useState<BaselineSummary[]>([]);
  const [offset, setOffset] = useState(0);
  const [events, setEvents] = useState<IntegrityEvent[]>([]);
  const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState<string | null>(desktop ? "load" : null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const mounted = useRef(true);
  const locked = useRef(false);
  const current = useRef<IntegrationState | null>(null);
  function clearResults() {
    setPreview(null);
    setResult(null);
    setHistory([]);
    setEvents([]);
    setConfirmed(false);
    setOffset(0);
  }
  function acceptSession(next: IntegrationState) {
    if (current.current && current.current.generation !== next.generation) {
      clearResults();
      setProposal(null);
      setUrl("");
      setScopeJson(JSON.stringify(example, null, 2));
    }
    current.current = next;
    setSession(next);
  }
  useEffect(() => {
    mounted.current = true;
    let active = true;
    if (desktop)
      getIntegrationState()
        .then((s) => {
          if (active) {
            current.current = s;
            setSession(s);
          }
        })
        .catch((e) => {
          if (active) setError(api.engineError(e));
        })
        .finally(() => {
          if (active) setBusy(null);
        });
    return () => {
      active = false;
      mounted.current = false;
    };
  }, []);
  async function run<T>(
    kind: string,
    work: () => Promise<EngineEnvelope<T>>,
    apply: (data: T) => void,
  ) {
    if (locked.current) return;
    locked.current = true;
    setBusy(kind);
    setError("");
    setNotice("");
    try {
      const before = await getIntegrationState();
      if (!mounted.current) return;
      if (current.current && current.current.generation !== before.generation) {
        acceptSession(before);
        throw { code: "stale_session" };
      }
      acceptSession(before);
      const response = await work();
      const after = await getIntegrationState();
      if (!mounted.current) return;
      if (
        response.generation !== after.generation ||
        before.generation !== after.generation ||
        before.auth.account?.subject !== after.auth.account?.subject ||
        response.account_subject !== after.auth.account?.subject ||
        after.auth.status !== "connected"
      ) {
        clearResults();
        acceptSession(after);
        throw { code: "stale_session" };
      }
      acceptSession(after);
      apply(response.data);
    } catch (e) {
      if (mounted.current) {
        setError(api.engineError(e));
        try {
          const next = await getIntegrationState();
          if (mounted.current) acceptSession(next);
        } catch {
          /* Preserve the safe operation error. */
        }
      }
    } finally {
      locked.current = false;
      if (mounted.current) setBusy(null);
    }
  }
  function parseScope(): MonitorScope {
    try {
      return JSON.parse(scopeJson) as MonitorScope;
    } catch {
      throw { code: "invalid_json" };
    }
  }
  const query = preview
    ? { spreadsheet_id: preview.snapshot.spreadsheet_id, scope_hash: preview.snapshot.scope_hash }
    : null;
  const connected = session?.auth.status === "connected";
  const disabled = !desktop || !connected || busy !== null;
  function changedScope(value: string) {
    setScopeJson(value);
    clearResults();
  }
  return (
    <div className="engine-page">
      <div className="google-boundary">
        <ShieldCheck />
        <div>
          <strong>Integridade sob demanda · somente leitura</strong>
          <p>
            A análise usa apenas o escopo configurado e uma referência aprovada. Capturar e validar
            não alteram seus arquivos Google. Nenhum monitoramento contínuo é iniciado.
          </p>
        </div>
      </div>
      <div className="page-title">
        <div>
          <p className="eyebrow">SPRINT 03 · OPERAÇÕES TÉCNICAS</p>
          <h1>Integridade técnica</h1>
          <p>Capture, revise a referência e execute uma verificação manual.</p>
        </div>
        <Badge variant="outline">
          {connected ? "Conta conectada" : "Conecte a conta na área Google"}
        </Badge>
      </div>
      {!desktop && (
        <p role="status" className="feedback error">
          Estas operações exigem o aplicativo Windows. Nenhum resultado real é simulado nesta
          prévia.
        </p>
      )}
      {error && (
        <div role="alert" className="feedback error">
          <AlertCircle />
          {error}
        </div>
      )}
      {notice && (
        <div role="status" className="feedback success">
          <CheckCircle2 />
          {notice}
        </div>
      )}
      {busy && (
        <div role="status" className="integration-progress">
          <LoaderCircle className="animate-spin" />
          Verificando: {busy}. Aguarde a conclusão.
        </div>
      )}
      <Card>
        <CardHeader>
          <CardTitle>1. Definir o escopo</CardTitle>
          <CardDescription>
            Os índices de linhas e colunas começam em zero; os limites finais são exclusivos. A
            proposta inicial usa a primeira aba, até 100 linhas e 8 colunas. Revise cabeçalhos,
            chave, fórmulas e criticidade antes de capturar.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <label className="field-label" htmlFor="integrity-url">
            URL da planilha autorizada
          </label>
          <Input
            id="integrity-url"
            value={url}
            disabled={disabled}
            onChange={(e) => {
              setUrl(e.target.value);
              clearResults();
              setProposal(null);
            }}
            placeholder="https://docs.google.com/spreadsheets/d/…"
          />
          <div className="integration-actions">
            <Button
              variant="outline"
              disabled={disabled || !url.trim()}
              onClick={() =>
                void run(
                  "preparando proposta",
                  () => api.proposeScope(url),
                  (data) => {
                    clearResults();
                    setProposal(data);
                    setScopeJson(JSON.stringify(data.scope, null, 2));
                    setNotice("Proposta preparada para revisão. Nenhuma referência foi aprovada.");
                  },
                )
              }
            >
              <FileSearch />
              Preparar proposta de escopo
            </Button>
          </div>
          {proposal && (
            <details>
              <summary>Abas disponíveis para configurar o escopo</summary>
              <ul className="engine-tabs">
                {proposal.available_sheets.map((s) => (
                  <li key={s.sheet_id}>
                    <code>{s.sheet_id}</code> · {s.title}
                  </li>
                ))}
              </ul>
            </details>
          )}
          <label className="field-label" htmlFor="integrity-scope">
            Escopo técnico em JSON
          </label>
          <textarea
            id="integrity-scope"
            className="engine-json-editor"
            value={scopeJson}
            disabled={disabled}
            onChange={(e) => changedScope(e.target.value)}
            spellCheck={false}
          />
          <p className="field-hint">
            O exemplo é apenas um modelo de regras. Ajuste os nomes reais e as regiões: valores de
            linhas são reduzidos a hashes; cabeçalhos e fórmulas protegidas necessários à análise
            ficam no SQLite local. Inclua script IDs autorizados em scripts, com criticidade
            explícita. Dependências dinâmicas podem tornar a verificação Indeterminada.
          </p>
          <Button
            disabled={disabled || !url.trim()}
            onClick={() => {
              clearResults();
              void run(
                "capturando snapshot",
                () => api.captureSnapshot(url, parseScope()),
                (data) => {
                  setPreview(data);
                  setResult(null);
                  setEvents([]);
                  setHistory([]);
                  setConfirmed(false);
                  setOffset(0);
                  setNotice(
                    "Snapshot capturado. Confira a cobertura e as regras antes de aprovar.",
                  );
                },
              );
            }}
          >
            <Camera />
            Capturar snapshot
          </Button>
        </CardContent>
      </Card>
      {preview && (
        <Card>
          <CardHeader>
            <CardTitle>2. Revisar a referência</CardTitle>
            <CardDescription>
              A aprovação vale para esta conta, planilha, escopo e conteúdo capturado. Alterar o
              escopo exige outra referência; a versão anterior permanece preservada.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <dl className="api-metadata">
              <div>
                <dt>Snapshot</dt>
                <dd>
                  <code>{preview.snapshot.id}</code>
                </dd>
              </div>
              <div>
                <dt>Capturado em</dt>
                <dd>{preview.snapshot.captured_at}</dd>
              </div>
              <div>
                <dt>Cobertura da captura</dt>
                <dd>
                  {preview.snapshot.coverage.complete
                    ? "Completa no escopo solicitado"
                    : "Parcial: confira os diagnósticos"}
                </dd>
              </div>
              <div>
                <dt>Referência ativa</dt>
                <dd>
                  {preview.active_baseline
                    ? `Versão ${preview.active_baseline.version}`
                    : "Nenhuma neste escopo"}
                </dd>
              </div>
            </dl>
            {preview.snapshot.coverage.issues.length > 0 && (
              <ul className="engine-findings">
                {preview.snapshot.coverage.issues.map((issue, i) => (
                  <li key={i}>
                    {issue.code} ·{" "}
                    {issue.critical ? "Verificação obrigatória" : "Cobertura opcional"}
                  </li>
                ))}
              </ul>
            )}
            <p className="field-hint">
              Escopo desta referência: {preview.snapshot.scope.sheets.length} aba(s),{" "}
              {preview.snapshot.scope.sheets.reduce(
                (n, s) =>
                  n +
                  s.regions.reduce(
                    (m, r) =>
                      m +
                      (r.range.end_row - r.range.start_row) * (r.range.end_col - r.range.start_col),
                    0,
                  ),
                0,
              )}{" "}
              células na janela e{" "}
              {preview.snapshot.scope.sheets.reduce(
                (n, s) => n + s.regions.reduce((m, r) => m + r.formulas.length, 0),
                0,
              )}{" "}
              regra(s) de fórmula. {preview.snapshot.scope.scripts.length} projeto(s) de script
              configurado(s). A cobertura da análise de scripts pode ser parcial; confira os
              detalhes.
            </p>
            <details className="engine-preview">
              <summary>Examinar regras, fórmulas protegidas, hashes e cobertura</summary>
              <pre className="json-preview" tabIndex={0}>
                {JSON.stringify(preview.snapshot, null, 2)}
              </pre>
            </details>
            <label className="integration-checkbox">
              <input
                type="checkbox"
                checked={confirmed}
                disabled={disabled || !preview.snapshot.coverage.complete}
                onChange={(e) => setConfirmed(e.target.checked)}
              />
              <span>
                Revisei o conteúdo e aprovo explicitamente este snapshot como nova versão da
                referência estrutural para este escopo.
              </span>
            </label>
            <Button
              disabled={disabled || !confirmed || !preview.snapshot.coverage.complete}
              onClick={() =>
                void run(
                  "aprovando referência",
                  () =>
                    api.approveBaseline({
                      expected_generation: preview.session_generation,
                      snapshot_id: preview.snapshot.id,
                      snapshot_fingerprint: preview.snapshot_fingerprint,
                      expected_version: preview.active_baseline?.version ?? 0,
                      confirmed,
                    }),
                  (data) => {
                    setPreview((p) => (p ? { ...p, active_baseline: data } : p));
                    setConfirmed(false);
                    setNotice(
                      `Referência versão ${data.version} aprovada. As versões anteriores foram preservadas.`,
                    );
                  },
                )
              }
            >
              <ShieldCheck />
              Aprovar nova versão de baseline
            </Button>
            <div className="integration-actions">
              <Button
                variant="outline"
                disabled={disabled || !query}
                onClick={() => {
                  if (query)
                    void run(
                      "consultando versões",
                      () => api.listBaselines(query, 0),
                      (data) => {
                        setHistory(data);
                        setOffset(0);
                      },
                    );
                }}
              >
                <History />
                Consultar versões anteriores
              </Button>
            </div>
            {history.length > 0 && (
              <>
                <ul className="engine-history">
                  {history.map((b) => (
                    <li key={b.id}>
                      Versão {b.version} · {b.active ? "Ativa" : "Preservada"} · {b.approved_at}
                    </li>
                  ))}
                </ul>
                <div className="integration-actions">
                  <Button
                    variant="outline"
                    disabled={disabled || offset === 0}
                    onClick={() => {
                      if (query)
                        void run(
                          "consultando versões",
                          () => api.listBaselines(query, Math.max(0, offset - 5)),
                          (data) => {
                            setHistory(data);
                            setOffset(Math.max(0, offset - 5));
                          },
                        );
                    }}
                  >
                    Mais recentes
                  </Button>
                  <Button
                    variant="outline"
                    disabled={disabled || history.length < 5}
                    onClick={() => {
                      if (query)
                        void run(
                          "consultando versões",
                          () => api.listBaselines(query, offset + 5),
                          (data) => {
                            setHistory(data);
                            setOffset(offset + 5);
                          },
                        );
                    }}
                  >
                    Mais antigas
                  </Button>
                </div>
              </>
            )}
          </CardContent>
        </Card>
      )}
      <Card>
        <CardHeader>
          <CardTitle>3. Validar a captura e consultar eventos</CardTitle>
          <CardDescription>
            A verificação compara a captura selecionada com o baseline ativo. Para observar novas
            mudanças no Google, capture novamente antes de validar.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="integration-actions">
            <Button
              disabled={disabled || !preview?.active_baseline}
              onClick={() => {
                if (preview)
                  void run(
                    "validando integridade",
                    () => api.validateIntegrity(preview.snapshot.id),
                    (data) => {
                      setResult(data);
                      setEvents(data.events);
                      setNotice(
                        "Verificação registrada. Ocorrências persistentes são deduplicadas; mudanças novas e recuperação geram eventos distintos.",
                      );
                    },
                  );
              }}
            >
              Validar integridade da captura
            </Button>
            <Button
              variant="outline"
              disabled={disabled || !query}
              onClick={() => {
                if (query) void run("consultando eventos", () => api.listEvents(query), setEvents);
              }}
            >
              Consultar últimos eventos
            </Button>
            <Button
              variant="outline"
              disabled={disabled || !query}
              onClick={() => {
                if (query)
                  void run(
                    "aplicando retenção",
                    () => api.pruneHistory(query),
                    (count) =>
                      setNotice(
                        `${count} registros fora da retenção foram removidos. Referências aprovadas e estado de incidentes foram preservados.`,
                      ),
                  );
              }}
            >
              Aplicar retenção configurada
            </Button>
          </div>
          {result && (
            <section aria-label="Resultado da integridade">
              <div className="service-result-heading">
                <h2>Resultado no escopo aprovado</h2>
                <Badge
                  variant="outline"
                  className={
                    result.report.status === "online"
                      ? "access-success"
                      : result.report.status === "offline"
                        ? "access-error"
                        : "access-neutral"
                  }
                >
                  {statusNames[result.report.status]}
                </Badge>
              </div>
              <p className="field-hint">
                Resultado registrado em {result.report.detected_at}. Conectividade:{" "}
                {result.report.connectivity}. {result.report.differences.length} diferenças
                observadas nesta captura. A classificação não representa abas ou regras fora do
                escopo.
              </p>
              {result.report.findings.length > 0 ? (
                <ul className="engine-findings">
                  {result.report.findings.map((f, i) => (
                    <li key={`${f.key}-${i}`}>
                      <strong>{f.code}</strong> · {f.severity} · {f.outcome}
                      <p>
                        Regra: {f.rule_id}
                        {f.sheet_id !== null ? ` · Aba ${f.sheet_id}` : ""}
                        {f.location ? ` · ${f.location}` : ""}
                      </p>
                      {f.expected && <p>Esperado: {f.expected}</p>}
                      {f.found && <p>Encontrado: {f.found}</p>}
                    </li>
                  ))}
                </ul>
              ) : (
                <p>
                  Nenhuma violação crítica ou incerteza obrigatória foi encontrada nas regras
                  avaliadas.
                </p>
              )}
            </section>
          )}
          {events.length > 0 && (
            <section className="service-result" aria-label="Eventos de integridade">
              <h2>Eventos registrados</h2>
              <ul className="engine-findings">
                {events.map((event) => (
                  <li key={event.event_id}>
                    <strong>{event.kind}</strong> · {event.detected_at}
                    <p>
                      {event.diagnosis}
                      {event.location ? ` · ${event.location}` : ""}
                    </p>
                    <code>{event.event_id}</code>
                  </li>
                ))}
              </ul>
            </section>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
