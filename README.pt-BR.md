<div align="center">

# Omni Presenter

**Prepare. Visualize. Apresente.**

Uma área de trabalho para apresentações ao vivo com letras, Bíblia, mídias e designs, controle no computador e ferramentas para a equipe no navegador.

[English](README.md) · **Português (Brasil)** · [Español](README.es.md)

[![Rust](https://img.shields.io/badge/feito%20com-Rust-DEA584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-alpha-E6A23C?style=for-the-badge)](#primeiros-passos)
[![Licença MIT](https://img.shields.io/badge/licença-MIT-6E7BF2?style=for-the-badge)](LICENSE)
[![Plataformas](https://img.shields.io/badge/plataformas-Linux%20%7C%20macOS%20%7C%20Windows-35A899?style=for-the-badge)](docs/operations.md)

[![Entre no Discord](https://img.shields.io/badge/JOIN%20US%20ON%20DISCORD-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/ttUAyhRkF)

[Primeiros passos](#primeiros-passos) · [Fluxo de apresentação](#da-biblioteca-para-o-ao-vivo) · [Documentação](#documentação) · [Créditos](#licença-e-créditos)

</div>

---

## Tudo para preparar e apresentar

| | |
| --- | --- |
| **Biblioteca** | Organize músicas, letras, passagens bíblicas, imagens, vídeos e textos. |
| **Planejamento** | Monte repertórios e configure perfis de operador e de evento. |
| **Design** | Crie composições visuais e configure temas de apresentação. |
| **Operação** | Confira o conteúdo na prévia antes de enviá-lo para uma tela física ou para o navegador. |
| **Equipe** | Habilite o controle e o chat no navegador pela rede local. |

## Da biblioteca para o ao vivo

1. Adicione conteúdo à biblioteca e monte o repertório do evento.
2. Configure o tema, a proporção e a tela de saída.
3. Confira o próximo item na prévia, independente do conteúdo ao vivo.
4. Envie o item para a apresentação e navegue pelos slides.

## Primeiros passos

Instale uma versão atual do Rust com Cargo. No Linux, a compilação também requer bibliotecas de desenvolvimento de ALSA, fontconfig e X11/Wayland.

```sh
# Após clonar o projeto, obtenha os dados bíblicos
git submodule update --init --recursive

# Execute a partir da pasta do projeto
cargo run

# Ou gere uma versão otimizada
cargo build --release
```

O executável de lançamento fica em `target/release/`. O banco `omni-presenter.db` é criado no diretório de trabalho do processo. Para reproduzir vídeos, instale `ffmpeg` e `ffprobe`; para extração e download de vídeos da web, instale também `yt-dlp`.

## Rede e dados

O servidor web vem desabilitado por padrão. Ative-o nas configurações para usar `/control`, `/chat` e as telas virtuais. A porta padrão configurada é `8080`; use os endereços exibidos pelo aplicativo. O controle usa autenticação de perfil de operador.

Ao fazer backup, inclua o banco, anexos, mídias baixadas, assets de design e arquivos externos referenciados. Veja o [guia de operação](docs/operations.md) para detalhes de rede, armazenamento e solução de problemas.

## Documentação

- [Operação](docs/operations.md) · [Arquitetura](docs/system.md) · [Desenvolvimento e internacionalização](docs/development.md) — em inglês.
- [Dados bíblicos](assets/bible/README.md) · [Fontes](assets/fonts/README.md).

O projeto está em estágio alpha. Para validar alterações, execute `cargo test` e `python3 tools/i18n_audit.py --catalogs`.

## Licença e créditos

O Omni Presenter é distribuído sob a [licença MIT](LICENSE). Recursos de terceiros podem ter suas próprias licenças.

Os dados bíblicos vêm de [thiagobodruk/bible](https://github.com/thiagobodruk/bible), de Thiago Bodruk e colaboradores, incluído como submódulo Git. O código e os dados desse projeto são distribuídos sob sua [licença MIT](assets/bible-source/LICENSE). Conforme o repositório original, as versões bíblicas são propriedade dos respectivos titulares, aos quais são reservados todos os direitos. Consulte o [aviso de licenças e direitos autorais original](https://github.com/thiagobodruk/bible#what-about-the-license-and-the-copyrights).

---

<div align="center">

Gostou do Omni Presenter? [Apoie o projeto no Ko-fi](https://ko-fi.com/lauel).

</div>
