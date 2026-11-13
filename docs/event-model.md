# Modelo de diferenças, eventos e incidentes

Conteúdo e integridade são comparações distintas. O baseline aprovado define obrigações estruturais; a observação anterior define mudanças de conteúdo para diagnóstico interno. Uma edição permitida pode manter Online, mas não é uma atualização de proteção. Offline exige violação crítica comprovada.

Cada evento inclui eventId, checkId, spreadsheetId, sheetId opcional, categoria, tipo, severidade, regra opcional, localização, referências anterior/atual, data, fingerprint e diagnóstico. Referências de conteúdo são hashes, sem valores brutos. Cabeçalhos/fórmulas necessários aos diagnósticos são dados privados locais; não devem entrar em evidências públicas.

Com chave configurada e única, o comparador identifica registros adicionados/removidos e células alteradas por identidade lógica. Reordenação de registros é registrada sem transformar todo deslocamento após inserção em spam. Sem chave estável, usa alinhamento/sequência e hashes de linhas, indicando mudança não especificada quando não é possível provar se houve inserção, exclusão ou edição. Não inventa operações.

Incidentes estruturais têm identidade estável e fingerprint do estado. A→B cria ocorrência; B persistente não cria evento idêntico; B→A gera recuperação; A→B posterior cria nova ocorrência. Uma mudança no diagnóstico mantém a identidade da regra. Inconclusão atual não é prova de recuperação de uma violação anterior.

Coleta parcial: regiões completas e scripts obtidos atualizam um journal por componente. Falha em outro serviço não apaga uma diferença observada. O último snapshot global completo permanece separado, e não se recuperam incidentes sem captura suficiente. CAS da referência ativa, último snapshot global e revisão do journal evita gravar diferenças calculadas sobre observações concorrentes antigas.

Transação grava relatório, eventos, estado de incidentes e journal de forma consistente. Repetir o mesmo checkId com o mesmo relatório é idempotente; payload divergente é rejeitado. Retenção remove histórico expirado conforme configuração, preservando baselines, últimos ponteiros e estado de deduplicação. O journal não armazena fonte de script ou valores completos.

## Semântica segura das atualizações

A linha do tempo, os destaques do dashboard, os exports CSV/JSON e as notificações de integridade mostram somente eventos da categoria `integrity` com `rule_id` não vazio: violação, inconclusão e recuperação/conclusão de uma regra avaliada. A vinculação vem do finding do motor; selecionar uma aba ou observar uma região não transforma qualquer edição de conteúdo em infração. Mudanças livres (`ContentDifference`, sem regra), inclusive valores, registros, cabeçalhos não protegidos e posição de aba não protegida, ficam fora dessas superfícies e nunca são enviadas como alertas de proteção.

O filtro é aplicado no banco antes de contar, paginar, exportar ou adquirir uma notificação para envio, incluindo pendências antigas. Não apaga eventos, baselines, configurações ou dados históricos. Diagnósticos operacionais de autenticação, rede, permissão e cota continuam disponíveis em Monitoramento e no histórico de notificações operacionais, sem serem apresentados como edição da planilha ou violação estrutural. Os relatórios técnicos preservam a comparação interna e a cobertura.

A regra `allow_record_deletion=false` protege os registros da referência aprovada, mesmo sem fórmulas. Uma chave única permite comprovar quais identidades faltam; uma redução na quantidade observada também comprova remoção. Sem chave estável, uma mudança sem redução de quantidade é inconclusiva quanto à preservação, não uma exclusão inventada. Região incompleta não comprova remoção. A violação continua enquanto o registro da referência estiver ausente; uma nova referência só muda essa obrigação com aprovação humana.

Os filtros de exportação (período inclusivo, planilha, aba e `event_type`) são cumulativos e independentes da paginação. CSV UTF-8/BOM neutraliza fórmulas; JSON permanece disponível. Export vazio não cria arquivo.

Autoria não é inferida da hora de detecção, da conta conectada ou do último editor Drive. Sem ligação exata com a alteração, o responsável é “Responsável não identificado”; evidências de auditoria são contexto separado.
