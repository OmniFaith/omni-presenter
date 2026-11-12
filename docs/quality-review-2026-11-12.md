# Checkup de qualidade e arquitetura — 12/11/2026

## Escopo e conclusão

Revisão do working tree, incluindo as correções de Bíblia desta sessão. O grafo
existente do Graphify foi usado como índice; os achados foram confirmados nos
arquivos atuais. Não é uma auditoria exaustiva de segurança nem uma medição de
cobertura ou desempenho.

A organização em `core`, `app`, `services`, `render` e `ui` é uma boa base para
evolução incremental. A separação entre prévia e saída ao vivo e a fila de comandos
remotos são pontos positivos. Não recomendo reescrever o aplicativo: as prioridades
são proteger persistência e releases e reduzir divergências de estado/renderização.

## Achados priorizados

Os achados abaixo registram o estado da revisão inicial. Consulte o andamento das
correções ao final deste documento antes de tratá-los como problemas ainda abertos.

### P1 — Falha de persistência pode ficar invisível ao usuário

**Evidência:** `src/app/mod.rs`, implementação de `Default`, abre
`omni-presenter.db` e, em qualquer erro, usa `AppDb::open_in_memory()` sem comunicar
o erro original. O caminho relativo também depende do diretório de lançamento.

**Impacto:** a aplicação pode aparentar salvar normalmente, mas as alterações da
sessão em memória não sobrevivem ao encerramento. Abrir de outro diretório pode
parecer uma perda do workspace, embora outro arquivo tenha sido criado.

**Recomendação:** apresentar erro e opções explícitas de recuperação; identificar
visivelmente qualquer modo temporário. Resolver/exibir o caminho do workspace e
testar falhas de abertura, permissões e restauração. Não mover bancos existentes
automaticamente sem uma migração e backup.

### P1 — O build Rust não verifica se o frontend foi compilado

**Evidência:** `build.rs::build_web_assets` descarta o resultado de
`Command::new("bun").arg("run").arg("build").status()`. As fontes estão em
`web/src`; os arquivos de `assets/web` também são versionados.

**Impacto:** Bun ausente ou erro TypeScript pode deixar o build Rust prosseguir
com assets antigos. Um executável compilado com sucesso não comprova que o
frontend corresponde às fontes.

**Recomendação:** tornar a política explícita: compilar e exigir sucesso, ou
consumir assets previamente gerados cuja correspondência seja verificada na CI.
Gerar arquivos no diretório de build, quando possível, em vez de alterar o
working tree durante a compilação Rust.

### P1 — Não há gate de qualidade em pull requests

**Evidência:** os workflows versionados são `release.yml` e
`prepare-release-notes.yml`. O primeiro empacota releases a partir de tags e não
executa testes/Clippy/TypeScript; o segundo prepara notas.

**Impacto:** regressões podem chegar à branch principal e ao empacotamento sem
validação automática dos comportamentos já cobertos pelos testes.

**Recomendação:** workflow de PR/push com checkout recursivo, toolchains
controladas, `cargo test --all-targets --locked`, Clippy, formatação, checagem
TypeScript e auditoria de catálogos. Começar com baseline dos avisos existentes;
não ativar `-D warnings` antes de tratar esse passivo. Executar testes gráficos e
de navegador em jobs próprios, com dependências explícitas.

### P2 — Operação de banco pode devolver sucesso mesmo falhando

**Evidência:** `src/services/db/themes.rs::update_themes_order` descarta o
resultado de cada `UPDATE` e retorna `Ok(())`, sem transação do lote.

**Impacto:** a ordenação pode ficar parcialmente persistida e o chamador não tem
como informar o erro ao usuário.

**Recomendação:** executar o lote em transação e propagar erros com contexto.
Revisar outros `let _ = conn.execute(...)` por finalidade: ignorar uma migração
idempotente conhecida não é equivalente a ignorar uma gravação operacional.
Adicionar testes de rollback e falha parcial.

