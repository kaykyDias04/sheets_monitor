import { useState, type FormEvent } from "react";
import {
  CircleAlert,
  CheckCircle2,
  Cloud,
  FileCode2,
  FileSpreadsheet,
  KeyRound,
  Link2,
  LoaderCircle,
  LockKeyhole,
  LogOut,
  RefreshCw,
  ShieldCheck,
  Unplug,
  UserRound,
} from "lucide-react";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { desktop } from "@/services/foundation";
import {
  configureIntegration,
  connectIntegration,
  disconnectIntegration,
  revokeIntegration,
  integrationErrorMessage,
} from "@/services/google";
import { useGoogleIntegration } from "@/hooks/useGoogleIntegration";
import type { OAuthConfiguration, ServiceResult, ValueRange } from "@/types/google";
const scopes = {
  sheets: "https://www.googleapis.com/auth/spreadsheets.readonly",
  script: "https://www.googleapis.com/auth/script.projects.readonly",
  drive: "https://www.googleapis.com/auth/drive.metadata.readonly",
};
const authLabels = {
  unconfigured: "Cliente não configurado",
  disconnected: "Conta desconectada",
  connected: "Conta autorizada",
  reconnect_required: "Nova autorização necessária",
};
export function GooglePage() {
  const {
    state,
    sheet,
    script,
    busy,
    error,
    notice,
    changeSession,
    readSheet,
    readScript,
    refresh,
  } = useGoogleIntegration();
  const [driveOptIn, setDriveOptIn] = useState<boolean | null>(null);
  const [revokeOpen, setRevokeOpen] = useState(false);
  const [revokeConfirmed, setRevokeConfirmed] = useState(false);
  const [associate, setAssociate] = useState(false);
  const [sourceIndex, setSourceIndex] = useState(0);
  const connected = state?.auth.status === "connected";
  const disabled = !desktop || busy !== null;
  const driveChoice = driveOptIn ?? state?.drive_metadata_opt_in ?? false;
  const metadata = sheet?.sheets.status === "success" ? sheet.sheets.data : null;
  const project = script?.script.status === "success" ? script.script.data : null;
  const sourceFile = project?.files[sourceIndex] ?? project?.files[0];
  async function configure(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const form = e.currentTarget;
    const fields = new FormData(form);
    const secret = String(fields.get("client_secret") ?? "");
    const secretInput = form.elements.namedItem("client_secret") as HTMLInputElement | null;
    if (secretInput) secretInput.value = "";
    fields.delete("client_secret");
    const input: OAuthConfiguration = {
      client_id: String(fields.get("client_id") ?? "").trim(),
      drive_metadata_opt_in: driveChoice,
    };
    if (secret) input.client_secret = secret;
    await changeSession(
      "configure",
      () => configureIntegration(input),
      "Cliente Desktop salvo no armazenamento seguro. Conecte a conta para autorizar os escopos escolhidos.",
    );
  }
  async function querySheet(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const fields = new FormData(e.currentTarget);
    const ranges = String(fields.get("ranges") ?? "")
      .split(/\r?\n/)
      .map((item) => item.trim())
      .filter(Boolean);
    await readSheet({ url: String(fields.get("url") ?? "").trim(), ranges });
  }
  async function queryScript(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const fields = new FormData(e.currentTarget);
    await readScript({
      script_id: String(fields.get("script_id") ?? "").trim(),
      ...(associate && metadata ? { spreadsheet_id: metadata.spreadsheet_id } : {}),
    });
  }
  return (
    <div className="google-page">
      <div className="google-boundary">
        <ShieldCheck size={19} />
        <div>
          <strong>Integração Google · somente leitura</strong>
          <p>
            Consultas reais exigem sua autorização no aplicativo Windows. A homologação externa
            desta sprint permanece pendente até os testes com uma conta autorizada.
          </p>
        </div>
        <Badge variant="outline">SPRINT 02</Badge>
      </div>
      <div className="page-title">
        <div>
          <p className="eyebrow">CONEXÃO COM O GOOGLE WORKSPACE</p>
          <h1>Integração Google</h1>
          <p>Autorize uma conta e consulte planilhas e projetos acessíveis.</p>
        </div>
        <Button variant="outline" size="sm" disabled={disabled} onClick={() => void refresh()}>
          <RefreshCw />
          Atualizar sessão
        </Button>
      </div>
      {!desktop && (
        <div className="feedback error" role="status">
          <LockKeyhole size={18} />
          <span>
            Requer o aplicativo Windows. Esta prévia não conecta contas, não chama APIs Google e não
            apresenta resultados simulados como consultas reais.
          </span>
        </div>
      )}
      {error && (
        <div className="feedback error" role="alert">
          <CircleAlert size={18} />
          <span>{error}</span>
        </div>
      )}
      {notice && (
        <div className="feedback success" role="status">
          <CheckCircle2 size={18} />
          <span>{notice}</span>
        </div>
      )}
      {busy && (
        <div className="integration-progress" role="status">
          <LoaderCircle size={17} className="animate-spin motion-reduce:animate-none" />
          <span>
            {busy === "connect"
              ? "Aguardando a autorização no navegador do sistema. Conclua ou negue o consentimento para continuar."
              : busy === "sheet"
                ? "Consultando Sheets e os metadados Drive autorizados…"
                : busy === "script"
                  ? "Consultando o projeto Apps Script autorizado…"
                  : "Processando a sessão no backend local…"}
          </span>
        </div>
      )}
      <div className="google-connection-grid">
        <Card>
          <CardHeader>
            <div className="settings-icon">
              <UserRound size={21} />
            </div>
            <CardTitle>Conta Google</CardTitle>
            <CardDescription>OAuth Desktop com PKCE e navegador do sistema.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="account-state">
              <Badge variant="outline" className={connected ? "access-success" : "access-neutral"}>
                {desktop
                  ? state
                    ? authLabels[state.auth.status]
                    : "Estado ainda não consultado"
                  : "Disponível somente no desktop"}
              </Badge>
              {state?.auth.account && (
                <div className="account-identity">
                  <strong>{state.auth.account.name ?? "Conta Google autorizada"}</strong>
                  {state.auth.account.email && <p>{state.auth.account.email}</p>}
                  <code>Identificador: {state.auth.account.subject}</code>
                </div>
              )}
            </div>
            <div className="integration-actions">
              <Button
                disabled={disabled || !state?.configured || connected}
                onClick={() =>
                  void changeSession(
                    "connect",
                    connectIntegration,
                    "Autorização concluída. A identidade e os escopos concedidos estão disponíveis nesta sessão.",
                  )
                }
              >
                <Link2 />
                {state?.auth.status === "reconnect_required"
                  ? "Reconectar conta Google"
                  : "Conectar conta Google"}
              </Button>
              <Button
                variant="outline"
                disabled={disabled}
                onClick={() =>
                  void changeSession(
                    "disconnect",
                    disconnectIntegration,
                    "Sessão desconectada deste dispositivo. A autorização no Google permanece até ser revogada.",
                  )
                }
              >
                <LogOut />
                Desconectar
              </Button>
            </div>
            <p className="field-hint">
              Desconectar remove a sessão local. Revogar solicita ao Google o cancelamento da
              autorização.
            </p>
            <button
              className="revoke-link"
              disabled={
                disabled ||
                !state?.configured ||
                state.auth.status === "disconnected" ||
                state.auth.status === "unconfigured"
              }
              onClick={() => {
                setRevokeConfirmed(false);
                setRevokeOpen(true);
              }}
            >
              Revogar autorização no Google
            </button>
            <details className="granted-scopes">
              <summary>Permissões concedidas</summary>
              {state?.auth.granted_scopes.length ? (
                <ul>
                  {state.auth.granted_scopes.map((scope) => (
                    <li key={scope}>
                      <code>{scope}</code>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>Nenhuma permissão confirmada nesta sessão.</p>
              )}
            </details>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <div className="settings-icon">
              <KeyRound size={21} />
            </div>
            <CardTitle>Cliente OAuth Desktop</CardTitle>
            <CardDescription>Use as credenciais do seu projeto Google Cloud.</CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={(e) => void configure(e)}>
              <label htmlFor="google-client-id" className="field-label">
                Client ID do aplicativo Desktop
              </label>
              <Input
                id="google-client-id"
                name="client_id"
                key={state?.client_id ?? "unconfigured"}
                defaultValue={state?.client_id ?? ""}
                required
                disabled={disabled}
                maxLength={500}
                autoComplete="off"
                spellCheck={false}
                placeholder="…apps.googleusercontent.com"
              />
              <label htmlFor="google-client-secret" className="field-label">
                Client secret do Desktop <span>(opcional)</span>
              </label>
              <Input
                id="google-client-secret"
                name="client_secret"
                type="password"
                disabled={disabled}
                autoComplete="new-password"
                maxLength={2048}
              />
              <p className="field-hint">
                O segredo, se necessário para seu cliente Desktop, é enviado somente ao backend e
                limpo deste campo ao salvar. Deixe em branco para manter o segredo já salvo para o
                mesmo Client ID. Nunca informe sua senha Google.
              </p>
              <label className="integration-checkbox">
                <input
                  type="checkbox"
                  checked={driveChoice}
                  disabled={disabled}
                  onChange={(e) => setDriveOptIn(e.target.checked)}
                />
                <span>Autorizar leitura de metadados do Drive</span>
              </label>
              <p className="field-hint">
                Opcional: solicita drive.metadata.readonly. Sheets e Apps Script usam apenas escopos
                de leitura, além de openid, email e profile. Alterar o cliente ou os escopos encerra
                a sessão local e exige novo consentimento completo.
              </p>
              <Button variant="outline" className="mt-5" type="submit" disabled={disabled}>
                <ShieldCheck />
                {connected ? "Salvar e encerrar sessão atual" : "Salvar cliente Desktop"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
      <section className="service-indicators" aria-label="Permissões dos serviços Google">
        {(
          [
            { name: "Google Sheets", scope: scopes.sheets, icon: FileSpreadsheet },
            { name: "Google Drive", scope: scopes.drive, icon: Cloud },
            { name: "Apps Script", scope: scopes.script, icon: FileCode2 },
          ] as const
        ).map((service) => (
          <Card key={service.name}>
            <service.icon size={20} />
            <div>
              <h2>{service.name}</h2>
              <p>
                {!connected
                  ? "Aguardando autorização"
                  : service.name === "Google Drive" && !state?.drive_metadata_opt_in
                    ? "Metadados não solicitados"
                    : state?.auth.granted_scopes.includes(service.scope)
                      ? "Escopo de leitura concedido"
                      : "Escopo não concedido"}
              </p>
            </div>
          </Card>
        ))}
      </section>
      <Card className="google-query-card">
        <CardHeader>
          <CardTitle>Consultar uma planilha</CardTitle>
          <CardDescription>
            Leia metadados, abas e múltiplos intervalos; nenhum monitor será criado.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="sheet-query-form" onSubmit={(e) => void querySheet(e)}>
            <div>
              <label htmlFor="google-sheet-url" className="field-label">
                URL da planilha Google Sheets
              </label>
              <Input
                id="google-sheet-url"
                name="url"
                type="url"
                required
                disabled={disabled || !connected}
                placeholder="https://docs.google.com/spreadsheets/d/…"
              />
              <p className="field-hint">
                Use uma planilha de teste compartilhada com a conta conectada.
              </p>
            </div>
            <div>
              <label htmlFor="google-ranges" className="field-label">
                Intervalos A1 · um por linha
              </label>
              <textarea
                id="google-ranges"
                name="ranges"
                required
                defaultValue="A1:D10"
                rows={3}
                disabled={disabled || !connected}
                spellCheck={false}
              />
            </div>
            <Button type="submit" disabled={disabled || !connected}>
              <FileSpreadsheet />
              Testar acesso
            </Button>
          </form>
          {sheet && (
            <div className="sheet-query-results">
              <div className="query-result-grid">
                <ServicePanel title="Sheets API · metadados" result={sheet.sheets}>
                  {metadata && (
                    <>
                      <dl className="api-metadata">
                        <div>
                          <dt>Nome</dt>
                          <dd>{metadata.title}</dd>
                        </div>
                        <div>
                          <dt>ID</dt>
                          <dd>
                            <code>{metadata.spreadsheet_id}</code>
                          </dd>
                        </div>
                        <div>
                          <dt>Localidade / fuso</dt>
                          <dd>
                            {metadata.locale ?? "Não informado"} ·{" "}
                            {metadata.time_zone ?? "Não informado"}
                          </dd>
                        </div>
                      </dl>
                      <h3>Abas acessíveis</h3>
                      <ul className="api-sheet-list">
                        {metadata.sheets.map((tab) => (
                          <li key={tab.sheet_id}>
                            <strong>{tab.title}</strong>
                            <span>
                              ID {tab.sheet_id} · posição {tab.index}
                              {tab.hidden ? " · oculta" : ""}
                            </span>
                            <small>
                              {tab.row_count ?? "—"} linhas · {tab.column_count ?? "—"} colunas
                            </small>
                          </li>
                        ))}
                      </ul>
                    </>
                  )}
                </ServicePanel>
                <ServicePanel title="Drive API · metadados" result={sheet.drive}>
                  {sheet.drive.data && (
                    <>
                      <dl className="api-metadata">
                        <div>
                          <dt>Nome</dt>
                          <dd>{sheet.drive.data.name}</dd>
                        </div>
                        <div>
                          <dt>Tipo MIME</dt>
                          <dd>{sheet.drive.data.mime_type}</dd>
                        </div>
                        <div>
                          <dt>Modificado em</dt>
                          <dd>{sheet.drive.data.modified_time ?? "Não informado"}</dd>
                        </div>
                        <div>
                          <dt>Na lixeira</dt>
                          <dd>{sheet.drive.data.trashed ? "Sim" : "Não"}</dd>
                        </div>
                        <div>
                          <dt>Versão</dt>
                          <dd>{sheet.drive.data.version ?? "Não informada"}</dd>
                        </div>
                      </dl>
                      <p className="field-hint">
                        A data de modificação não comprova alteração estrutural nem descreve todas
                        as edições de células.
                      </p>
                    </>
                  )}
                </ServicePanel>
              </div>
              <ServicePanel title="Sheets API · valores e fórmulas" result={sheet.values}>
                {sheet.values.status === "success" && sheet.values.data && (
                  <div className="value-result-grid">
                    <ValueRanges title="Valores calculados" ranges={sheet.values.data.values} />
                    <ValueRanges title="Fórmulas · FORMULA" ranges={sheet.values.data.formulas} />
                  </div>
                )}
              </ServicePanel>
            </div>
          )}
        </CardContent>
      </Card>
      <Card className="google-query-card">
        <CardHeader>
          <CardTitle>Consultar Apps Script autorizado</CardTitle>
          <CardDescription>
            Informe o scriptId explicitamente. Código é lido como texto; nunca executado ou
            alterado.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={(e) => void queryScript(e)}>
            <label htmlFor="google-script-id" className="field-label">
              ID do projeto Apps Script
            </label>
            <div className="script-query-line">
              <Input
                id="google-script-id"
                name="script_id"
                required
                maxLength={200}
                disabled={disabled || !connected}
                placeholder="scriptId das configurações do projeto"
              />
              <Button type="submit" disabled={disabled || !connected}>
                <FileCode2 />
                Testar Apps Script
              </Button>
            </div>
            <label className="integration-checkbox">
              <input
                type="checkbox"
                checked={associate}
                disabled={disabled || !metadata}
                onChange={(e) => setAssociate(e.target.checked)}
              />
              <span>
                Associar explicitamente este projeto à planilha consultada
                {metadata ? `: ${metadata.title}` : " (consulte uma planilha antes)"}
              </span>
            </label>
            <p className="field-hint">
              Sem marcar a opção, a consulta não cria associação a uma planilha. spreadsheetId e
              scriptId são identificadores diferentes. A API Apps Script pode exigir habilitação
              adicional de acesso no painel do serviço.
            </p>
          </form>
          {script && (
            <ServicePanel title="Apps Script API" result={script.script}>
              {project && (
                <>
                  <dl className="api-metadata">
                    <div>
                      <dt>Projeto</dt>
                      <dd>{project.metadata.title}</dd>
                    </div>
                    <div>
                      <dt>scriptId</dt>
                      <dd>
                        <code>{project.metadata.script_id}</code>
                      </dd>
                    </div>
                    <div>
                      <dt>Arquivo pai informado pela API</dt>
                      <dd>{project.metadata.parent_id ?? "Não informado"}</dd>
                    </div>
                    <div>
                      <dt>Hash local do conteúdo</dt>
                      <dd>
                        <code>{project.source_hash}</code>
                      </dd>
                    </div>
                  </dl>
                  {project.files.length ? (
                    <>
                      <label htmlFor="google-script-file" className="field-label">
                        Arquivo do projeto
                      </label>
                      <select
                        id="google-script-file"
                        className="select-input script-file-select"
                        value={Math.min(sourceIndex, project.files.length - 1)}
                        onChange={(e) => setSourceIndex(Number(e.target.value))}
                      >
                        {project.files.map((file, index) => (
                          <option key={`${file.name}-${file.file_type}`} value={index}>
                            {file.name} · {file.file_type}
                          </option>
                        ))}
                      </select>
                      <pre
                        className="script-source"
                        tabIndex={0}
                        aria-label={`Código de ${sourceFile?.name ?? "arquivo"}`}
                      >
                        {sourceFile?.source}
                      </pre>
                    </>
                  ) : (
                    <p className="field-hint">O projeto retornou uma lista de arquivos vazia.</p>
                  )}
                </>
              )}
            </ServicePanel>
          )}
        </CardContent>
      </Card>
      <div className="privacy-note">
        <ShieldCheck size={23} />
        <div>
          <h2>Consultas pontuais, sem classificação de integridade</h2>
          <p>
            Esta sprint não cria baseline aprovada, não classifica dados reais como Online ou
            Offline, não analisa dependências estruturais e não inicia monitoramento contínuo.
            Valores e fontes consultados permanecem apenas na memória da interface.
          </p>
        </div>
      </div>
      <Dialog
        open={revokeOpen}
        onOpenChange={(open) => {
          if (!busy) {
            setRevokeOpen(open);
            if (open) setRevokeConfirmed(false);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Revogar autorização Google</DialogTitle>
            <DialogDescription>
              Esta ação solicita ao Google a revogação da autorização concedida ao aplicativo. Para
              novas consultas, será necessário autorizar a conta novamente.
            </DialogDescription>
          </DialogHeader>
          <label className="integration-checkbox">
            <input
              type="checkbox"
              checked={revokeConfirmed}
              disabled={busy !== null}
              onChange={(e) => setRevokeConfirmed(e.target.checked)}
            />
            <span>Confirmo que desejo revogar o acesso deste aplicativo no Google.</span>
          </label>
          <div className="integration-actions">
            <Button variant="outline" disabled={busy !== null} onClick={() => setRevokeOpen(false)}>
              Cancelar
            </Button>
            <Button
              disabled={!revokeConfirmed || busy !== null}
              onClick={() =>
                void changeSession(
                  "revoke",
                  revokeIntegration,
                  "Autorização revogada e sessão local removida.",
                ).then(() => setRevokeOpen(false))
              }
            >
              <Unplug />
              Confirmar revogação
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
function ServicePanel<T>({
  title,
  result,
  children,
}: {
  title: string;
  result: ServiceResult<T>;
  children: React.ReactNode;
}) {
  const labels = {
    success: "Leitura concluída",
    error: "Falha na consulta",
    not_authorized: "Permissão não concedida",
    not_requested: "Não solicitado",
  };
  return (
    <section className="service-result" aria-label={title}>
      <div className="service-result-heading">
        <h2>{title}</h2>
        <Badge
          variant="outline"
          className={
            result.status === "success"
              ? "access-success"
              : result.status === "error" || result.status === "not_authorized"
                ? "access-error"
                : "access-neutral"
          }
        >
          {labels[result.status]}
        </Badge>
      </div>
      {result.status === "success" ? (
        children
      ) : result.status === "not_requested" ? (
        <p className="field-hint">
          A leitura de metadados do Drive não foi solicitada. Habilite a opção no cliente e autorize
          novamente se desejar consultá-los.
        </p>
      ) : (
        <p className="api-error" role="status">
          {result.status === "not_authorized"
            ? "O escopo necessário não foi concedido. Revise as permissões e autorize a conta novamente."
            : integrationErrorMessage(result.error)}
        </p>
      )}
    </section>
  );
}
function ValueRanges({ title, ranges }: { title: string; ranges: ValueRange[] }) {
  return (
    <div className="value-ranges">
      <h3>{title}</h3>
      {!ranges.length && <p className="field-hint">Nenhum intervalo retornado.</p>}
      {ranges.map((range, index) => (
        <section key={`${range.range}-${index}`}>
          <h4>{range.range}</h4>
          {range.values.length ? (
            <div className="api-values-scroll" tabIndex={0}>
              <table aria-label={`${title} de ${range.range}`}>
                <tbody>
                  {range.values.map((row, rowIndex) => (
                    <tr key={rowIndex}>
                      {row.map((cell, columnIndex) => (
                        <td key={columnIndex}>
                          {cell === null || cell === undefined
                            ? ""
                            : typeof cell === "object"
                              ? JSON.stringify(cell)
                              : String(cell)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <p className="field-hint">Intervalo vazio.</p>
          )}
        </section>
      ))}
    </div>
  );
}
