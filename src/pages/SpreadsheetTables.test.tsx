import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DiscoveryPanel, RegisteredTable } from "./SpreadsheetTables";
import type { DiscoveryPage, DiscoveryQuery } from "@/types/discovery";
afterEach(cleanup);
const query: DiscoveryQuery = {
  search: "",
  order: "name",
  page_token: null,
  drive_id: null,
  page_size: 50,
};
const empty: DiscoveryPage = {
  files: [],
  next_page_token: "opaque-next",
  incomplete_search: true,
  complete: false,
  corpus: "allDrives",
};
function panel(page = empty, canDiscover = true) {
  const onNext = vi.fn();
  const onRegister = vi.fn();
  render(
    <DiscoveryPanel
      query={query}
      onQuery={() => undefined}
      page={page}
      onSearch={() => undefined}
      onNext={onNext}
      onFirst={() => undefined}
      onRegister={onRegister}
      registered={[]}
      disabled={false}
      canDiscover={canDiscover}
    />,
  );
  return { onNext, onRegister };
}
describe("Descoberta e cadastro visual", () => {
  it("permite próxima página vazia e não apresenta resultados incompletos como totais", async () => {
    const user = userEvent.setup();
    const { onNext } = panel();
    expect(screen.getByText("Pesquisa incompleta")).toBeInTheDocument();
    expect(screen.getByText(/Há mais páginas; continue a busca/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Próxima página" }));
    expect(onNext).toHaveBeenCalledOnce();
  });
  it("mostra mais de 30 descobertas, identifica compartilhadas e cadastra somente por ação", async () => {
    const user = userEvent.setup();
    const files = Array.from({ length: 31 }, (_, i) => ({
      id: `fixture${i}`,
      name: `Teste ${i}`,
      mime_type: "application/vnd.google-apps.spreadsheet",
      modified_time: null,
      shared: i === 0,
      drive_id: i === 1 ? "fixture-drive" : null,
      resource_key: null,
    }));
    const { onRegister } = panel({
      ...empty,
      files,
      incomplete_search: false,
      next_page_token: null,
      complete: true,
    });
    expect(screen.getAllByRole("button", { name: "Monitorar" })).toHaveLength(31);
    expect(screen.getByText("Compartilhada")).toBeInTheDocument();
    expect(screen.getByText("Unidade compartilhada")).toBeInTheDocument();
    expect(onRegister).not.toHaveBeenCalled();
    await user.click(screen.getAllByRole("button", { name: "Monitorar" })[0]);
    expect(onRegister).toHaveBeenCalledWith(files[0]);
  });
  it("explica falta de permissão sem habilitar descoberta", () => {
    panel(empty, false);
    expect(screen.getByRole("button", { name: "Buscar planilhas" })).toBeDisabled();
    expect(screen.getByText(/permissão de leitura de metadados do Drive/)).toBeInTheDocument();
  });
  it("inicia cadastro com estado vazio sem fixtures de planilhas", () => {
    render(<RegisteredTable rows={[]} onOpen={() => undefined} onDiscover={() => undefined} />);
    expect(screen.getByText("Seu monitoramento começa aqui")).toBeInTheDocument();
    expect(screen.getByText(/Nenhum arquivo será cadastrado automaticamente/)).toBeInTheDocument();
    expect(screen.queryByText("Online")).not.toBeInTheDocument();
  });
});