### P2 — A prévia de configurações reimplementa a renderização da Bíblia

**Evidência:** `src/ui/pages/bible/settings_preview.rs` mede/desenha textos,
aspas, numeração e referência manualmente; `src/render/text.rs` faz a projeção
usando diretivas produzidas por `src/core/bible_format.rs`.

**Impacto:** as duas prévias podem mostrar resultados diferentes. Mudanças como
numeração fora das aspas precisam ser reproduzidas em mais de um caminho.

**Recomendação:** usar o mesmo formatador e renderizador na prévia das
configurações. Evoluir para um modelo estruturado de slide bíblico — texto,
numeração, referência e opções — sem codificar semântica em prefixos de string.
Manter testes de equivalência entre prévia, saída nativa e saída web.

### P2 — Estado de aplicação e domínio têm fronteiras porosas

**Evidência:** `src/core/state/mod.rs` importa `crate::ui::theme::AppTheme`,
armazena serviços de chat/áudio, tarefas de importação e progressos de conversão,
além de preferências e seleção de conteúdo. `capture_profiles` e
`apply_operator_profile` copiam manualmente campos entre estado e perfil.

**Impacto:** novas preferências podem funcionar na tela mas não sobreviver à
troca de perfil; serviços e UI tornam mais difícil testar o domínio isolado.
Adicionar módulos menores, sozinho, não elimina esse acoplamento.

**Recomendação:** separar gradualmente estado persistente do operador, estado
transitório da tela e recursos de runtime. Centralizar transições em ações da
camada `app`; migrar uma responsabilidade por vez, com testes de round-trip e
troca de perfil. Não converter toda a aplicação em múltiplos crates de uma vez.

**Proteção arquitetural:** `tests/architecture_tests.rs` verifica apenas o limite
de 1.000 linhas. Manter esse guardrail, mas adicionar regras sobre dependências e
testes de invariantes: preview não altera live, render não restaura atalhos
removidos e preferências persistidas são restauradas fielmente.

### P3 — Passivo de lint e formatação reduz o sinal das verificações

**Evidência nesta execução:** Clippy concluiu sem erro, com 136 avisos no binário
e 192 na compilação de testes, incluindo os 136 compartilhados, além de um aviso
no build script. Muitos são simplificações de estilo; outros apontam funções com
muitos argumentos e tipos complexos. O check recursivo do rustfmt encontrou
diferenças em 202 arquivos.

**Recomendação:** separar uma mudança mecânica de formatação de alterações
funcionais. Corrigir primeiro avisos de complexidade e comportamento nos módulos
tocados; introduzir bloqueio de novas regressões e então reduzir o baseline.
Não aplicar `clippy --fix` indiscriminadamente durante correções de interface.

## Validações executadas

- `cargo clippy --all-targets --message-format=short`: concluiu com os avisos acima.
- `bun run check` em `web/`: passou.
- `python3 tools/i18n_audit.py --catalogs`: passou; 2.171 entradas completas em
  inglês, português brasileiro e espanhol.
- `rustfmt --check --edition 2024 src/main.rs`: falhou por diferenças de formatação;
  não houve reformatação em massa.
- `cargo test --all-targets`: a primeira rodada identificou uma expectativa de
  inicialização de atalhos afetada pela correção de remoção. O comportamento foi
  ajustado para distinguir perfil novo de barra explicitamente esvaziada. A rodada
  final passou: 342 testes unitários e um teste de arquitetura; cinco testes
  dependentes de ambiente ficaram explicitamente ignorados.
- O teste de centralização cobre referência superior/inferior, tema com caixa
  deslocada, fonte real do tema e tamanhos distintos de prévia.
- Não foram executados os testes explicitamente ignorados que dependem de
  Chromium/Playwright, compositor gráfico ou rede multicast.

## Correções funcionais incluídas nesta sessão

