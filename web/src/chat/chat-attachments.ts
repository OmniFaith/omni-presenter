declare function status(text: string): void;

function fileElement(message: ChatMessage): HTMLElement {
  const file = message.file!;
  const box = node("div", undefined, "attachment");
  const url = "/api/chat/files/" + encodeURIComponent(file.id);
  if (file.mime.startsWith("audio/")) {
    const audio = node("audio");
    audio.preload = "metadata";
    audio.src = url;
    audio.hidden = true;
    const controls = node("div", undefined, "voice-controls");
    const play = node("button", undefined, "voice-play");
    play.append(icon("play"));
    play.title = "Ouvir mensagem de voz";
    play.setAttribute("aria-label", "Ouvir mensagem de voz");
    const info = node("div", undefined, "voice-info");
    const track = node("input");
    track.type = "range";
    track.min = "0";
    track.max = "100";
    track.step = "0.1";
    track.value = "0";
    track.setAttribute("aria-label", "Posição do áudio");
    const time = node("small", "Áudio · " + formatSize(file.size));
    info.append(node("strong", "Mensagem de voz"), track, time);
    play.onclick = (event: MouseEvent) => {
      event.stopPropagation();
      if (audio.paused) {
        audio
          .play()
          .catch((error) =>
            status("Não foi possível reproduzir: " + (error instanceof Error ? error.message : String(error))),
          );
      } else {
        audio.pause();
      }
    };
    audio.onplay = () => {
      play.replaceChildren(icon("pause"));
      play.title = "Pausar mensagem de voz";
      play.setAttribute("aria-label", "Pausar mensagem de voz");
    };
    audio.onpause = audio.onended = () => {
      play.replaceChildren(icon("play"));
      play.title = "Ouvir mensagem de voz";
      play.setAttribute("aria-label", "Ouvir mensagem de voz");
    };
    const update = () => {
      const duration = Number.isFinite(audio.duration) ? audio.duration : 0;
      track.value = String(duration ? (audio.currentTime / duration) * 100 : 0);
      time.textContent = duration
        ? clock(audio.currentTime) + " / " + clock(duration)
        : clock(audio.currentTime) + " · " + formatSize(file.size);
    };
    audio.ontimeupdate =
      audio.onloadedmetadata =
      audio.ondurationchange =
        update;
    track.oninput = () => {
      if (Number.isFinite(audio.duration)) {
        audio.currentTime = (audio.duration * Number(track.value)) / 100;
      }
    };
    controls.append(play, info);
    box.append(controls, audio);
  } else if (
    ["image/png", "image/jpeg", "image/gif", "image/webp"].includes(file.mime)
  ) {
    const image = node("img");
    image.src = url;
    image.alt = file.name;
    image.loading = "lazy";
    box.append(image);
  } else if (["video/mp4", "video/webm"].includes(file.mime)) {
    const video = node("video");
    video.controls = true;
    video.preload = "none";
    video.src = url;
    box.append(video);
  }
  if (!file.mime.startsWith("audio/")) {
    const kind = file.mime.startsWith("image/")
      ? "image"
      : file.mime.startsWith("video/")
        ? "video"
        : "file";
    const heading = resourceHeader(
      kind,
      kind === "image" ? "IMAGEM" : kind === "video" ? "VÍDEO" : "ARQUIVO",
      file.name,
      formatSize(file.size) + " · " + (file.name.split(".").pop() || "").toUpperCase(),
    );
    const titleEl = heading.querySelector("h3");
    const link = node("a", file.name);
    link.href = url;
    link.download = file.name;
    if (titleEl) titleEl.replaceChildren(link);
    box.append(heading);
  }
  return box;
}

async function upload(
  file: File | Blob & { name?: string },
  kind = "file",
  destination = target(),
): Promise<void> {
  const fileName = file.name || "audio.webm";
  status("Enviando " + fileName + "…");
  const attachment = await api<{ id: string }>(
    "/uploads?name=" + encodeURIComponent(fileName),
    {
      method: "POST",
      headers: { "Content-Type": file.type || "application/octet-stream" },
      body: file,
    },
  );
  await send({ kind, file_id: attachment.id }, destination);
  status("");
}

const fileInput = $<HTMLInputElement>("file");
if (fileInput) {
  fileInput.onchange = async () => {
    const file = fileInput.files?.[0];
    if (file) {
      try {
        await upload(file);
      } catch (e) {
        status((e as Error).message);
      }
    }
    fileInput.value = "";
  };
}

const audioFallback = $<HTMLInputElement>("audio-fallback");
if (audioFallback) {
  audioFallback.onchange = async () => {
    const file = audioFallback.files?.[0];
    if (file) {
      try {
        await upload(file, "audio");
      } catch (e) {
        status((e as Error).message);
      }
    }
    audioFallback.value = "";
  };
}

