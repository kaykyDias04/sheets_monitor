import { useState } from "react";
import { LogIn, LogOut, ShieldCheck, RefreshCw, Save } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { Preferences } from "@/types/management";
import type { IntegrationState } from "@/types/google";
export function AccountCard({
  session,
  disabled,
  onConnect,
  onDisconnect,
  onDrive,
  onRefresh,
}: {
  session: IntegrationState | null;
  disabled: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onDrive: (enabled: boolean) => void;
  onRefresh: () => void;
}) {
  const connected = session?.auth.status === "connected";
  return (
    <Card>
      <CardHeader>
        <CardTitle>Conta Google</CardTitle>
        <CardDescription>
          Entre pelo navegador do sistema. O aplicativo consulta planilhas e scripts sem editar seus
          arquivos.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="op-account">
          <span className="op-account-icon">
            <ShieldCheck size={25} />
          </span>
          <div>
            <strong>
              {session?.auth.account?.name ??
                (connected ? "Conta Google conectada" : "Conecte sua conta Google")}
            </strong>
            <p>
              {session?.auth.account?.email ??
                (session?.auth.status === "reconnect_required"
                  ? "Uma nova autorização é necessária."
                  : "Suas credenciais ficam no armazenamento seguro do Windows.")}
            </p>
          </div>
        </div>
        {session && !session.configured && (
          <p className="op-notice op-warning">
            Esta instalação ainda não foi provisionada com a configuração OAuth da aplicação. O
            mantenedor precisa preparar o cliente Desktop para distribuir o aplicativo; o usuário
            final não deve criar projetos Cloud nem informar chaves.
          </p>
        )}
        <div className="op-actions">
          <Button disabled={disabled || !session?.configured} onClick={onConnect}>
            <LogIn />
            {connected || session?.auth.status === "reconnect_required"
              ? "Reautorizar permissões"
              : "Entrar com Google"}
          </Button>
          <Button
            variant="outline"
            disabled={disabled || !session?.auth.account}
            onClick={onDisconnect}
          >
            <LogOut />
            Desconectar conta
          </Button>
          <Button variant="ghost" disabled={disabled} onClick={onRefresh}>
            <RefreshCw />
            Atualizar conexão
          </Button>
        </div>
        <div className="op-drive-optin">
          <label className="op-check">
            <input
              type="checkbox"
              checked={session?.drive_metadata_opt_in ?? false}
              disabled={disabled || !session?.configured}
              onChange={(e) => onDrive(e.target.checked)}
            />
            Permitir descoberta de planilhas pelo Drive
          </label>
          <p className="op-hint">
            A descoberta usa leitura de metadados dos arquivos acessíveis. Ativar pode exigir novo
            consentimento; nenhuma permissão de escrita é solicitada.
          </p>
        </div>
        <details className="op-advanced">
          <summary>Permissões e estado das APIs</summary>
          <ul className="op-api-status">
            {[
              ["Sheets", "https://www.googleapis.com/auth/spreadsheets.readonly"],
              ["Drive", "https://www.googleapis.com/auth/drive.metadata.readonly"],
              ["Apps Script", "https://www.googleapis.com/auth/script.projects.readonly"],
            ].map(([label, scope]) => (
              <li key={label}>
                <strong>{label}</strong>
                <span>
                  {session?.auth.granted_scopes.includes(scope)
                    ? "Permissão concedida; a disponibilidade é verificada nas consultas"
                    : "Permissão não confirmada"}
                </span>
              </li>
            ))}
          </ul>
        </details>
        <p className="op-hint">
          Desconectar encerra a sessão local; não revoga a concessão no Google. A conexão com
          usuários de teste não comprova publicação para contas externas.
        </p>
      </CardContent>
    </Card>
  );
}
export function PreferenceForm({
  value,
  disabled,
  onSave,
}: {
  value: Preferences;
  disabled: boolean;
  onSave: (value: Preferences) => void;
}) {
  const [preferences, setPreferences] = useState(value);
  return (
    <Card>
      <CardHeader>
        <CardTitle>Preferências do aplicativo</CardTitle>
        <CardDescription>
          Salvas por conta no SQLite. O início automático com Windows é configurado separadamente
          neste dispositivo.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="op-stack"
          onSubmit={(e) => {
            e.preventDefault();
            onSave(preferences);
          }}
        >
          <fieldset className="op-preferences-fields" disabled={disabled}>
            <legend className="sr-only">Preferências persistidas</legend>
            <div className="op-form-grid">
              <label className="op-field">
                Intervalo de verificação (segundos)
                <Input
                  type="number"
                  min={60}
                  max={300}
                  required
                  value={preferences.interval_seconds}
                  onChange={(e) =>
                    setPreferences({ ...preferences, interval_seconds: Number(e.target.value) })
                  }
                />
                <small>Aplicado pelo serviço Rust enquanto o monitoramento estiver ativo.</small>
              </label>
              <label className="op-field">
                Política padrão de histórico (dias)
                <Input
                  type="number"
                  min={1}
                  max={3650}
                  required
                  value={preferences.history_days}
                  onChange={(e) =>
                    setPreferences({ ...preferences, history_days: Number(e.target.value) })
                  }
                />
                <small>A retenção de cada proteção é configurada nas regras.</small>
              </label>
              <label className="op-field">
                Aparência
                <select
                  value={preferences.theme}
                  onChange={(e) =>
                    setPreferences({
                      ...preferences,
                      theme: e.target.value as Preferences["theme"],
                    })
                  }
                >
                  <option value="system">Seguir o Windows</option>
                  <option value="light">Clara</option>
                  <option value="dark">Escura</option>
                </select>
              </label>
            </div>
            <label className="op-check">
              <input
                type="checkbox"
                checked={preferences.notifications_enabled}
                onChange={(e) =>
                  setPreferences({ ...preferences, notifications_enabled: e.target.checked })
                }
              />
              Enviar notificações de alterações e falhas
            </label>
            <p className="op-hint">
              Alertas desativados ficam registrados como suprimidos, sem envio retroativo ao ativar.
            </p>
            <label className="op-check">
              <input
                type="checkbox"
                checked={preferences.run_minimized}
                onChange={(event) =>
                  setPreferences({ ...preferences, run_minimized: event.target.checked })
                }
              />
              Ocultar a janela ao iniciar com Windows, se a bandeja estiver disponível
            </label>
          </fieldset>
          <Button type="submit" disabled={disabled}>
            <Save />
            Salvar preferências
          </Button>
        </form>
        <p className="op-hint">
          Sheets Monitor 0.1.0 · Windows 11 · Dados e referências locais. As operações Google exigem
          internet.
        </p>
      </CardContent>
    </Card>
  );
}
