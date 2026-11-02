<div align="center">

# Omni Presenter

[English](README.md) · **Português (Brasil)** · [Español](README.es.md)

**Prepare. Visualize. Apresente.**

Uma área de trabalho para apresentações ao vivo com letras, passagens bíblicas, mídias e designs — com operação no computador e ferramentas para a equipe no navegador.

[![Rust](https://img.shields.io/badge/feito%20com-Rust-DEA584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-alpha-E6A23C?style=for-the-badge)](#primeiros-passos)
[![Licença: MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-6E7BF2?style=for-the-badge)](LICENSE)
[![Plataformas](https://img.shields.io/badge/plataformas-Linux%20%7C%20macOS%20%7C%20Windows-35A899?style=for-the-badge)](docs/operations.md#prerequisites-and-startup)

<br>

[![Entre no Discord](https://img.shields.io/badge/JOIN%20US%20ON%20DISCORD-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/ttUAyhRkF)

<br>

[Primeiros passos](#primeiros-passos) · [Conheça o fluxo](#da-biblioteca-para-o-ao-vivo) · [Documentação](#documentação) · [Desenvolvimento](#desenvolvimento)

</div>

---

<!-- Adicione uma captura de tela aqui quando estiver disponível: ![Área de trabalho do Omni Presenter](docs/images/workspace.png) -->

## Uma área de trabalho, da preparação à apresentação

O Omni Presenter reúne o conteúdo e os controles de um evento ao vivo em um aplicativo para computador. Monte um repertório com itens da biblioteca, confira o próximo item na prévia e mantenha a saída ao vivo independente enquanto prepara o que vem a seguir.

| | |
| --- | --- |
| **Monte sua biblioteca** | Organize músicas e letras, passagens bíblicas, imagens, vídeos e textos personalizados para reutilizar em eventos. |
| **Planeje o evento** | Monte repertórios e trabalhe com perfis de operador e de evento. |
| **Crie a saída visual** | Use temas de apresentação e crie composições visuais com camadas, texto e recursos gráficos. |
| **Mantenha o controle** | Prepare o conteúdo na prévia antes de enviá-lo ao vivo para telas físicas ou no navegador. |
| **Trabalhe em equipe** | Habilite o controle e o chat no navegador para sua equipe na rede local. |

## Da biblioteca para o ao vivo

```text
  BIBLIOTECA              PRÉVIA               SAÍDA AO VIVO
  Músicas · Bíblia   →    Confira o item   →   Tela física ou
  Mídias · Designs        e o tema              no navegador
         │                                             ▲
         └────────────── Repertório do evento ─────────┘
```

1. **Prepare** — adicione conteúdo à biblioteca e monte um repertório para o evento.
2. **Configure** — escolha um tema, uma proporção e uma saída física ou virtual.
3. **Visualize** — selecione um item e confira-o sem substituir o conteúdo ao vivo.
4. **Apresente** — envie o item ao vivo e navegue pelos slides no computador ou pelo controle no navegador.

## Primeiros passos

Instale uma versão atual do Rust com Cargo. No Linux, a compilação também requer bibliotecas de desenvolvimento de ALSA, fontconfig e do sistema de janelas X11/Wayland.

```sh
# Inicialize o submódulo de dados bíblicos após clonar
git submodule update --init --recursive

# Execute a partir da pasta do projeto
cargo run

# Ou gere um executável otimizado
cargo build --release
```

O executável de lançamento fica em `target/release/`. O aplicativo armazena seu banco padrão no **diretório de trabalho**; portanto, inicie-o no diretório onde deseja manter os dados.

Para reproduzir vídeos, instale `ffmpeg` e `ffprobe`. A extração e o download de vídeos da web compatíveis também exigem `yt-dlp`. Consulte o [guia de operação](docs/operations.md) para configuração e solução de problemas.

## Traga sua equipe

O servidor web vem desabilitado por padrão. Habilite-o nas configurações para usar as ferramentas no navegador e abra os endereços exibidos pelo aplicativo em dispositivos que consigam acessar o computador.

- **Controle** — opere a apresentação em `/control` usando um perfil de operador.
- **Chat** — acesse as conversas da equipe em `/chat`.
- **Telas no navegador** — publique telas virtuais configuradas para clientes no navegador.

A porta padrão configurada é `8080`. O mDNS pode anunciar os serviços em redes compatíveis; consulte [acesso pelo navegador e descoberta](docs/operations.md#browser-access-and-discovery) para mais detalhes.

## Sua área de trabalho, seus arquivos

`omni-presenter.db` contém os dados e as configurações da área de trabalho. Anexos do chat, mídias baixadas da web, recursos de design e outros arquivos referenciados podem estar fora do banco. Para mover ou fazer backup da área de trabalho, inclua também esses arquivos. O [guia de backup](docs/operations.md#storage-and-backups) explica o que preservar.

O aplicativo inclui recursos de interface em inglês, português brasileiro e espanhol. Perfis de operador, atalhos e aparência da interface podem ser configurados no aplicativo.

## Documentação

| Guia | Conteúdo |
| --- | --- |
| [Operação](docs/operations.md) | Inicialização, saídas de apresentação, rede, backup e solução de problemas |
| [Arquitetura do sistema](docs/system.md) | Estado do aplicativo, comandos do navegador, persistência e renderização |
| [Desenvolvimento e internacionalização](docs/development.md) | Verificações de build, catálogos Fluent e pontos de entrada para extensões |
| [Dados bíblicos](assets/bible/README.md) · [Fontes](assets/fonts/README.md) | Conteúdo incluído, formatos e licenciamento das fontes |

Consulte o [índice completo da documentação](docs/README.md). Os guias estão em inglês.

## Desenvolvimento

```sh
cargo test
python3 tools/i18n_audit.py --catalogs
```

O projeto está em estágio alpha. Alguns testes de integração exigem ferramentas adicionais ou um ambiente gráfico ou de rede adequado; consulte o [guia de desenvolvimento](docs/development.md#build-and-test).

## Licença

O Omni Presenter está disponível sob a [licença MIT](LICENSE). Recursos de terceiros podem ter suas próprias licenças; consulte a documentação que acompanha esses recursos.

### Créditos dos dados bíblicos

Os dados bíblicos são fornecidos por [thiagobodruk/bible](https://github.com/thiagobodruk/bible), mantido por Thiago Bodruk e colaboradores, e incluídos como submódulo Git. O código e os dados desse projeto são distribuídos sob sua [licença MIT](assets/bible-source/LICENSE); segundo o projeto original, as versões bíblicas são propriedade dos respectivos titulares, aos quais são reservados todos os direitos. Consulte o [README original](https://github.com/thiagobodruk/bible#what-about-the-license-and-the-copyrights) para os avisos e as versões individuais.

---

<div align="center">

Gostou do Omni Presenter? Você pode [apoiar o projeto no Ko-fi](https://ko-fi.com/lauel).

</div>
