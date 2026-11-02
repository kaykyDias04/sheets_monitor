import { useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ScopeEditor, scopeForTabs } from "./ScopeEditor";
import type { MonitorScope } from "@/types/integrity";
afterEach(cleanup);
const tabs = [{ sheet_id: 17, title: "Receitas", index: 0, hidden: false }];
function Harness() {
  const [scope, setScope] = useState<MonitorScope>(scopeForTabs(null, [17]));
  return <ScopeEditor value={scope} tabs={tabs} onChange={setScope} />;
}
describe("Regras estruturadas", () => {
  it("usa a retenção padrão somente em regras novas, preservando políticas salvas e rascunhos", () => {
    const draft = scopeForTabs(null, [17], 365);
    expect(draft.history_retention_days).toBe(365);
    expect(scopeForTabs(draft, [17, 88], 90).history_retention_days).toBe(365);
    const saved = { ...draft, history_retention_days: 30 };
    expect(scopeForTabs(saved, [17, 88], 365).history_retention_days).toBe(30);
    expect(saved.history_retention_days).toBe(30);
  });
  it("preserva identidade da aba e exige cabeçalho explícito sem inventar proteção", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    expect(screen.getByText("Identidade estável: 17")).toBeInTheDocument();
    const header = screen.getByRole("textbox", { name: /Cabeçalho protegido/ });
    expect(header).toHaveValue("");
    expect(header).toBeRequired();
    await user.type(header, "Código");
    await user.selectOptions(screen.getByLabelText("Coluna-chave"), "coluna_1");
    await user.click(screen.getByRole("button", { name: "Proteger fórmula" }));
    expect(screen.getByLabelText("Linha da fórmula")).toHaveValue(2);
    await user.selectOptions(screen.getByLabelText("Comparação"), "structural");
    expect(screen.getByLabelText("Comparação")).toHaveValue("structural");
    expect(screen.getByText(/Somente|Referências entre abas/)).toBeInTheDocument();
  });
  it("remove regras dependentes quando a coluna é removida e mantém escopos independentes", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(screen.getByRole("button", { name: "Proteger fórmula" }));
    await user.click(screen.getByRole("button", { name: "Remover coluna coluna_1" }));
    expect(screen.queryByLabelText("Linha da fórmula")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Coluna-chave")).toHaveValue("");
    const existing = scopeForTabs(null, [17]);
    existing.sheets[0].regions[0].columns[0].header = "ID";
    const result = scopeForTabs(existing, [17, 88]);
    result.sheets[0].regions[0].columns[0].header = "Outro";
    expect(existing.sheets[0].regions[0].columns[0].header).toBe("ID");
    expect(result.sheets.map((s) => s.sheet_id)).toEqual([17, 88]);
  });
});
