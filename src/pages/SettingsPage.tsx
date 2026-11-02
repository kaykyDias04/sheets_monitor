import type { FormEvent } from "react";

import { Clock3, LockKeyhole, LoaderCircle, Check, ShieldCheck } from "lucide-react";

import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/components/ui/card";

import { Button } from "@/components/ui/button";

import { Input } from "@/components/ui/input";

import { Switch } from "@/components/ui/switch";

import { desktop } from "@/services/foundation";

import type { FoundationDashboard, AppSettings } from "@/types/foundation";

export function SettingsPage({
  data,

  settings,

  setSettings,

  busy,

  submitSettings,
}: {
  data: FoundationDashboard;

  settings: AppSettings;

  setSettings: (settings: AppSettings) => void;

  busy: boolean;

  submitSettings: (e: FormEvent) => Promise<void>;
}) {
  return (
    <div className="settings-grid">
      <Card>
        <CardHeader>
          <div className="settings-icon">
            <Clock3 size={21} />
          </div>

          <CardTitle>Preferências do workspace</CardTitle>

          <CardDescription>Valores persistidos localmente para as próximas etapas.</CardDescription>
        </CardHeader>

        <CardContent>
          <form onSubmit={(e) => void submitSettings(e)}>
            <label htmlFor="interval" className="field-label">
              Intervalo de verificação planejado
            </label>

            <div className="input-unit">
              <Input
                id="interval"

                type="number"

                min={60}

                max={300}

                step={1}

                required

                value={Number.isFinite(settings.interval_seconds) ? settings.interval_seconds : ""}

                onChange={(e) =>
                  setSettings({ ...settings, interval_seconds: e.target.valueAsNumber })
                }
              />

              <span>segundos</span>
            </div>

            <p className="field-hint">
              De 60 a 300 segundos. Padrão: 60. A coleta periódica será implementada em uma sprint
              futura.
            </p>

            <label htmlFor="retention" className="field-label">
              Histórico padrão
            </label>

            <div className="input-unit">
              <Input
                id="retention"

                value={settings.history_days}

                readOnly

                aria-describedby="retention-help"
              />

              <span>dias</span>
            </div>

            <p id="retention-help" className="field-hint">
              Retenção definida para a V1.0. Nesta etapa há apenas eventos sintéticos.
            </p>

            <div className="switch-setting">
              <div>
                <label htmlFor="autostart">Inicializar com Windows</label>

                <p>Disponível em uma etapa futura.</p>
              </div>

              <Switch
                id="autostart"

                checked={false}

                disabled

                aria-label="Inicializar com Windows · indisponível nesta sprint"
              />
            </div>

            <Button type="submit" disabled={busy}>
              {busy ? <LoaderCircle className="animate-spin" /> : <Check />}Salvar preferências
              locais
            </Button>
          </form>
        </CardContent>
      </Card>

      <div className="settings-side">
        <Card>
          <CardHeader>
            <div className="settings-icon">
              <LockKeyhole size={21} />
            </div>

            <CardTitle>Google e credenciais</CardTitle>

            <CardDescription>Configure a conta na tela Integração Google.</CardDescription>
          </CardHeader>

          <CardContent>
            <p className="settings-copy">
              OAuth com PKCE, cofre Windows e consultas manuais estão na área Integração Google.
              Estas preferências continuam locais e demonstrativas.
            </p>
          </CardContent>
        </Card>

        <Card className="foundation-card">
          <CardHeader>
            <CardTitle>Fundação em funcionamento</CardTitle>
          </CardHeader>

          <CardContent>
            <dl>
              <div>
                <dt>Interface</dt>

                <dd>React + shadcn/ui</dd>
              </div>

              <div>
                <dt>Persistência</dt>

                <dd>{desktop ? "SQLite local" : "Memória · prévia web"}</dd>
              </div>

              <div>
                <dt>Versão do banco</dt>

                <dd>{data.database_version}</dd>
              </div>

              <div>
                <dt>Dados desta sprint</dt>

                <dd>100% demonstrativos</dd>
              </div>
            </dl>
          </CardContent>
        </Card>
      </div>

      <div className="privacy-note">
        <ShieldCheck size={23} />

        <div>
          <h2>Workspace local, sem servidor próprio</h2>

          <p>
            Nesta tela, os dados fictícios validam a navegação e o SQLite. Monitoramento de arquivos
            reais, análise de Apps Script e notificações de produção aguardam suas sprints.
          </p>
        </div>
      </div>
    </div>
  );
}
