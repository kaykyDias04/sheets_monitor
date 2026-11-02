# Modelo de diferenças, eventos e incidentes

Conteúdo e integridade são comparações distintas. O baseline aprovado define obrigações estruturais; a observação anterior define mudanças de conteúdo. Uma edição permitida gera evento e pode manter Online. Offline exige violação crítica comprovada.

Cada evento inclui eventId, checkId, spreadsheetId, sheetId opcional, categoria, tipo, severidade, regra opcional, localização, referências anterior/atual, data, fingerprint e diagnóstico. Referências de conteúdo são hashes, sem valores brutos. Cabeçalhos/fórmulas necessários aos diagnósticos são dados privados locais; não devem entrar em evidências públicas.

Com chave configurada e única, o comparador identifica registros adicionados/removidos e células alteradas por identidade lógica. Reordenação de registros é registrada sem transformar todo deslocamento após inserção em spam. Sem chave estável, usa alinhamento/sequência e hashes de linhas, indicando mudança não especificada quando não é possível provar se houve inserção, exclusão ou edição. Não inventa operações.

Incidentes estruturais têm identidade estável e fingerprint do estado. A→B cria ocorrência; B persistente não cria evento idêntico; B→A gera recuperação; A→B posterior cria nova ocorrência. Uma mudança no diagnóstico mantém a identidade da regra. Inconclusão atual não é prova de recuperação de uma violação anterior.

Coleta parcial: regiões completas e scripts obtidos atualizam um journal por componente. Falha em outro serviço não apaga uma diferença observada. O último snapshot global completo permanece separado, e não se recuperam incidentes sem captura suficiente. CAS da referência ativa, último snapshot global e revisão do journal evita gravar diferenças calculadas sobre observações concorrentes antigas.

Transação grava relatório, eventos, estado de incidentes e journal de forma consistente. Repetir o mesmo checkId com o mesmo relatório é idempotente; payload divergente é rejeitado. Retenção remove histórico expirado conforme configuração, preservando baselines, últimos ponteiros e estado de deduplicação. O journal não armazena fonte de script ou valores completos.

A tela técnica consulta os últimos200 eventos; histórico visual definitivo e notificações nativas ficam para etapas futuras. Não há deduplicação que suprima diferenças distintas comprovadas só para reduzir toasts. Nesta sprint os eventos são persistidos para a futura entrega de notificações, sem scheduler ou envio nativo.