- Rolagem e dimensionamento da janela de configurações da Bíblia.
- Referências visíveis e centralização com referência superior/inferior.
- Preferência lista/cards por operador e painéis de cards com rolagem visível.
- Numeração resumida fora das aspas, inclusive ao dividir um versículo em slides.
- Remoção de tradução dos atalhos sem reinserção automática durante renderização
  ou restauração do perfil; tradução ativa e tradução fixada são estados distintos.
- Extração de um teste de renderização para módulo separado para respeitar o
  limite de tamanho sem remover cobertura.

## Ordem sugerida de evolução

1. Proteger abertura/gravação do banco e tornar o build web verificável.
2. Implantar CI de qualidade para PRs, com baseline explícito de lints/formatação.
3. Unificar a prévia bíblica com o renderizador de produção.
4. Separar preferências persistentes, estado transitório e serviços de runtime.
5. Ampliar testes reais de interação/rolagem/perfis e equivalência desktop/web.

As recomendações de arquitetura e infraestrutura são um plano de trabalho, não
refatorações aplicadas por esta revisão.

## Correções aplicadas após a revisão

- **Abertura do workspace:** removido o fallback silencioso para memória. A
  inicialização valida as preferências salvas e, em caso de falha, mostra um erro
  localizado com o caminho absoluto do banco. Preferências inválidas não são
  substituídas por defaults no encerramento. O caminho continua relativo ao
  diretório de execução por compatibilidade, sem migração automática de dados.
- **Build web:** Bun e sucesso da compilação agora são obrigatórios. O build usa
  a instalação local do TypeScript, sem download implícito por `bun x`, e monitora
  o lockfile. Adicionados testes para Bun ausente e builds com sucesso/falha.
- **Ordenação dos temas:** atualizações em transação, com propagação de erro,
  rollback e notificação nas interfaces chamadoras. Testes simulam falha na
  segunda gravação e IDs inexistentes.
- **Prévia da Bíblia:** removida a implementação independente de desenho de
  versículos/referências. A prévia usa o formatador e o renderizador de produção;
  a amostra resumida mantém o número fora das aspas. A interface deixa de simular
  estilos que não estariam presentes na projeção real.
- **CI:** criado `quality.yml` para PRs e pushes em main/master, com toolchains
  controladas, testes, TypeScript, catálogos e orçamento Clippy. O workflow de
  release também instala Bun e as dependências web pelo lockfile. A execução
  remota do GitHub Actions só ocorrerá após enviar as mudanças ao repositório.
- **Clippy:** baseline explícito de 191 avisos únicos, separado por lint, com
  verificador e quatro testes de regressão. Aumento em uma categoria bloqueia a
  CI; o orçamento não comprova a identidade de cada aviso individual.
- **Arquitetura:** além do limite de linhas, um guardrail verifica referências
  diretas de novos módulos `core` a UI/serviços, com exceções explícitas para
  adaptadores legados. É uma checagem textual leve, não análise completa de AST.
- **Documentação:** atualizados pré-requisitos de build, procedimento de falha de
  abertura do banco e política de qualidade nos guias e READMEs.

Permanecem como evolução gradual: separação completa de `ControlPanelState`,
limpeza dos avisos legados, formatação global e validação visual/multiplataforma.
O check de formatação na CI é informativo, não um gate aprovado. Não foi feita
uma reformatação em massa nem uma reescrita do domínio.

### Validação das correções

- `cargo test --all-targets --locked`: 350 testes unitários, dois de arquitetura
  e três do build web passaram; cinco dependentes de ambiente ficaram ignorados.
- `python3 tools/test_quality_checks.py`: quatro testes passaram.
- `python3 tools/check_clippy.py`: aprovado com 191 avisos únicos dentro do
  orçamento explícito de 191; isso não significa ausência de avisos.
- `bun run build` e `bun run check`: passaram.
- Auditoria dos catálogos: 2.176 entradas completas em en/pt-BR/es.
- GitHub Actions e pacotes de outros sistemas operacionais não foram executados
  remotamente nesta sessão.
