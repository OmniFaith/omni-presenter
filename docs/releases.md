# Builds e releases

O fluxo `.github/workflows/release.yml` gera pacotes portáteis e cria um **rascunho** no GitHub Releases. A publicação final é feita na página do release, após a revisão.

## Plataformas

| Sistema | Arquitetura | Target Rust | Arquivo |
| --- | --- | --- | --- |
| Linux (glibc) | x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz`, `.AppImage`, `.flatpak` |
| Linux (glibc) | ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz`, `.AppImage`, `.flatpak` |
| Windows | x86-64 | `x86_64-pc-windows-msvc` | `.zip`, instalador `-setup.exe` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `.zip`, instalador `-setup.exe` |
| macOS | Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS | Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |

Os runners ARM do GitHub precisam estar disponíveis para o repositório. Os binários Linux e AppImages usam a glibc do Ubuntu 24.04 e não são compatíveis com todas as distribuições antigas. Os pacotes Windows e macOS não são assinados ou notarizados e podem apresentar avisos ao abri-los. `ffmpeg`, `ffprobe` e `yt-dlp` são dependências externas para os recursos descritos em [operations.md](operations.md). No Flatpak esses programas do sistema não ficam automaticamente disponíveis dentro da sandbox, portanto esses recursos de vídeo não estão garantidos.

O instalador Windows instala para o usuário atual, sem exigir administrador, e cria atalhos no menu Iniciar e na área de trabalho. A instalação padrão fica em `%LOCALAPPDATA%\Programs\Omni Presenter`. Os atalhos usam `%LOCALAPPDATA%\Omni Presenter\workspace` como diretório de trabalho, onde ficam o banco de dados e outros arquivos do projeto. A desinstalação preserva esse workspace. Abrir o executável diretamente ou usar o `.zip` pode usar outro diretório de trabalho e, portanto, outro banco de dados.

Instale o Flatpak com `flatpak install --user ./nome-do-pacote.flatpak` e inicie com `flatpak run org.omnipresenter.OmniPresenter`; a instalação pode baixar o runtime do Flathub. O banco do Flatpak fica no diretório de dados privado do aplicativo (`~/.var/app/org.omnipresenter.OmniPresenter/data/omni-presenter`), separado do banco usado pelos outros formatos. Para usar o AppImage, conceda permissão de execução (`chmod +x nome.AppImage`) e execute-o.

## Preparar uma versão

1. Atualize `package.version` em `Cargo.toml` e atualize `Cargo.lock` executando `cargo check`. Confira e inclua ambos no commit.
2. Copie `release-notes/TEMPLATE.md` para `release-notes/vX.Y.Z.md` (ou, por exemplo, `vX.Y.Z-alpha.1.md`). Escreva as notas que aparecerão diretamente na descrição do GitHub Release. Mantenha os arquivos anteriores para preservar o histórico.
3. Faça commit dessas alterações e envie o commit e o workflow para o GitHub. Instale o [GitHub CLI](https://cli.github.com/) e autentique-se com `gh auth login`. Então execute:

   ```sh
   python3 tools/release.py build
   ```

   O comando lê a versão do `Cargo.toml`, verifica as notas, cria e envia a tag para `origin`, aguarda todos os builds do GitHub Actions, cria um rascunho do release e baixa os doze pacotes e o manifesto para `dist/`, verificando os hashes. Arquivos de mesmo nome em `dist/` são substituídos. Não há compilação local das outras plataformas: elas são compiladas nos runners GitHub, exigindo internet, acesso ao repositório e disponibilidade desses runners. No Windows, use `python`. O diretório de trabalho Git deve estar limpo. Uma tag já existente pode ser reutilizada se apontar para o commit atual; caso contrário, defina uma nova versão e crie as notas correspondentes. Para outro remoto, informe `--remote nome`; ele deve apontar para o repositório selecionado pelo `gh`. Se o envio falhar, a tag local é mantida para que possa ser enviada manualmente ou removida antes de tentar novamente.

4. Acompanhe **Actions → Build and draft release**. O fluxo valida a correspondência entre tag, versão e arquivo de notas, compila os seis targets e cria um rascunho somente depois que todos terminarem. Tags com sufixo, como `v1.2.3-alpha.1`, também são marcadas como pré-release.
5. No GitHub Releases, revise as notas e os treze anexos (doze pacotes e `SHA256SUMS.txt`), teste os pacotes necessários e publique o rascunho.

Para iniciar o release sem aguardar nem baixar os arquivos, use `python3 tools/release.py publish`.

Para executar novamente, use **Actions → Build and draft release → Run workflow** e informe uma tag existente. O fluxo usa o conteúdo daquela tag, inclusive as notas. Caso já exista um release com essa tag, remova o rascunho anterior antes de executar novamente; o fluxo não substitui releases existentes. Corrija problemas no código e crie uma nova versão/tag quando necessário.

## Verificar downloads

Baixe `SHA256SUMS.txt` e os pacotes para a mesma pasta. No Linux, `sha256sum -c SHA256SUMS.txt --ignore-missing` verifica os arquivos presentes; no macOS, `shasum -a 256 -c SHA256SUMS.txt` verifica todos os arquivos listados, então baixe todos ou filtre a linha do pacote desejado. No PowerShell, compare `Get-FileHash .\nome-do-pacote.zip -Algorithm SHA256` com a linha correspondente em `SHA256SUMS.txt`. Obtenha o manifesto pela página oficial do release: os hashes detectam alterações, mas não são uma assinatura digital.

## Build local

Com Python 3.11+, Rust e as bibliotecas de desenvolvimento da plataforma instalados, execute:

```sh
git submodule update --init --recursive
python3 tools/release.py build --local
```

O comando local lê a versão do `Cargo.toml` (incluindo sufixos como `-alpha`), detecta o target do Rust e compila, empacota e gera os hashes para a máquina atual. No Linux, gera também AppImage e Flatpak: instale `flatpak`, `flatpak-builder`, `curl` e `rsvg-convert` (pacote `librsvg2-bin` no Debian/Ubuntu ou `librsvg` no Arch), além das bibliotecas de desenvolvimento; é necessário acesso à internet para baixar o linuxdeploy e os runtimes do Flathub. Os empacotadores Linux devem ser executados em uma máquina da mesma arquitetura do target. No Windows, instale o [NSIS](https://nsis.sourceforge.io/Download) para gerar o instalador e use `python` em vez de `python3`. As notas da versão devem existir antes do build. Para escolher outro target já instalado e configurado, use `python3 tools/release.py build --target TRIPLE`. Os arquivos são gerados em `dist/`; o manifesto do build local contém apenas os pacotes recém-gerados. O diretório de trabalho ao iniciar os formatos portáteis determina onde a base de dados é criada.