const recordBtn = $("record");
if (recordBtn) {
  recordBtn.onclick = async () => {
    if (state.recorder) {
      if (state.recorder.state === "recording") state.recorder.stop();
      return;
    }
    if (!navigator.mediaDevices?.getUserMedia || !window.MediaRecorder) {
      status(
        "Para gravar diretamente, abra o endereço HTTPS do chat. Você também pode gravar/enviar áudio pelo seletor do aparelho.",
      );
      audioFallback?.click();
      return;
    }
    try {
      state.stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mime = [
        "audio/webm;codecs=opus",
        "audio/ogg;codecs=opus",
        "audio/mp4",
      ].find((t) => MediaRecorder.isTypeSupported(t));
      state.recorder = new MediaRecorder(
        state.stream,
        mime ? { mimeType: mime } : undefined,
      );
      state.chunks = [];
      state.cancelRecording = false;
      state.recordingTarget = target();
      const started = Date.now();
      state.recorder.ondataavailable = (e: BlobEvent) => {
        if (e.data.size) state.chunks.push(e.data);
      };
      state.recorder.onstop = async () => {
        const type = state.recorder?.mimeType.split(";")[0] || "audio/webm";
        const destination = state.recordingTarget || undefined,
          cancel = state.cancelRecording;
        state.stream?.getTracks().forEach((t) => t.stop());
        state.recorder = null;
        if (state.recordTimer) clearInterval(state.recordTimer);
        $("recording").hidden = true;
        recordBtn.classList.remove("recording");
        recordBtn.title = "Gravar áudio";
        recordBtn.replaceChildren(icon("record"));
        $<HTMLInputElement>("input").disabled = false;
        composer();
        if (!cancel && state.chunks.length) {
          const extension = type.includes("ogg")
            ? "ogg"
            : type.includes("mp4")
              ? "m4a"
              : "webm";
          const audioFile = new File(
            state.chunks,
            ["audio-", Date.now(), ".", extension].join(""),
            { type },
          );
          try {
            await upload(audioFile, "audio", destination);
          } catch (e) {
            status((e as Error).message);
          }
        }
      };
      state.recorder.start();
      $("recording").hidden = false;
      recordBtn.classList.add("recording");
      recordBtn.title = "Parar e enviar áudio";
      recordBtn.replaceChildren(icon("stop"));
      $<HTMLInputElement>("input").disabled = true;
      state.recordTimer = setInterval(() => {
        const seconds = Math.floor((Date.now() - started) / 1000);
        $("recording-time").textContent =
          "Gravando · " +
          Math.floor(seconds / 60) +
          ":" +
          String(seconds % 60).padStart(2, "0");
        if (seconds >= 180 && state.recorder?.state === "recording")
          state.recorder.stop();
      }, 250);
      composer();
    } catch (e) {
      state.stream?.getTracks().forEach((t) => t.stop());
      status("Não foi possível acessar o microfone: " + (e as Error).message);
    }
  };
}

const cancelRec = $("cancel-recording");
if (cancelRec) {
  cancelRec.onclick = () => {
    state.cancelRecording = true;
    if (state.recorder?.state === "recording") state.recorder.stop();
  };
}

let songRequest = 0;
let songDestination: unknown;

interface SongItem {
  id: string | number;
  title: string;
  artist?: string;
}

async function searchSongs(): Promise<void> {
  const request = ++songRequest;
  const songError = $("song-error");
  if (songError) songError.textContent = "";
  const songResults = $("song-results");
  if (songResults) songResults.replaceChildren();
  try {
    const searchVal = $<HTMLInputElement>("song-search")?.value || "";
    const songs = await api<SongItem[]>("/songs?q=" + encodeURIComponent(searchVal));
    const dialog = $<HTMLDialogElement>("song-dialog");
    if (request !== songRequest || !dialog?.open) return;
    songResults?.replaceChildren(
      ...songs.map((song) => {
        const b = node("button", song.title);
        b.type = "button";
        if (song.artist) b.append(node("small", song.artist));
        b.onclick = async () => {
          if (b.disabled) return;
          b.disabled = true;
          try {
            await send({ kind: "song", song_id: song.id }, songDestination);
            dialog.close();
          } catch (e) {
            if (songError) songError.textContent = (e as Error).message;
          } finally {
            b.disabled = false;
          }
        };
        return b;
      }),
    );
    if (!songs.length && songResults) {
      songResults.append(
        node("p", "Nenhum louvor encontrado no aplicativo."),
      );
    }
  } catch (e) {
    if (request === songRequest && songError) {
      songError.textContent = (e as Error).message;
    }
  }
}

const songBtn = $("song");
if (songBtn) {
  songBtn.onclick = () => {
    songDestination = target();
    $<HTMLDialogElement>("song-dialog")?.showModal();
    $<HTMLInputElement>("song-search")?.focus();
    searchSongs();
  };
}

const songDialog = $<HTMLDialogElement>("song-dialog");
if (songDialog) {
  songDialog.onclose = () => {
    ++songRequest;
    clearTimeout(songTimer);
  };
}

