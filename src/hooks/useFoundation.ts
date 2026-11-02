import { useState, useEffect, type FormEvent } from "react";
import { desktop, getDashboard, saveSettings } from "@/services/foundation";
import { defaultSettings } from "@/fixtures";
import type { AppSettings, FoundationDashboard } from "@/types/foundation";
export function useFoundation() {
  const [data, setData] = useState<FoundationDashboard | null>(null);
  const [settings, setSettings] = useState<AppSettings>({ ...defaultSettings });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  useEffect(() => {
    let active = true;
    getDashboard()
      .then((result) => {
        if (active) {
          setData(result);
          setSettings(result.settings);
        }
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, []);
  async function refresh() {
    if (busy) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setData(await getDashboard());
      setNotice("Demonstração atualizada. Nenhuma API Google foi consultada.");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function submitSettings(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await saveSettings(settings);
      const next = await getDashboard();
      setData(next);
      setSettings(next.settings);
      setNotice(
        desktop
          ? "Preferências demonstrativas salvas no SQLite local."
          : "Preferências salvas nesta sessão de demonstração do navegador.",
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return { data, settings, setSettings, busy, error, notice, setNotice, refresh, submitSettings };
}
