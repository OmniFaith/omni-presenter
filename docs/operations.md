# Operating Omni Presenter

## Prerequisites and startup

After cloning, initialize the Bible data with `git submodule update --init --recursive`. Build and launch from the project directory with `cargo run`, or build with `cargo build --release` and run the resulting `target/release/omni-presenter` executable. The process working directory determines where the default database and web-video downloads are created; launching the same binary from another directory can therefore open a different workspace.

The desktop needs a graphical environment and working graphics drivers. Linux builds require the development packages for ALSA, fontconfig and the X11/Wayland stack. Install `ffmpeg` and `ffprobe` for video playback and `yt-dlp` for supported web-video extraction and downloads. These programs are external to the Rust build.

**Persistence:** if opening `omni-presenter.db` fails, the current application falls back to an in-memory database. Changes in that session will not survive a restart. If expected data is missing, check the working directory, database permissions and available disk space before continuing work.

## Presenting

1. Add or select content in the library. A playlist can contain items prepared for an event.
2. Select an item to inspect it in preview. Preview and live presentation state are separate: checking an item does not itself replace what is live.
3. Configure the target monitor or virtual screen and presentation theme in settings. Test the output and its aspect ratio before the event.
4. Send the selected content live and navigate its slides. Use the presentation controls to end the presentation when finished.

For network viewing, configure a virtual screen and use the screen address displayed in the application. Browser output and physical output are separate destinations; verify both if they are in use.

## Browser access and discovery

The web server is disabled in default settings. Enable it in the application settings and choose whether to allow LAN access. The default configured port is **8080**, but the actual address and port depend on the saved settings and network. A port conflict prevents the listener from starting; change the port or stop the conflicting service.

The browser interfaces use `/control` for the operator workspace, `/chat` for team chat and `/screen/<screen-id>` for a virtual screen. Use the URLs offered by the application rather than guessing screen IDs. Control access uses operator-profile authentication; preview and control endpoints are protected by that session. Treat screen URLs as presentation outputs, not as an access-control boundary for confidential content.

Discovery advertises `_omnipresenter._tcp.local.` over mDNS. The displayed hostname can use the host's name rather than `omni-presenter.local`. Discovery and `.local` name resolution depend on the network and client operating system; if name resolution fails, use the host's reachable IP address and configured port.

On Linux, `--configure-mdns` is an administrative setup operation. It checks for the NSS mDNS module, enables Avahi and updates the hosts entry in `/etc/nsswitch.conf`; it requires root privileges. It is not required just to launch the desktop application. Consult `src/services/discovery/install.rs` before incorporating it into an installer.

## Storage and backups

The default database, `omni-presenter.db`, contains workspace data and settings. Chat media can be stored in `chat-media/` alongside the database; web-video downloads use `web_media/` relative to the process working directory. Converted media copies are saved in `converted-media/` relative to the working directory. Design assets can be copied to a `design-assets/` directory. Other songs, images, video, wallpapers or fonts may be referenced by paths outside these directories.

Media imported from the file picker or dropped into the application can be converted before being added. Conversion is also available from a media item's context menu. PNG and WebP images and FLAC audio are lossless; MP4/H.264 and MP3 prioritize compatibility at high quality, while MKV/FFV1 offers lossless video in larger files. Conversion requires `ffmpeg` with the selected encoders installed. Original files are retained; PDF, animated GIF and SVG are imported unchanged. Video conversion keeps the first video stream and audio streams, but does not copy subtitles or additional video streams. The import prompt can be disabled in the dialog and re-enabled under Interface settings.

For a consistent backup, close the application first, then copy the database, relevant local asset directories and every externally referenced file. Preserve paths where possible. To restore, put the files back in their original locations, launch from the intended working directory and verify the library, media and outputs before an event.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Workspace appears empty | Confirm the process working directory and whether `omni-presenter.db` is readable and writable; the in-memory fallback is temporary. |
| Browser cannot connect | Confirm the server is enabled, the port is free, LAN access is allowed when needed, and the client can reach the host. Try the host IP if `.local` resolution fails. |
| Video does not play | Check that `ffmpeg` and `ffprobe` are installed and callable, and that the referenced media path is accessible. |
| Web video cannot be resolved or downloaded | Check `yt-dlp`, network access and the source URL. Downloads also require a writable `web_media/` directory. |
| A physical or browser output differs from preview | Check the selected output, theme and aspect ratio, then test with a known item. Preview is intentionally independent from live content. |
