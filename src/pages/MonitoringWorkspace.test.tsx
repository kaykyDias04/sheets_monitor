import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import userEvent from "@testing-library/user-event";
import { MonitoringWorkspace } from "./MonitoringWorkspace";
import type { IntegrationState } from "@/types/google";
import type { NotificationRecord } from "@/types/monitoring";

const mocks = vi.hoisted(() => ({
  state: vi.fn(),
  status: vi.fn(),
  checkNow: vi.fn(),
  history: vi.fn(),
  native: vi.fn(),
  readiness: vi.fn(),
  retry: vi.fn(),
  byId: vi.fn(),
  updaterStatus: vi.fn(),
  updaterCheck: vi.fn(),
  updaterDismiss: vi.fn(),
  updaterApply: vi.fn(),
  changed: null as (() => void) | null,
  open: null as ((id: string) => void) | null,
  updaterChanged: null as ((s: unknown) => void) | null,
  off: vi.fn(),
}));
vi.mock("@/services/platform", () => ({ desktop: true }));
vi.mock("@/services/google", async (original) => ({
  ...(await original<Record<string, unknown>>()),
  getIntegrationState: mocks.state,
}));
vi.mock("@/services/monitoring", () => ({
  getMonitoringStatus: mocks.status,
  checkNow: mocks.checkNow,
  notificationHistory: mocks.history,
  getDesktopStatus: mocks.native,
  getNotificationReadiness: mocks.readiness,
  retryBlockedNotifications: mocks.retry,
  notificationById: mocks.byId,
  getUpdaterStatus: mocks.updaterStatus,
  checkForUpdates: mocks.updaterCheck,
  dismissUpdate: mocks.updaterDismiss,
  applyUpdateAndRestart: mocks.updaterApply,
  onMonitoringChanged: vi.fn(async (callback: () => void) => {
    mocks.changed = callback;
    return mocks.off;
  }),
  onDesktopNavigate: vi.fn(async () => mocks.off),
  onNotificationOpen: vi.fn(async (callback: (id: string) => void) => {
    mocks.open = callback;
    return mocks.off;
  }),
  onUpdaterStateChanged: vi.fn(async (callback: (s: unknown) => void) => {
    mocks.updaterChanged = callback;
    return mocks.off;
  }),
}));
const original: IntegrationState = {
  generation: 1,
  configured: true,
  client_id: null,
  drive_metadata_opt_in: false,
  auth: {
    status: "connected",
    account: { subject: "fixture_account", name: null, email: null },
    granted_scopes: [],
    expires_at: null,
  },
};
let session = original;
const notification: NotificationRecord = {
  id: "opaque-notice",
  event_id: "fixture-event",
  source: "operational",
  spreadsheet_id: "fixture-file",
  spreadsheet_name: "Private synthetic name",
  sheet_id: null,
  sheet_title: null,
  severity: "warning",
  diagnosis: "Synthetic connection failure",
  diagnostic_code: "network",
  detected_at: "2026-11-02T12:00:00Z",
  state: "dispatch_submitted",
  attempt_count: 1,
  last_attempt_ms: 1,
  submitted_ms: 1,
  group_id: "group",
};
function envelope<T>(data: T, account = session) {
  return { generation: account.generation, account_subject: account.auth.account?.subject, data };
}
function view() {
  return (
    <MonitoringWorkspace
      key={session.generation}
      session={session}
      rows={[]}
      route="activities"
      disabled={false}
      onChanged={vi.fn()}
      onNavigate={vi.fn()}
      onOpenSheet={vi.fn()}
    />
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  session = original;
  mocks.retry.mockImplementation(async () => envelope(1));
  mocks.checkNow.mockImplementation(async () => envelope(2));
  mocks.updaterStatus.mockResolvedValue({
    state: "up_to_date",
    current_version: "1.0.5",
    available_version: null,
    release_url: null,
    release_notes: null,
    published_at: null,
    install_mode: "per_user_nsis",
    can_auto_update: true,
    dismissed: false,
    last_checked_at: "2026-11-04T12:00:00Z",
    last_error_message: null,
  });
  mocks.updaterDismiss.mockImplementation(async (version: string) => ({
    state: "idle",
    current_version: "1.0.5",
    available_version: version,
    release_url: "https://github.com/kaykyDias04/sheets_monitor/releases/tag/v1.0.6",
    release_notes: null,
    published_at: null,
    install_mode: "per_user_nsis",
    can_auto_update: true,
    dismissed: true,
    last_checked_at: "2026-11-04T12:00:00Z",
    last_error_message: null,
  }));
  mocks.updaterApply.mockResolvedValue(undefined);
  mocks.readiness.mockResolvedValue({
    identity_mode: "development_powershell",
    setting: "enabled",
    error_code: null,
  });
  mocks.state.mockImplementation(async () => session);
  mocks.native.mockResolvedValue({
    tray_available: true,
    startup: { registered: false, managed: false, error_code: null, os_execution: "unknown" },
    exiting: false,
    notification_capability: "desktop_unverified",
  });
  mocks.status.mockImplementation(async () =>
    envelope({
      control: { enabled: true, generation: 1 },
      schedules: [],
      eligible_count: 0,
      interval_seconds: 60,
      service: { running: true, active: 0, queued: 0, warning: null },
    }),
  );
  mocks.history.mockImplementation(async () =>
    envelope({ items: [], total: 0, offset: 0, limit: 25 }),
  );
});
afterEach(cleanup);
it("explains a Windows notification block without changing startup or stopping monitoring", async () => {
  mocks.readiness.mockResolvedValue({
    identity_mode: "development_powershell",
    setting: "disabled_for_user",
    error_code: null,
  });
  render(
    <MonitoringWorkspace
      session={session}
      rows={[]}
      route="settings"
      disabled={false}
      onChanged={vi.fn()}
      onNavigate={vi.fn()}
      onOpenSheet={vi.fn()}
    />,
  );
  expect(
    await screen.findByText(/As notificações estão desativadas no Windows para este usuário/),
  ).toBeInTheDocument();
  expect(screen.getByText(/os alertas usam o remetente Windows PowerShell/)).toBeInTheDocument();
  expect(screen.getByRole("checkbox", { name: "Registrar início com Windows" })).not.toBeChecked();
  expect(screen.getByRole("button", { name: "Reenviar alertas bloqueados" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Pausar monitoramento" })).toBeInTheDocument();
});
it("retries blocked alerts only after an explicit action in the current generation", async () => {
  render(
    <MonitoringWorkspace
      session={session}
      rows={[]}
      route="settings"
      disabled={false}
      onChanged={vi.fn()}
      onNavigate={vi.fn()}
      onOpenSheet={vi.fn()}
    />,
  );
  const button = await screen.findByRole("button", { name: "Reenviar alertas bloqueados" });
  await waitFor(() => expect(button).toBeEnabled());
  expect(mocks.retry).not.toHaveBeenCalled();
  await userEvent.click(button);
  await waitFor(() => expect(mocks.retry).toHaveBeenCalledWith(1));
});
it("refreshes from a Tauri signal and resolves the exact opaque notification in the current session", async () => {
  render(view());
  await waitFor(() => expect(mocks.status).toHaveBeenCalled());
  const before = mocks.status.mock.calls.length;
  act(() => mocks.changed?.());
  await waitFor(() => expect(mocks.status.mock.calls.length).toBeGreaterThan(before));
  mocks.byId.mockResolvedValue(envelope(notification));
  act(() => mocks.open?.("opaque-notice"));
  expect(await screen.findByRole("dialog")).toBeInTheDocument();
  expect(screen.getByText(notification.diagnosis)).toBeInTheDocument();
  expect(mocks.byId).toHaveBeenCalledWith("opaque-notice");
});
it("discards an old account's pending notification after the session changes", async () => {
  const rendered = render(view());
  await waitFor(() => expect(mocks.status).toHaveBeenCalled());
  let finish!: (value: unknown) => void;
  mocks.byId.mockReturnValue(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  act(() => mocks.open?.("opaque-old-account"));
  session = {
    ...original,
    generation: 2,
    auth: { ...original.auth, account: { subject: "other_account", email: null, name: null } },
  };
  rendered.rerender(view());
  await act(async () => {
    finish(envelope(notification, original));
  });
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  expect(screen.queryByText(notification.spreadsheet_name)).not.toBeInTheDocument();
  expect(mocks.off).toHaveBeenCalledTimes(4);
});
it("enfileira verificações elegíveis silenciosamente sem exibir banner solto", async () => {
  mocks.status.mockImplementation(async () =>
    envelope({
      control: { enabled: true, generation: 1 },
      schedules: [],
      eligible_count: 2,
      interval_seconds: 60,
      service: { running: true, active: 0, queued: 0, warning: null },
    }),
  );
  render(
    <MonitoringWorkspace
      session={session}
      rows={[]}
      route="overview"
      disabled={false}
      onChanged={vi.fn()}
      onNavigate={vi.fn()}
      onOpenSheet={vi.fn()}
    />,
  );
  const btn = await screen.findByRole("button", { name: "Verificar elegíveis agora" });
  await waitFor(() => expect(btn).toBeEnabled());
  await userEvent.click(btn);
  await waitFor(() => expect(mocks.checkNow).toHaveBeenCalledWith(1));
  expect(screen.queryByText(/Verificações elegíveis adicionadas à fila/i)).not.toBeInTheDocument();
});
it("solicita autorização explícita para reiniciar e atualizar e permite adiar o aviso", async () => {
  mocks.updaterStatus.mockResolvedValue({
    state: "ready_to_install",
    current_version: "1.0.5",
    available_version: "1.0.6",
    release_url: "https://github.com/kaykyDias04/sheets_monitor/releases/tag/v1.0.6",
    release_notes: "Correções de estabilidade",
    published_at: "2026-11-04T12:00:00Z",
    install_mode: "per_user_nsis",
    can_auto_update: true,
    dismissed: false,
    last_checked_at: "2026-11-04T12:00:00Z",
    last_error_message: null,
  });
  render(
    <MonitoringWorkspace
      session={session}
      rows={[]}
      route="overview"
      disabled={false}
      onChanged={vi.fn()}
      onNavigate={vi.fn()}
      onOpenSheet={vi.fn()}
    />,
  );
  expect(
    await screen.findByText(/Nova versão estável disponível \(v1\.0\.6\)/),
  ).toBeInTheDocument();
  expect(mocks.updaterApply).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", { name: "Agora não" }));
  await waitFor(() => expect(mocks.updaterDismiss).toHaveBeenCalledWith("1.0.6"));
  await waitFor(() =>
    expect(
      screen.queryByText(/Nova versão estável disponível \(v1\.0\.6\)/),
    ).not.toBeInTheDocument(),
  );
});
