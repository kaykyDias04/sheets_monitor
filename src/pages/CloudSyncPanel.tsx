import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { desktop } from "@/services/platform";
import { connectIntegration, getIntegrationState } from "@/services/google";
import { managementError } from "@/services/management";
import { formatLocalDateTime } from "@/lib/syncPresentation";
import type { IntegrationState } from "@/types/google";
import type { EngineEnvelope } from "@/types/integrity";
interface SyncStatus {
  state: string;
  message: string;
  conflicts: { commit_id: string; created_at: string }[];
  last_synced_at: string | null;
  conflict_token: string | null;
}
const cloudScope = "https://www.googleapis.com/auth/drive.appdata";
const auditScope = "https://www.googleapis.com/auth/admin.reports.audit.readonly";
export function CloudSyncPanel({ onChanged }: { onChanged: () => void }) {
  const [session, setSession] = useState<IntegrationState | null>(null);
  const [cloud, setCloud] = useState(false);
  const [audit, setAudit] = useState(false);
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const live = useRef(true);
  const locked = useRef(false);
  const load = useCallback(async (loadChoices = false) => {
    if (!desktop) return;
    const before = await getIntegrationState();
    const flags = loadChoices ? await invoke<[boolean, boolean]>("integration_get_features") : null;
    const result = before.auth.account
      ? await invoke<EngineEnvelope<SyncStatus>>("cloud_sync_status")
      : null;
    const after = await getIntegrationState();
    if (!live.current) return;
    if (
      before.generation !== after.generation ||
      (result &&
        (result.generation !== after.generation ||
          result.account_subject !== after.auth.account?.subject))
    ) {
      setStatus(null);
      setSession(after);
      return;
    }
    setSession(after);
    setStatus(result?.data ?? null);
    if (flags) {
      setCloud(flags[0]);
      setAudit(flags[1]);
    }
  }, []);
  useEffect(() => {
    live.current = true;
    void Promise.resolve()
      .then(() => load(true))
      .catch((e) => {
        if (live.current) setError(managementError(e, "consultar a sincronização"));
      });
    const unlisten = desktop
      ? listen("monitoring:changed", () => {
          if (!locked.current) void load().catch(() => undefined);
        })
      : Promise.resolve(() => undefined);
    return () => {
      live.current = false;
      void unlisten.then((stop) => stop());
    };
  }, [load]);
  async function perform(operation: () => Promise<unknown>) {
    if (locked.current) return;
    locked.current = true;
    setBusy(true);
    setError("");
    try {
      await operation();
      await load(true);
      if (live.current) onChanged();
    } catch (e) {
      if (live.current)
        setError(
          managementError(e, "sincronizar configurações; os dados locais foram preservados"),
        );
    } finally {
      locked.current = false;
      if (live.current) setBusy(false);
    }
  }
  const granted = session?.auth.granted_scopes ?? [];
  const needsConsent =
    (cloud && !granted.includes(cloudScope)) || (audit && !granted.includes(auditScope));
  return (
    <Card>
      <CardHeader>
        <CardTitle>Configurações na conta Google</CardTitle>
      </CardHeader>
      <CardContent className="op-stack">
        <p>
          Sincronização opcional na pasta privada do aplicativo no Drive. Use a mesma conta e o
          mesmo aplicativo OAuth em outro computador. Planilhas, abas, proteção e referências
          aprovadas são recuperadas; credenciais, logs e inicialização do Windows não são enviados.
        </p>
        <label className="op-check">
          <input
            type="checkbox"
            checked={cloud}
            disabled={!desktop || busy}
            onChange={(e) => setCloud(e.target.checked)}
          />
          Sincronizar configurações entre dispositivos
        </label>
        <label className="op-check">
          <input
            type="checkbox"
            checked={audit}
            disabled={!desktop || busy}
            onChange={(e) => setAudit(e.target.checked)}
          />
          Consultar auditoria Google Workspace (requer privilégio administrativo)
        </label>
        <p>
          Novas permissões exigem consentimento no navegador. A auditoria do arquivo não comprova a
          autoria de uma alteração específica.
        </p>
        <Button
          disabled={!desktop || busy || !session?.configured}
          onClick={() =>
            void perform(async () => {
              await invoke("integration_set_features", {
                cloud,
                audit,
                expectedGeneration: session?.generation,
              });
              if (needsConsent) await connectIntegration();
            })
          }
        >
          {busy
            ? "Processando…"
            : needsConsent
              ? "Salvar e autorizar no Google"
              : "Salvar opções de nuvem"}
        </Button>
        <Button
          variant="outline"
          disabled={!desktop || busy || !cloud || !granted.includes(cloudScope)}
          onClick={() =>
            void perform(async () => {
              await invoke("cloud_sync_now", {
                expectedGeneration: session?.generation,
                resolution: null,
                expectedConflictToken: null,
              });
            })
          }
        >
          Sincronizar agora
        </Button>
        {status && (
          <div role="status">
            <p>{status.message}</p>
            {status.last_synced_at && (
              <p>Última sincronização: {formatLocalDateTime(status.last_synced_at)}</p>
            )}
          </div>
        )}
        {status && status.conflicts.length > 0 && (
          <section aria-label="Conflitos de sincronização">
            <p>
              Há alterações concorrentes. Nada será substituído automaticamente. A escolha mantém
              cópias das versões locais e remotas para recuperação.
            </p>
            <Button
              disabled={busy || !session || !status.conflict_token}
              onClick={() =>
                void perform(async () => {
                  await invoke("cloud_sync_now", {
                    expectedGeneration: session?.generation,
                    resolution: "keep_local",
                    expectedConflictToken: status.conflict_token,
                  });
                })
              }
            >
              Manter configuração local
            </Button>
            {status.conflicts.map((conflict) => (
              <Button
                key={conflict.commit_id}
                variant="outline"
                disabled={busy || !session || !status.conflict_token}
                onClick={() =>
                  void perform(async () => {
                    await invoke("cloud_sync_now", {
                      expectedGeneration: session?.generation,
                      resolution: `remote:${conflict.commit_id}`,
                      expectedConflictToken: status.conflict_token,
                    });
                  })
                }
              >
                Usar versão remota de {formatLocalDateTime(conflict.created_at)} (
                {conflict.commit_id.slice(0, 8)})
              </Button>
            ))}
          </section>
        )}
        {error && (
          <p className="feedback error" role="alert">
            {error}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
