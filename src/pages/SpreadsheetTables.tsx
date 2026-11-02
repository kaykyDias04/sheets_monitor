import { useState } from "react";
import { ArrowRight, FileSpreadsheet, Search, Plus, Users, FolderOpen } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { StatusBadge, dateLabel, EmptyState, statusNames } from "./OperationalShared";
import type { RegisteredSpreadsheet } from "@/types/management";
import type { DiscoveryPage, DiscoveryQuery, DiscoveredFile } from "@/types/discovery";

export function RegisteredTable({
  rows,
  onOpen,
  onDiscover,
  disabled = false,
}: {
  rows: RegisteredSpreadsheet[];
  onOpen: (row: RegisteredSpreadsheet) => void;
  onDiscover: () => void;
  disabled?: boolean;
}) {
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("all");
  const visible = rows.filter(
    (row) =>
      `${row.name} ${row.account_email ?? ""}`
        .toLocaleLowerCase("pt-BR")
        .includes(search.toLocaleLowerCase("pt-BR")) &&
      (status === "all" || row.status === status),
  );
  return (
    <Card>
      <CardHeader>
        <div className="op-section-heading">
          <div>
            <CardTitle>
              Planilhas cadastradas <span className="op-count">{rows.length}/30</span>
            </CardTitle>
            <CardDescription>
              Arquivos selecionados por você. Verificações são executadas sob demanda.
            </CardDescription>
          </div>
          <Button onClick={onDiscover} disabled={disabled}>
            <Plus />
            Encontrar planilhas
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {rows.length === 0 ? (
          <EmptyState
            title="Seu monitoramento começa aqui"
            description="Encontre as planilhas disponíveis na conta Google ou adicione um link. Nenhum arquivo será cadastrado automaticamente."
            action="Descobrir planilhas"
            onAction={onDiscover}
          />
        ) : (
          <>
            <div className="op-toolbar">
              <label className="op-search">
                <Search size={16} />
                <Input
                  aria-label="Buscar planilhas cadastradas"
                  placeholder="Buscar por nome ou conta"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
              </label>
              <select
                aria-label="Filtrar estado das planilhas"
                value={status}
                onChange={(e) => setStatus(e.target.value)}
              >
                <option value="all">Todos os estados</option>
                {Object.entries(statusNames)
                  .filter(([key]) => key !== "checking")
                  .map(([key, label]) => (
                    <option key={key} value={key}>
                      {label}
                    </option>
                  ))}
              </select>
            </div>
            <div className="op-table-scroll">
              <table className="op-table">
                <caption className="sr-only">Planilhas selecionadas para monitoramento</caption>
                <thead>
                  <tr>
                    <th>Planilha</th>
                    <th>Integridade</th>
                    <th>Última verificação</th>
                    <th>Referência</th>
                    <th>
                      <span className="sr-only">Ação</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {visible.map((row) => (
                    <tr key={row.spreadsheet_id}>
                      <td>
                        <button
                          className="op-file-link"
                          disabled={disabled}
                          onClick={() => onOpen(row)}
                        >
                          <FileSpreadsheet size={20} />
                          <span>
                            <strong>{row.name}</strong>
                            <small>{row.account_email ?? "Conta Google conectada"}</small>
                          </span>
                        </button>
                        {row.authorization === "error" && (
                          <small className="op-access-error">
                            Acesso indisponível; consulte o diagnóstico
                          </small>
                        )}
                      </td>
                      <td>
                        <StatusBadge status={row.status} />
                      </td>
                      <td>{dateLabel(row.last_checked_at)}</td>
                      <td>
                        {row.baseline_version
                          ? `Versão ${row.baseline_version}`
                          : "Aguardando aprovação"}
                      </td>
                      <td>
                        <Button
                          variant="ghost"
                          size="icon"
                          aria-label={`Abrir detalhes de ${row.name}`}
                          disabled={disabled}
                          onClick={() => onOpen(row)}
                        >
                          <ArrowRight />
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            {visible.length === 0 && (
              <p className="op-empty">Nenhuma planilha corresponde aos filtros.</p>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}

export function DiscoveryPanel({
  query,
  onQuery,
  page,
  onSearch,
  onNext,
  onFirst,
  onRegister,
  registered,
  disabled,
  canDiscover,
}: {
  query: DiscoveryQuery;
  onQuery: (query: DiscoveryQuery) => void;
  page: DiscoveryPage | null;
  onSearch: () => void;
  onNext: () => void;
  onFirst: () => void;
  onRegister: (file: DiscoveredFile) => void;
  registered: string[];
  disabled: boolean;
  canDiscover: boolean;
}) {
  return (
    <Card id="discovery-panel">
      <CardHeader>
        <CardTitle>Encontrar no Google Drive</CardTitle>
        <CardDescription>
          Liste planilhas acessíveis sem baixar suas células. O limite de 30 se aplica aos
          cadastros, não aos resultados da pesquisa.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="op-discovery-form"
          onSubmit={(e) => {
            e.preventDefault();
            onSearch();
          }}
        >
          <label className="op-field">
            Pesquisar no Drive
            <Input
              placeholder="Nome da planilha"
              value={query.search}
              onChange={(e) => onQuery({ ...query, search: e.target.value, page_token: null })}
              maxLength={200}
            />
          </label>
          <label className="op-field">
            Ordenação
            <select
              value={query.order}
              onChange={(e) =>
                onQuery({
                  ...query,
                  order: e.target.value as DiscoveryQuery["order"],
                  page_token: null,
                })
              }
            >
              <option value="name">Nome: A a Z</option>
              <option value="name_desc">Nome: Z a A</option>
              <option value="modified_desc">Modificação: recentes primeiro</option>
              <option value="modified_asc">Modificação: antigas primeiro</option>
            </select>
          </label>
          <Button type="submit" disabled={disabled || !canDiscover}>
            <Search />
            Buscar planilhas
          </Button>
        </form>
        {!canDiscover && (
          <p className="op-notice">
            A descoberta precisa da permissão de leitura de metadados do Drive. Ative a descoberta
            em Configurações e conclua o consentimento Google, se solicitado.
          </p>
        )}
        <details className="op-advanced">
          <summary>Unidade compartilhada específica</summary>
          <label className="op-field">
            ID da unidade compartilhada (opcional)
            <Input
              value={query.drive_id ?? ""}
              onChange={(e) =>
                onQuery({ ...query, drive_id: e.target.value.trim() || null, page_token: null })
              }
              placeholder="Deixe vazio para pesquisar nas unidades acessíveis"
            />
          </label>
          <p className="op-hint">
            Se o Google informar pesquisa incompleta, restrinja a uma unidade autorizada. Os
            resultados sempre respeitam as permissões da conta.
          </p>
        </details>
        {page && (
          <>
            <div className="op-discovery-summary">
              <span>{page.files.length} arquivo(s) nesta página</span>
              <Badge variant="outline">
                {page.corpus === "drive" ? "Unidade selecionada" : "Unidades acessíveis"}
              </Badge>
            </div>
            {page.incomplete_search && (
              <p className="op-notice op-warning" role="status">
                <strong>Pesquisa incompleta</strong>
                <br />O Google não pesquisou todas as unidades. Estes resultados não representam
                todas as planilhas acessíveis; tente uma unidade específica.
              </p>
            )}
            {page.files.length === 0 && (
              <p className="op-empty">
                Nenhuma planilha nesta página.
                {page.next_page_token
                  ? " Há mais páginas; continue a busca."
                  : " Tente outro nome ou revise as permissões."}
              </p>
            )}
            {page.files.length > 0 && (
              <div className="op-table-scroll">
                <table className="op-table">
                  <caption className="sr-only">Planilhas descobertas no Drive</caption>
                  <thead>
                    <tr>
                      <th>Arquivo</th>
                      <th>Compartilhamento</th>
                      <th>Modificação</th>
                      <th>Seleção</th>
                    </tr>
                  </thead>
                  <tbody>
                    {page.files.map((file) => (
                      <tr key={file.id}>
                        <td>
                          <span className="op-file-link">
                            <FileSpreadsheet size={18} />
                            <strong>{file.name}</strong>
                          </span>
                        </td>
                        <td>
                          {file.drive_id ? (
                            <span className="op-shared">
                              <FolderOpen size={14} />
                              Unidade compartilhada
                            </span>
                          ) : file.shared ? (
                            <span className="op-shared">
                              <Users size={14} />
                              Compartilhada
                            </span>
                          ) : (
                            "Acesso da conta"
                          )}
                        </td>
                        <td>
                          {file.modified_time ? dateLabel(file.modified_time) : "Não informada"}
                        </td>
                        <td>
                          <Button
                            variant="outline"
                            size="sm"
                            disabled={
                              disabled || registered.includes(file.id) || registered.length >= 30
                            }
                            onClick={() => onRegister(file)}
                          >
                            {registered.includes(file.id)
                              ? "Cadastrada"
                              : registered.length >= 30
                                ? "Limite atingido"
                                : "Monitorar"}
                          </Button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            <div className="op-pagination">
              <span>
                {page.complete
                  ? "Fim dos resultados desta pesquisa"
                  : page.incomplete_search
                    ? "Cobertura incompleta"
                    : "Há mais resultados"}
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={disabled || !query.page_token}
                onClick={onFirst}
              >
                Primeira página
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={disabled || !page.next_page_token}
                onClick={onNext}
              >
                Próxima página
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
