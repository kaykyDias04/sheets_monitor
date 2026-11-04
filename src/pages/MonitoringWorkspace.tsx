import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { desktop } from "@/services/platform";
import { getIntegrationState } from "@/services/google";
import { managementError } from "@/services/management";
import * as api from "@/services/monitoring";
import type { IntegrationState } from "@/types/google";
import type { Page, RegisteredSpreadsheet } from "@/types/management";
import type {
  DesktopStatus,
  MonitoringStatus,
  NotificationRecord,
  NativeReadiness,
  UpdaterStatus,
} from "@/types/monitoring";
import { MonitoringPanel, NotificationHistory } from "./MonitoringPanel";
import { diagnosticText, formatLocalDateTime } from "./OperationalShared";

/** Event-driven view of the Rust service; no frontend scheduling timer. */
export function MonitoringWorkspace({
  session,
  rows,
  route,
  disabled,
  suppressedError = "",
  onChanged,
  onNavigate,
  onOpenSheet,
}: {
  session: IntegrationState | null;
  rows: RegisteredSpreadsheet[];
  route: string;
  disabled: boolean;
  suppressedError?: string;
  onChanged: () => void;
  onNavigate: (route: "settings" | "overview") => void;
  onOpenSheet: (id: string) => void;
}) {
  const [status, setStatus] = useState<MonitoringStatus | null>(null);
  const [native, setNative] = useState<DesktopStatus | null>(null);
  const [readiness, setReadiness] = useState<NativeReadiness | null>(null);
  const [updater, setUpdater] = useState<UpdaterStatus | null>(null);
  const [history, setHistory] = useState<Page<NotificationRecord> | null>(null);
  const [selected, setSelected] = useState<NotificationRecord | null>(null);
  const [offset, setOffset] = useState(0);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState<{ route: string; text: string } | null>(null);
  const live = useRef({ session, route, offset, onChanged, onNavigate });
  useLayoutEffect(() => {
    live.current = { session, route, offset, onChanged, onNavigate };
  }, [session, route, offset, onChanged, onNavigate]);
  const mounted = useRef(false);
  const loading = useRef(false);
  const pending = useRef(false);
  const same = (before: IntegrationState, after: IntegrationState) =>
    before.generation === after.generation &&
    before.auth.account?.subject === after.auth.account?.subject;
  const load = useCallback(async () => {
    if (!desktop || !mounted.current) return;
    if (loading.current) {
      pending.current = true;
      return;
    }
    loading.current = true;
    try {
      do {
        pending.current = false;
        const before = live.current.session;
        const [nativeStatus, notificationReadiness, updaterStatus] = await Promise.all([
          api.getDesktopStatus(),
          api.getNotificationReadiness(),
          api.getUpdaterStatus ? api.getUpdaterStatus().catch(() => null) : Promise.resolve(null),
        ]);
        if (mounted.current) {
          setNative(nativeStatus);
          setReadiness(notificationReadiness);
          if (updaterStatus) setUpdater(updaterStatus);
        }
        if (!before?.auth.account) continue;
        const [monitor, notifications] = await Promise.all([
          api.getMonitoringStatus(),
          api.notificationHistory(live.current.offset),
        ]);
        const after = await getIntegrationState();
        if (
          !mounted.current ||
          !same(before, after) ||
          !live.current.session ||
          !same(before, live.current.session)
        )
          continue;
        if (
          [monitor, notifications].some(
            (value) =>
              value.generation !== after.generation ||
              value.account_subject !== after.auth.account?.subject,
          )
        )
          continue;
        setStatus(monitor.data);
        setHistory(notifications.data);
        setError("");
      } while (pending.current && mounted.current);
    } catch (failure) {
      if (mounted.current)
        setError(managementError(failure, "consultar o serviço de monitoramento"));
    } finally {
      loading.current = false;
    }
  }, []);
  useEffect(() => {
    mounted.current = true;
    let subscribed = true;
    const unlisten: (() => void)[] = [];
    function retain(off: () => void) {
      if (subscribed && mounted.current) unlisten.push(off);
      else off();
    }
    if (desktop) {
      void api
        .onMonitoringChanged(() => {
          void load();
          live.current.onChanged();
        })
        .then(retain)
        .catch(() => undefined);
      void api
        .onDesktopNavigate((target) =>
          live.current.onNavigate(target === "settings" ? "settings" : "overview"),
        )
        .then(retain)
        .catch(() => undefined);
      void api
        .onNotificationOpen((id) => {
          const before = live.current.session;
          if (!before?.auth.account) return;
          void api
            .notificationById(id)
            .then(async (envelope) => {
              const after = await getIntegrationState();
              if (
                mounted.current &&
                same(before, after) &&
                live.current.session &&
                same(before, live.current.session) &&
                envelope.generation === after.generation &&
                envelope.account_subject === after.auth.account?.subject
              ) {
                setSelected(envelope.data);
                if (!envelope.data)
                  setNotice({
                    route: live.current.route,
                    text: "O evento não está disponível nesta conta ou já expirou.",
                  });
              }
            })
            .catch((failure) => {
              if (mounted.current)
                setError(managementError(failure, "abrir o evento da notificação"));
            });
        })
        .then(retain)
        .catch(() => undefined);
      if (api.onUpdaterStateChanged) {
        void api
          .onUpdaterStateChanged((nextUpdater) => {
            if (mounted.current) setUpdater(nextUpdater);
          })
          .then(retain)
          .catch(() => undefined);
      }
    }
    return () => {
      mounted.current = false;
      subscribed = false;
      unlisten.forEach((off) => off());
    };
  }, [load]);
  useEffect(() => {
    void Promise.resolve().then(load);
  }, [route, offset, load]);
  async function action(
    work: () => Promise<unknown>,
    message: string | undefined,
    context: string,
  ) {
    if (working || disabled) return;
    setWorking(true);
    setError("");
    setNotice(null);
    try {
      await work();
      await load();
      setNotice(message ? { route: live.current.route, text: message } : null);
      live.current.onChanged();
    } catch (failure) {
      setError(managementError(failure, context));
    } finally {
      if (mounted.current) setWorking(false);
    }
  }
  async function updaterAction(work: () => Promise<UpdaterStatus>, context: string) {
    if (working || disabled) return;
    setWorking(true);
    setError("");
    try {
      const next = await work();
      if (mounted.current) setUpdater(next);
    } catch (failure) {
      if (mounted.current) setError(managementError(failure, context));
    } finally {
      if (mounted.current) setWorking(false);
    }
  }
  const blocked = disabled || working;
  const generation = session?.generation;
  const visibleError = error && error !== suppressedError ? error : "";
  const visibleNotice = notice?.route === route ? notice.text : "";
  const updatePromptVisible = Boolean(
    updater &&
    !updater.dismissed &&
    (updater.state === "ready_to_install" || updater.state === "manual_required") &&
    (route === "overview" || route === "settings"),
  );
  const hasVisibleBlocks = Boolean(
    visibleError ||
    visibleNotice ||
    updatePromptVisible ||
    route === "settings" ||
    session?.auth.account,
  );
  return (
    <>
      {hasVisibleBlocks && (
        <div className="op-stack op-monitoring-workspace">
          {visibleError && (
            <p className="feedback error" role="alert">
              {visibleError}
            </p>
          )}
          {visibleNotice && (
            <p className="feedback success" role="status">
              {visibleNotice}
            </p>
          )}
          {updatePromptVisible && updater && (
            <Card className="op-update-banner">
              <CardHeader>
                <CardTitle>Nova versão estável disponível (v{updater.available_version})</CardTitle>
                <CardDescription>
                  {updater.state === "ready_to_install"
                    ? "O pacote compatível com Windows foi baixado do GitHub Releases e validado com SHA-256. Suas configurações, credenciais e planilhas cadastradas serão preservadas."
                    : updater.install_mode === "per_machine_nsis"
                      ? "Esta instalação administrativa exige atualização manual pelo instalador oficial para não solicitar elevação automática."
                      : "Esta execução isolada ou de desenvolvimento não sobrescreve o executável automaticamente. Utilize o instalador oficial da release."}
                </CardDescription>
              </CardHeader>
              <CardContent className="op-actions">
                {updater.state === "ready_to_install" && updater.can_auto_update && (
                  <Button
                    disabled={blocked}
                    onClick={() =>
                      void action(
                        () => api.applyUpdateAndRestart(),
                        "Reiniciando para concluir a atualização…",
                        "aplicar a atualização e reiniciar o aplicativo",
                      )
                    }
                  >
                    Reiniciar e atualizar
                  </Button>
                )}
                {updater.release_url && updater.state === "manual_required" && (
                  <a
                    className="op-release-link"
                    href={updater.release_url}
                    target="_blank"
                    rel="noreferrer"
                  >
                    Ver página da versão v{updater.available_version}
                  </a>
                )}
                {updater.available_version && (
                  <Button
                    variant="outline"
                    disabled={blocked}
                    onClick={() => {
                      const targetVersion = updater.available_version;
                      if (targetVersion)
                        void updaterAction(
                          () => api.dismissUpdate(targetVersion),
                          "adiar o aviso de atualização",
                        );
                    }}
                  >
                    Agora não
                  </Button>
                )}
              </CardContent>
            </Card>
          )}
          {session?.auth.account && (route === "overview" || route === "settings") && (
            <MonitoringPanel
              status={status}
              rows={rows}
              disabled={blocked}
              onToggle={(enabled) => {
                if (generation !== undefined)
                  void action(
                    () => api.setMonitoringEnabled(enabled, generation),
                    enabled
                      ? "Monitoramento iniciado."
                      : "Monitoramento pausado; verificações automáticas canceladas.",
                    enabled ? "iniciar o monitoramento" : "pausar o monitoramento",
                  );
              }}
              onCheck={() => {
                if (generation !== undefined)
                  void action(
                    () => api.checkNow(generation),
                    undefined,
                    "agendar a verificação das planilhas elegíveis",
                  );
              }}
            />
          )}
          {session?.auth.account && route !== "overview" && route !== "settings" && (
            <p className="op-hint" role="status">
              Monitoramento {status?.control.enabled ? "ativo" : "pausado"} ·{" "}
              {status?.eligible_count ?? 0} planilhas prontas
            </p>
          )}
          {route === "settings" && (
            <Card>
              <CardHeader>
                <CardTitle>Windows e segundo plano</CardTitle>
                <CardDescription>
                  Configuração deste usuário Windows, independente da conta Google.
                </CardDescription>
              </CardHeader>
              <CardContent className="op-stack">
                <p>
                  {native?.tray_available
                    ? "Bandeja disponível. Fechar a janela mantém o aplicativo em execução."
                    : "Bandeja indisponível. A janela permanecerá acessível ao tentar fechar."}
                </p>
                <label className="op-check">
                  <input
                    type="checkbox"
                    checked={native?.startup.registered ?? false}
                    disabled={
                      blocked || !native || (!!native.startup.error_code && !native.startup.managed)
                    }
                    onChange={(event) => {
                      const enabled = event.target.checked;
                      void action(
                        () => api.setStartup(enabled),
                        enabled
                          ? "Registro de inicialização ativado para este usuário Windows."
                          : "Registro de inicialização desativado.",
                        "alterar a inicialização com o Windows",
                      );
                    }}
                  />
                  Registrar início com Windows
                </label>
                <p className="op-hint">
                  Registro {native?.startup.registered ? "ativo" : "desativado"}. A execução no
                  login também depende das configurações de inicialização do Windows.
                </p>
                {native?.startup.error_code && (
                  <p className="op-hint">
                    Não foi possível confirmar ou alterar o registro de inicialização com segurança.
                    Nenhuma entrada conflitante foi sobrescrita.
                  </p>
                )}
                <p className="op-hint">
                  O Windows pode silenciar alertas nas configurações de notificações ou em Não
                  incomodar. O retorno da API nativa não confirma que um alerta foi exibido.
                </p>
                {readiness?.identity_mode === "development_powershell" && (
                  <p className="op-hint">
                    Neste build de desenvolvimento, os alertas usam o remetente Windows PowerShell.
                    A identidade instalada será validada na distribuição.
                  </p>
                )}
                {readiness && readiness.setting !== "enabled" && (
                  <p className="feedback error" role="status">
                    {readiness.setting === "disabled_for_user"
                      ? "As notificações estão desativadas no Windows para este usuário."
                      : readiness.setting === "disabled_for_application"
                        ? "As notificações deste remetente estão desativadas no Windows."
                        : readiness.setting === "disabled_by_group_policy"
                          ? "Uma política do Windows bloqueia as notificações."
                          : readiness.setting === "disabled_by_manifest"
                            ? "A identidade de notificações ainda não está habilitada neste build."
                            : "Não foi possível confirmar a disponibilidade das notificações no Windows."}{" "}
                    O monitoramento continua; revise as configurações do Windows quando desejar.
                  </p>
                )}
                <div className="op-actions">
                  <Button
                    variant="outline"
                    disabled={
                      blocked ||
                      !session?.auth.account ||
                      !readiness ||
                      readiness.setting.startsWith("disabled")
                    }
                    onClick={() => {
                      if (generation !== undefined)
                        void action(
                          () => api.retryBlockedNotifications(generation),
                          "Reenvio solicitado para alertas bloqueados ainda elegíveis. Itens suprimidos ou expirados permanecem encerrados.",
                          "reenviar os alertas bloqueados",
                        );
                    }}
                  >
                    Reenviar alertas bloqueados
                  </Button>
                  <Button
                    variant="outline"
                    disabled={blocked || !native}
                    onClick={() =>
                      void action(
                        api.exitDesktop,
                        "Encerrando aplicativo…",
                        "encerrar o aplicativo",
                      )
                    }
                  >
                    Sair do aplicativo
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
          {route === "settings" && updater && (
            <Card>
              <CardHeader>
                <CardTitle>Atualizações do aplicativo</CardTitle>
                <CardDescription>
                  Verificação discreta via GitHub Releases estáveis com validação de origem e
                  integridade SHA-256.
                </CardDescription>
              </CardHeader>
              <CardContent className="op-stack">
                <p>
                  Versão instalada: <strong>v{updater.current_version}</strong> ·{" "}
                  {updater.install_mode === "per_user_nsis"
                    ? "Instalação por usuário (atualização automática compatível)"
                    : updater.install_mode === "per_machine_nsis"
                      ? "Instalação por máquina (requer instalador manual)"
                      : "Execução isolada ou de desenvolvimento (atualização manual)"}
                </p>
                <p className="op-hint">
                  Última verificação:{" "}
                  {updater.last_checked_at
                    ? formatLocalDateTime(updater.last_checked_at)
                    : "Ainda não realizada nesta sessão"}
                  {updater.state === "up_to_date"
                    ? " · Nenhuma versão estável mais recente encontrada."
                    : updater.state === "checking"
                      ? " · Consultando GitHub Releases…"
                      : updater.state === "downloading"
                        ? " · Baixando e validando instalador…"
                        : ""}
                </p>
                {updater.last_error_message && (
                  <p className="op-hint" role="status">
                    Última tentativa de atualização: {updater.last_error_message}
                  </p>
                )}
                <div className="op-actions">
                  <Button
                    variant="outline"
                    disabled={
                      blocked ||
                      updater.state === "checking" ||
                      updater.state === "downloading" ||
                      updater.state === "installing"
                    }
                    onClick={() =>
                      void updaterAction(
                        () => api.checkForUpdates(),
                        "verificar atualizações do aplicativo",
                      )
                    }
                  >
                    Verificar atualizações
                  </Button>
                  {updater.state === "ready_to_install" && updater.can_auto_update && (
                    <Button
                      disabled={blocked}
                      onClick={() =>
                        void action(
                          () => api.applyUpdateAndRestart(),
                          "Reiniciando para concluir a atualização…",
                          "aplicar a atualização e reiniciar o aplicativo",
                        )
                      }
                    >
                      Reiniciar e atualizar (v{updater.available_version})
                    </Button>
                  )}
                </div>
              </CardContent>
            </Card>
          )}
          {session?.auth.account && (route === "activities" || route === "settings") && (
            <NotificationHistory
              page={history}
              disabled={blocked}
              onPage={setOffset}
              onOpen={setSelected}
            />
          )}
        </div>
      )}
      <Dialog
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open) setSelected(null);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{selected?.event_type_label ?? "Evento da notificação"}</DialogTitle>
            <DialogDescription>
              {selected?.spreadsheet_name}
              {selected?.sheet_title ? ` · ${selected.sheet_title}` : ""} ·{" "}
              {selected
                ? selected.formatted_detected_at || formatLocalDateTime(selected.detected_at)
                : ""}
            </DialogDescription>
          </DialogHeader>
          {selected && (
            <>
              <p>{diagnosticText(selected.diagnosis)}</p>
              <p className="op-hint">
                Origem: {selected.source === "integrity" ? "Integridade" : "Operação"} ·{" "}
                {selected.sheet_title ??
                  (selected.sheet_id !== null ? `Aba ${selected.sheet_id}` : "Arquivo")}
              </p>
              <p className="op-hint">
                {selected.attempt_count} tentativas registradas. A submissão nativa não comprova
                exibição ou leitura.
              </p>
              <Button
                onClick={() => {
                  const id = selected.spreadsheet_id;
                  setSelected(null);
                  onOpenSheet(id);
                }}
              >
                Ver em Planilhas
              </Button>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
