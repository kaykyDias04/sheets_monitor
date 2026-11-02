import { useEffect, useRef, useState } from "react";
import { desktop } from "@/services/platform";
import * as api from "@/services/google";
import type { IntegrationState, SheetAccessResult, ScriptAccessResult } from "@/types/google";
export function useGoogleIntegration() {
  const [state, setState] = useState<IntegrationState | null>(null);
  const [sheet, setSheet] = useState<SheetAccessResult | null>(null);
  const [script, setScript] = useState<ScriptAccessResult | null>(null);
  const [busy, setBusy] = useState<string | null>(desktop ? "load" : null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const current = useRef<IntegrationState | null>(null);
  const locked = useRef(false);
  const mounted = useRef(true);
  function applyState(next: IntegrationState) {
    if (current.current?.generation !== next.generation) {
      setSheet(null);
      setScript(null);
    }
    current.current = next;
    setState(next);
  }
  useEffect(() => {
    mounted.current = true;
    if (!desktop)
      return () => {
        mounted.current = false;
      };
    let active = true;
    api
      .getIntegrationState()
      .then((next) => {
        if (active) {
          current.current = next;
          setState(next);
        }
      })
      .catch((e) => {
        if (active) setError(api.integrationErrorMessage(e));
      })
      .finally(() => {
        if (active) setBusy(null);
      });
    return () => {
      active = false;
      mounted.current = false;
    };
  }, []);
  async function action(kind: string, work: () => Promise<void>) {
    if (locked.current) return;
    locked.current = true;
    setBusy(kind);
    setError("");
    setNotice("");
    try {
      await work();
    } catch (e) {
      if (mounted.current) {
        setError(api.integrationErrorMessage(e));
        try {
          const next = await api.getIntegrationState();
          if (mounted.current) applyState(next);
        } catch {
          /* Preserve the original safe error. */
        }
      }
    } finally {
      locked.current = false;
      if (mounted.current) setBusy(null);
    }
  }
  async function changeSession(
    kind: string,
    work: () => Promise<IntegrationState>,
    success: string,
  ) {
    await action(kind, async () => {
      setSheet(null);
      setScript(null);
      const next = await work();
      if (mounted.current) {
        applyState(next);
        setNotice(success);
      }
    });
  }
  function accepts(result: { generation: number; account_subject: string }) {
    return (
      current.current?.auth.status === "connected" &&
      result.generation === current.current.generation &&
      result.account_subject === current.current.auth.account?.subject
    );
  }
  async function readSheet(input: { url: string; ranges: string[] }) {
    await action("sheet", async () => {
      setSheet(null);
      setScript(null);
      const result = await api.testSheetAccess(input);
      if (!mounted.current) return;
      if (!accepts(result)) {
        applyState(await api.getIntegrationState());
        throw { code: "stale_session" };
      }
      setSheet(result);
      setNotice(
        "Consulta concluída. Confira os diagnósticos independentes de Sheets, valores e Drive.",
      );
    });
  }
  async function readScript(input: { script_id: string; spreadsheet_id?: string }) {
    await action("script", async () => {
      setScript(null);
      const result = await api.testScriptAccess(input);
      if (!mounted.current) return;
      if (!accepts(result)) {
        applyState(await api.getIntegrationState());
        throw { code: "stale_session" };
      }
      setScript(result);
    });
  }
  return {
    state,
    sheet,
    script,
    busy,
    error,
    notice,
    changeSession,
    readSheet,
    readScript,
    refresh: () =>
      changeSession(
        "load",
        api.getIntegrationState,
        "Estado da sessão consultado no armazenamento seguro local.",
      ),
  };
}
