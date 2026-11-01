function fileElement(message) {
  const file = message.file,
    box = node("div", undefined, "attachment");
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
    track.min = 0;
    track.max = 100;
    track.step = 0.1;
    track.value = 0;
    track.setAttribute("aria-label", "Posição do áudio");
    const time = node("small", "Áudio · " + formatSize(file.size));
    info.append(node("strong", "Mensagem de voz"), track, time);
    play.onclick = (event) => {
      event.stopPropagation();
      if (audio.paused)
        audio
          .play()
          .catch((error) =>
            status("Não foi possível reproduzir: " + error.message),
          );
      else audio.pause();
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
      track.value = duration ? (audio.currentTime / duration) * 100 : 0;
      time.textContent = duration
        ? clock(audio.currentTime) + " / " + clock(duration)
        : clock(audio.currentTime) + " · " + formatSize(file.size);
    };
    audio.ontimeupdate =
      audio.onloadedmetadata =
      audio.ondurationchange =
        update;
    track.oninput = () => {
      if (Number.isFinite(audio.duration))
        audio.currentTime = (audio.duration * Number(track.value)) / 100;
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
      formatSize(file.size) + " · " + file.name.split(".").pop().toUpperCase(),
    );
    const title = heading.querySelector("h3");
    const link = node("a", file.name);
    link.href = url;
    link.download = file.name;
    title.replaceChildren(link);
    box.append(heading);
  }
  return box;
}

async function upload(file, kind = "file", destination = target()) {
  status("Enviando " + file.name + "…");
  const attachment = await api(
    "/uploads?name=" + encodeURIComponent(file.name),
    {
      method: "POST",
      headers: { "Content-Type": file.type || "application/octet-stream" },
      body: file,
    },
  );
  await send({ kind, file_id: attachment.id }, destination);
  status("");
}
$("file").onchange = async () => {
  const file = $("file").files[0];
  if (file)
    try {
      await upload(file);
    } catch (e) {
      status(e.message);
    }
  $("file").value = "";
};
$("audio-fallback").onchange = async () => {
  const file = $("audio-fallback").files[0];
  if (file)
    try {
      await upload(file, "audio");
    } catch (e) {
      status(e.message);
    }
  $("audio-fallback").value = "";
};
$("record").onclick = async () => {
  if (state.recorder) {
    if (state.recorder.state === "recording") state.recorder.stop();
    return;
  }
  if (!navigator.mediaDevices?.getUserMedia || !window.MediaRecorder) {
    status(
      "Para gravar diretamente, abra o endereço HTTPS do chat. Você também pode gravar/enviar áudio pelo seletor do aparelho.",
    );
    $("audio-fallback").click();
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
    state.recorder.ondataavailable = (e) => {
      if (e.data.size) state.chunks.push(e.data);
    };
    state.recorder.onstop = async () => {
      const type = state.recorder.mimeType.split(";")[0] || "audio/webm";
      const destination = state.recordingTarget,
        cancel = state.cancelRecording;
      state.stream.getTracks().forEach((t) => t.stop());
      state.recorder = null;
      clearInterval(state.recordTimer);
      $("recording").hidden = true;
      $("record").classList.remove("recording");
      $("record").title = "Gravar áudio";
      $("record").replaceChildren(icon("record"));
      $("input").disabled = false;
      composer();
      if (!cancel && state.chunks.length) {
        const extension = type.includes("ogg")
          ? "ogg"
          : type.includes("mp4")
            ? "m4a"
            : "webm";
        const file = new File(
          state.chunks,
          ["audio-", Date.now(), ".", extension].join(""),
          { type },
        );
        try {
          await upload(file, "audio", destination);
        } catch (e) {
          status(e.message);
        }
      }
    };
    state.recorder.start();
    $("recording").hidden = false;
    $("record").classList.add("recording");
    $("record").title = "Parar e enviar áudio";
    $("record").replaceChildren(icon("stop"));
    $("input").disabled = true;
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
    status("Não foi possível acessar o microfone: " + e.message);
  }
};
$("cancel-recording").onclick = () => {
  state.cancelRecording = true;
  if (state.recorder?.state === "recording") state.recorder.stop();
};
let songRequest = 0;
let songDestination;
async function searchSongs() {
  const request = ++songRequest;
  $("song-error").textContent = "";
  $("song-results").replaceChildren();
  try {
    const songs = await api(
      "/songs?q=" + encodeURIComponent($("song-search").value),
    );
    if (request !== songRequest || !$("song-dialog").open) return;
    $("song-results").replaceChildren(
      ...songs.map((song) => {
        const b = node("button", song.title);
        b.type = "button";
        b.append(node("small", song.artist));
        b.onclick = async () => {
          if (b.disabled) return;
          b.disabled = true;
          try {
            await send({ kind: "song", song_id: song.id }, songDestination);
            $("song-dialog").close();
          } catch (e) {
            $("song-error").textContent = e.message;
          } finally {
            b.disabled = false;
          }
        };
        return b;
      }),
    );
    if (!songs.length)
      $("song-results").append(
        node("p", "Nenhum louvor encontrado no aplicativo."),
      );
  } catch (e) {
    if (request === songRequest) $("song-error").textContent = e.message;
  }
}
$("song").onclick = () => {
  songDestination = target();
  $("song-dialog").showModal();
  $("song-search").focus();
  searchSongs();
};
$("song-dialog").onclose = () => {
  ++songRequest;
  clearTimeout(songTimer);
};
