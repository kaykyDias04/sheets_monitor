import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { MonitoringWorkspace } from "./MonitoringWorkspace";
import type { IntegrationState } from "@/types/google";
import type { NotificationRecord } from "@/types/monitoring";

const mocks = vi.hoisted(() => ({
  state: vi.fn(),
  status: vi.fn(),
  history: vi.fn(),
  native: vi.fn(),
  byId: vi.fn(),
  changed: null as (() => void) | null,
  open: null as ((id: string) => void) | null,
  off: vi.fn(),
}));
vi.mock("@/services/platform", () => ({ desktop: true }));
vi.mock("@/services/google", () => ({ getIntegrationState: mocks.state }));
vi.mock("@/services/monitoring", () => ({
  getMonitoringStatus: mocks.status,
  notificationHistory: mocks.history,
  getDesktopStatus: mocks.native,
  notificationById: mocks.byId,
  onMonitoringChanged: vi.fn(async (callback: () => void) => {
    mocks.changed = callback;
    return mocks.off;
  }),
  onDesktopNavigate: vi.fn(async () => mocks.off),
  onNotificationOpen: vi.fn(async (callback: (id: string) => void) => {
    mocks.open = callback;
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
  expect(mocks.off).toHaveBeenCalledTimes(3);
});
