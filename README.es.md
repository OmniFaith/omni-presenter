<div align="center">

# Omni Presenter

**Prepara. Previsualiza. Presenta.**

Un espacio de trabajo para presentaciones en vivo con letras, Biblia, medios y diseños, control desde el equipo y herramientas para el grupo en el navegador.

[English](README.md) · [Português (Brasil)](README.pt-BR.md) · **Español**

[![Rust](https://img.shields.io/badge/hecho%20con-Rust-DEA584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Estado](https://img.shields.io/badge/estado-alpha-E6A23C?style=for-the-badge)](#primeros-pasos)
[![Licencia MIT](https://img.shields.io/badge/licencia-MIT-6E7BF2?style=for-the-badge)](LICENSE)
[![Plataformas](https://img.shields.io/badge/plataformas-Linux%20%7C%20macOS%20%7C%20Windows-35A899?style=for-the-badge)](docs/operations.md)

[![Únete a Discord](https://img.shields.io/badge/JOIN%20US%20ON%20DISCORD-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/ttUAyhRkF)

[Primeros pasos](#primeros-pasos) · [Flujo de presentación](#de-la-biblioteca-a-la-presentación) · [Documentación](#documentación) · [Créditos](#licencia-y-créditos)

</div>

---

## Todo para preparar y presentar

| | |
| --- | --- |
| **Biblioteca** | Organiza canciones, letras, pasajes bíblicos, imágenes, vídeos y textos. |
| **Planificación** | Prepara listas y configura perfiles de operador y de evento. |
| **Diseño** | Crea composiciones visuales y configura temas de presentación. |
| **Operación** | Comprueba el contenido en la vista previa antes de enviarlo a una pantalla física o al navegador. |
| **Equipo** | Activa el control y el chat en el navegador a través de la red local. |

## De la biblioteca a la presentación

1. Añade contenido a la biblioteca y prepara la lista del evento.
2. Configura el tema, la proporción y la pantalla de salida.
3. Comprueba el siguiente elemento en la vista previa, independientemente del contenido en vivo.
4. Envía el elemento a la presentación y navega por las diapositivas.

## Primeros pasos

Instala una versión actual de Rust con Cargo. En Linux, la compilación también requiere las bibliotecas de desarrollo de ALSA, fontconfig y X11/Wayland.

```sh
# Después de clonar el proyecto, descarga los datos bíblicos
git submodule update --init --recursive

# Ejecuta desde la carpeta del proyecto
cargo run

# O genera una versión optimizada
cargo build --release
```

El ejecutable de lanzamiento se encuentra en `target/release/`. La base de datos `omni-presenter.db` se crea en el directorio de trabajo del proceso. Para reproducir vídeos, instala `ffmpeg` y `ffprobe`; para extraer y descargar vídeos web, instala también `yt-dlp`.

## Red y datos

El servidor web está desactivado por defecto. Actívalo en los ajustes para usar `/control`, `/chat` y las pantallas virtuales. El puerto configurado por defecto es `8080`; utiliza las direcciones mostradas por la aplicación. El control requiere autenticación con un perfil de operador.

Para realizar copias de seguridad, incluye la base de datos, los archivos adjuntos, los medios descargados, los recursos de diseño y los archivos externos referenciados. Consulta la [guía de operación](docs/operations.md) para más información sobre red, almacenamiento y resolución de problemas.

## Documentación

- [Operación](docs/operations.md) · [Arquitectura](docs/system.md) · [Desarrollo e internacionalización](docs/development.md) — en inglés.
- [Datos bíblicos](assets/bible/README.md) · [Fuentes](assets/fonts/README.md).

El proyecto está en fase alpha. Para validar cambios, ejecuta `cargo test` y `python3 tools/i18n_audit.py --catalogs`.

## Licencia y créditos

Omni Presenter se distribuye bajo la [licencia MIT](LICENSE). Los recursos de terceros pueden tener sus propias licencias.

Los datos bíblicos proceden de [thiagobodruk/bible](https://github.com/thiagobodruk/bible), de Thiago Bodruk y colaboradores, incluido como submódulo Git. El código y los datos de ese proyecto se distribuyen bajo su [licencia MIT](assets/bible-source/LICENSE). Según el proyecto original, las versiones bíblicas son propiedad de sus respectivos titulares, que se reservan todos los derechos. Consulta el [aviso original sobre licencia y derechos de autor](https://github.com/thiagobodruk/bible#what-about-the-license-and-the-copyrights).

---

<div align="center">

¿Te gusta Omni Presenter? [Apoya el proyecto en Ko-fi](https://ko-fi.com/lauel).

</div>
