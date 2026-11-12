# Workspace backups and lyrics import

Open **Settings → Import / Export**. File dialogs, archive I/O and document parsing run in background workers.

## Complete workspace export

The **Export backup → Export ZIP** card creates a versioned ZIP containing `manifest.json` and referenced local assets. Expand **Backup contents, formats and precautions** for details in the application. It includes:

- Settings, operator and event profiles, sessions, playlists, quick-bar items and keyboard shortcuts.
- Songs, tags, folders and linked song versions.
- Presentation themes, theme options and theme backgrounds.
- Design documents and their local images.
- Media catalog and local media files, custom texts and saved verses.
- Installed Bible translations, interface themes and plugin packages/settings.

The Settings draft is included, along with the profiles captured when Settings opened. Save other open editors before exporting. Files referenced by theme options, wallpaper settings, designs, media playlists and quick-bar items are packaged once per stored path. Missing local files are reported; their original paths are retained. Web videos remain URLs and are not downloaded.

Chat history/uploads, chat certificates, installed system fonts, caches and external executables are excluded. Packages are **not encrypted** and may contain passwords, profile details and executable plugin scripts. Only import trusted backups. Install the necessary fonts and external runtime tools separately on the destination computer.

## Complete workspace restore

1. Choose **Import backup → Choose backup**. Review included assets and missing-file warnings; expand **View package contents** for table counts.
2. Confirm that you understand the replacement policy.
3. Choose **Restore and close application**.

This **replaces all backed-up categories**, rather than merging them or selecting individual categories. The chat is retained. The application suspends normal UI actions, remote listeners and autosave during restoration. Before changing data, it writes a mandatory `before_import_….zip` recovery package beside the current database. If recovery export fails, restoration is aborted.

Assets are streamed into a fresh `imported-assets/restored_…` directory beside the database, and known path fields are rewritten. Archive entry paths are never used as extraction destinations, existing source media files are not overwritten, and failed restores remove only their own newly created asset directory. The database replacement is a single transaction; failures roll it back. Imported SQL is never executed.

On success the app closes without saving its old in-memory state. **Reopen from the same workspace directory** to load the restored settings and catalogs. To undo a successful restore, import the corresponding recovery ZIP. Imported asset directories are intentionally retained after successful operations because saved paths depend on them.

Version 1 packages require the known table/column schema. Unsupported versions, unknown/duplicate tables, unexpected columns and malformed JSON are rejected. The manifest limit is 128 MiB, the expanded archive limit is 64 GiB and the entry limit is 100,000. These are safety limits, not claims that large imports fit every machine's available memory/disk.

## Additive lyrics import

Use **Import lyrics → Choose files** in Settings, or the import icon in the Lyrics library toolbar. Select one or several files. For extensionless text, switch the picker to **All files**. Review the resulting songs, uncheck unwanted entries, and edit title, artist, folder, tags or lyrics before confirming **Add selected songs**.

Existing songs are never replaced. The selected songs and their folders are added in one database transaction. Importing the same song again creates a separate copy. New songs appear in the library without changing prepared/live content. If a selected document cannot be parsed, the batch reports the file/error and does not add any songs; choose the remaining files again.

| Format | Behavior |
| --- | --- |
| Extensionless, `.txt`, `.text`, `.md`, `.markdown` | Plain text preserving Omni commands and markup. UTF-8 with/without BOM, or UTF-16 with BOM; CRLF/CR normalized. Not a general Markdown/HTML renderer. |
| `.lrc` | Removes leading timestamps; `[ti:…]` and `[ar:…]` supply title and artist. No synchronized playback/timing import. |
| `.srt`, `.vtt` | Extracts cue text; each cue becomes a stanza. No playback timing import. |
| `.pptx` | Text in presentation slide order, joined text runs and paragraph breaks. Each nonempty slide becomes a stanza; no images, notes, animation or visual formatting. |
| `.docx`, `.odt` | Extracts paragraphs, retaining empty paragraphs as stanza boundaries. No visual layout or embedded images. |
| `.odp` | Text per presentation page; each nonempty page becomes a stanza. |

One source file produces one song, with the filename as the default title. Legacy binary `.ppt`/`.doc`, PDF, RTF and legacy text encodings are unsupported: convert them first. No macros or document scripts are run. Text/XML entries are limited to 16 MiB, compressed document files to 128 MiB, PPTX expanded slide XML to 64 MiB, and each selection to 500 files / 64 MiB of resulting lyrics.

Header metadata before the first visible lyric line can use `//title:` / `//titulo:` / `//título:`, `//artist:` / `//artista:`, `//tags:` and `//folder:` / `//pasta:`. Blank lines separate stanzas. Metadata is removed from imported lyrics; ordinary comments, stanza labels, inline styling, per-slide directives and conditional breaks remain editable.

The complete, importable Portuguese example/command guide is [`exemplo-importacao-letras.txt`](exemplo-importacao-letras.txt), also available through **Example TXT and commands** in Settings. Command names and values follow the actual Omni parser, not PowerPoint syntax.
