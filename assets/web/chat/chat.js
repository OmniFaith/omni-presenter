const $ = (id) => document.getElementById(id);
const interfaceLanguage = localStorage.getItem("chat-language") || navigator.language || "pt-BR";
let interfaceTexts = {};
const channelInteractions = new ChatChannelList({
    state: () => state,
    request: (path, options) => api(path, options),
    refresh: () => {
        state.fingerprint = "";
        poll();
    },
    open: (id) => selectRoom(id, null),
    create: () => $("channel-dialog").showModal(),
    report: (error) => status(interfaceTexts[error.message] || error.message),
    text: (key) => interfaceTexts[key] || key,
});
const state = {
    token: sessionStorage.getItem("chat-token") || "",
    authenticated: false,
    profile: sessionStorage.getItem("chat-profile") || "",
    channel: "general",
    peer: null,
    query: "",
    all: false,
    messages: [],
    channels: [],
    profiles: [],
    fingerprint: "",
    busy: false,
    loading: false,
    recorder: null,
    stream: null,
    chunks: [],
    recordingTarget: null,
    cancelRecording: false,
    recordTimer: null,
    generation: 0,
    history: false,
    hasOlder: false,
    pendingMessage: new URLSearchParams(location.hash.slice(1)).get("message"),
    notifications: {
        total: 0,
        channels: {},
        private: {},
        recent: [],
        latest_id: 0,
    },
    noticeCursor: null,
    webNotifications: localStorage.getItem("chat-notifications") === "on",
    emojiCategories: [],
    emojiCategory: 0,
    reactionTarget: null,
};
function node(tag, text, className) {
    const el = document.createElement(tag);
    if (text !== undefined)
        el.textContent = text;
    if (className)
        el.className = className;
    return el;
}
function status(text) {
    const el = $("status");
    el.textContent = text;
    el.hidden = !text;
}
async function api(path, options = {}) {
    if (path.startsWith("/state") ||
        path.startsWith("/emojis") ||
        path.startsWith("/translations")) {
        path +=
            (path.includes("?") ? "&" : "?") +
                "lang=" +
                encodeURIComponent(interfaceLanguage);
    }
    const headers = new Headers(options.headers);
    if (state.token)
        headers.set("Authorization", "Bearer " + state.token);
    if (options.json !== undefined) {
        headers.set("Content-Type", "application/json");
        options.body = JSON.stringify(options.json);
    }
    const response = await fetch("/api/chat" + path, { ...options, headers });
    const body = await response.json();
    if (!response.ok) {
        if (response.status === 401 && path !== "/login")
            showLogin();
        throw new Error(body.error || "Não foi possível concluir a operação");
    }
    return body;
}
const target = () => state.peer ? { recipient_id: state.peer } : { channel_id: state.channel };
function showLogin() {
    state.authenticated = false;
    state.cancelRecording = true;
    if (state.recorder?.state === "recording")
        state.recorder.stop();
    state.generation++;
    state.messages = [];
    state.peer = null;
    state.channel = "general";
    state.query = "";
    state.all = false;
    state.history = false;
    state.hasOlder = false;
    state.noticeCursor = null;
    state.notifications = {
        total: 0,
        channels: {},
        private: {},
        recent: [],
        latest_id: 0,
    };
    state.reactionTarget = null;
    $("emojis").hidden = true;
    $("timeline").replaceChildren();
    $("input").value = "";
    $("search").value = "";
    state.token = "";
    state.profile = "";
    sessionStorage.removeItem("chat-token");
    sessionStorage.removeItem("chat-profile");
    $("login").hidden = false;
    $("app").hidden = true;
}
function enter() {
    state.authenticated = true;
    sessionStorage.setItem("chat-token", state.token);
    sessionStorage.setItem("chat-profile", state.profile);
    $("login").hidden = true;
    $("app").hidden = false;
    state.fingerprint = "";
    poll();
    if (state.pendingMessage)
        openMessage(state.pendingMessage);
}
async function loadProfiles() {
    try {
        const profiles = await api("/profiles");
        profiles.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" }));
        state.profiles = profiles;
        $("profile").replaceChildren(...profiles.map((p) => {
            const o = node("option", p.name + (p.online ? " · online" : ""));
            o.value = p.id;
            return o;
        }));
        showLoginPhoto();
    }
    catch (e) {
        $("login-error").textContent = e.message;
    }
}
$("login-form").onsubmit = async (event) => {
    event.preventDefault();
    try {
        const result = await api("/login", {
            method: "POST",
            json: {
                profile_id: $("profile").value,
                password: $("code").value,
            },
        });
        state.token = result.token;
        state.profile = result.profile_id;
        $("code").value = "";
        enter();
    }
    catch (e) {
        $("login-error").textContent = e.message;
    }
};
$("logout").onclick = async () => {
    await api("/logout", { method: "POST" }).catch(() => { });
    showLogin();
    loadProfiles();
};
$("menu").onclick = () => $("sidebar").classList.toggle("open");
function selectRoom(channel, peer) {
    if (state.recorder) {
        status("Conclua ou cancele a gravação antes de trocar de conversa.");
        return;
    }
    state.channel = channel || "general";
    $("notification-toast").hidden = true;
    state.peer = peer;
    state.all = false;
    state.history = false;
    state.hasOlder = false;
    state.messages = [];
    state.fingerprint = "";
    state.generation++;
    $("sidebar").classList.remove("open");
    renderNavigation();
    poll();
}
function renderNavigation() {
    $("notify").classList.toggle("selected", "Notification" in window &&
        Notification.permission === "granted" &&
        state.webNotifications);
    const me = state.profiles.find((p) => p.id === state.profile);
    if (me)
        $("identity").textContent = me.name;
    if (me)
        profilePicture($("identity-avatar"), me, me.name);
    $("online-count").textContent =
        state.profiles.filter((p) => p.online).length + " online";
    $("notification-count").textContent = String(state.notifications.total > 99 ? "99+" : state.notifications.total);
    $("notification-count").hidden = !state.notifications.total;
    document.title =
        (state.notifications.total ? "(" + state.notifications.total + ") " : "") +
            "Omni Presenter · Chat da equipe";
    if (!channelInteractions.dragIds)
        $("channels").replaceChildren(...state.channels.map((c) => {
            const b = node("button", "# " + c.name, !state.peer && state.channel === c.id ? "selected" : "");
            b.title = c.description || c.tool || "";
            if (state.notifications.channels?.[c.id])
                b.append(unreadBadge(state.notifications.channels[c.id]));
            channelInteractions.bind?.(b, c);
            return b;
        }));
    $("people").replaceChildren(...state.profiles
        .filter((p) => p.id !== state.profile)
        .map((p) => {
        const b = node("button", undefined, state.peer === p.id ? "selected" : "");
        const photo = node("span", undefined, "person-avatar");
        profilePicture(photo, p, p.name);
        photo.append(node("span", "", p.online ? "presence online" : "presence"));
        b.append(photo, node("span", p.name));
        if (state.notifications.private?.[p.id])
            b.append(unreadBadge(state.notifications.private[p.id]));
        b.title = p.online ? "Online" : "Offline · as mensagens ficam salvas";
        b.onclick = () => selectRoom(null, p.id);
        return b;
    }));
    const room = state.peer
        ? state.profiles.find((p) => p.id === state.peer)
        : state.channels.find((c) => c.id === state.channel);
    $("room-title").textContent = state.all
        ? "Pesquisa em todas as conversas"
        : (state.peer ? "@ " : "# ") +
            (room?.name || interfaceTexts["chat.general"] || "Geral");
    $("room-description").textContent = state.peer
        ? "Mensagem privada · apenas vocês dois têm acesso"
        : room?.description ||
            "Conversas da equipe na rede local";
}
api("/translations")
    .then((messages) => {
    interfaceTexts = messages;
    if (state.authenticated)
        renderNavigation();
})
    .catch(() => { });
function unreadBadge(count) {
    return node("span", count > 99 ? "99+" : String(count), "unread-badge");
}
function profilePicture(container, profile, name) {
    if (!container)
        return;
    const url = profile?.avatar_url || "";
    if (container.dataset.url === url && container.dataset.name === name)
        return;
    container.dataset.url = url;
    container.dataset.name = name;
    container.replaceChildren();
    if (url) {
        const image = node("img");
        image.src = url;
        image.alt = name;
        container.append(image);
    }
    else {
        container.append(document.createTextNode(name
            .split(/\s+/)
            .map((s) => s[0])
            .slice(0, 2)
            .join("")
            .toUpperCase()));
    }
}
function showLoginPhoto() {
    const p = state.profiles.find((p) => p.id === $("profile").value);
    if (p)
        profilePicture($("login-avatar"), p, p.name);
}
$("profile").onchange = showLoginPhoto;
$("network-info").onclick = () => {
    $("network-link").value = location.origin + "/chat";
    $("network-dialog").showModal();
};
$("notify").onclick = async () => {
    if (!window.isSecureContext) {
        status("Abra o endereço HTTPS para ativar notificações do navegador.");
        return;
    }
    if (!("Notification" in window)) {
        status("Este navegador não oferece notificações.");
        return;
    }
    if (Notification.permission === "granted" && state.webNotifications) {
        state.webNotifications = false;
        localStorage.setItem("chat-notifications", "off");
        $("notify").classList.remove("selected");
        $("notify").title = "Ativar notificações";
        return;
    }
    const permission = await Notification.requestPermission();
    state.webNotifications = permission === "granted";
    localStorage.setItem("chat-notifications", state.webNotifications ? "on" : "off");
    $("notify").classList.toggle("selected", permission === "granted");
    $("notify").title =
        permission === "granted" ? "Notificações ativadas" : "Ativar notificações";
};
function notifyIncoming(summary) {
    const previous = state.noticeCursor;
    state.noticeCursor = Math.max(previous || 0, summary.latest_id || 0);
    if (previous === null)
        return;
    const item = summary.recent?.find((item) => item.id > previous);
    if (!item)
        return;
    const inRoom = item.peer_id
        ? state.peer === item.peer_id
        : !state.peer && state.channel === item.channel_id;
    if (inRoom && document.hasFocus() && !document.hidden)
        return;
    const toast = $("notification-toast");
    toast.replaceChildren(node("small", "Nova mensagem"), node("strong", item.author), node("span", item.preview));
    toast.hidden = false;
    toast.onclick = () => {
        selectRoom(item.channel_id, item.peer_id);
        toast.hidden = true;
    };
    setTimeout(() => {
        toast.hidden = true;
    }, 8000);
    if ("Notification" in window &&
        Notification.permission === "granted" &&
        state.webNotifications &&
        (!document.hasFocus() || document.hidden)) {
        const photo = state.profiles.find((p) => p.id === item.author_id)?.avatar_url;
        const notice = new Notification(item.author, {
            body: item.preview,
            icon: photo || undefined,
            tag: "omni-presenter-" + item.id,
        });
        notice.onclick = () => {
            window.focus();
            selectRoom(item.channel_id, item.peer_id);
            notice.close();
        };
    }
}
function resourceHeader(symbol, kind, title, subtitle) {
    const header = node("div", undefined, "resource-heading");
    const badge = node("div", undefined, "resource-icon");
    badge.append(icon(symbol));
    const info = node("div", undefined, "resource-info");
    info.append(node("small", kind), node("h3", title));
    if (subtitle)
        info.append(node("span", subtitle));
    header.append(badge, info);
    return header;
}
function clock(seconds) {
    seconds = Math.max(0, Math.floor(seconds || 0));
    return Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}
function formatSize(n) {
    return n > 1048576
        ? (n / 1048576).toFixed(1) + " MiB"
        : Math.ceil(n / 1024) + " KiB";
}
async function contentAction(id, action, button) {
    button.disabled = true;
    try {
        const command = await api("/messages/" + id + "/actions", {
            method: "POST",
            json: { action },
        });
        status("Executando no Omni Presenter…");
        for (let attempt = 0; attempt < 30; attempt++) {
            await new Promise((resolve) => setTimeout(resolve, 500));
            const result = await api("/actions/" + command.id);
            if (result.status === "error")
                throw new Error(result.error);
            if (result.status === "applied") {
                status(action === "shortcut"
                    ? "Conteúdo adicionado aos atalhos do seu perfil."
                    : action === "playlist"
                        ? "Conteúdo adicionado à lista de reprodução do culto ativo."
                        : "Conteúdo projetado/reproduzido no Omni Presenter.");
                return;
            }
        }
        status("A solicitação está na fila. Mantenha o Omni Presenter aberto para concluir.");
    }
    catch (error) {
        status(error.message);
    }
    finally {
        button.disabled = false;
    }
}
async function openMessage(id) {
    try {
        const message = await api("/messages/" + encodeURIComponent(id));
        selectRoom(message.channel_id, message.channel_id
            ? null
            : message.author_id === state.profile
                ? message.recipient_id
                : message.author_id);
        state.pendingMessage = null;
        const params = new URLSearchParams({ before: String(message.id + 1) });
        if (state.peer)
            params.set("peer", state.peer);
        else
            params.set("channel", state.channel);
        const snapshot = await api("/state?" + params);
        state.messages = snapshot.messages;
        state.history = true;
        state.hasOlder = snapshot.messages.length === 100;
        state.fingerprint = JSON.stringify(state.messages);
        renderMessages();
        document
            .querySelector(`[data-id="${message.id}"]`)
            ?.scrollIntoView({ block: "center" });
        history.replaceState(null, "", location.pathname);
    }
    catch (error) {
        status(error.message);
    }
}
function renderMessages() {
    const holder = $("messages"), nearBottom = holder.scrollHeight - holder.scrollTop - holder.clientHeight < 90;
    const rows = [];
    for (const m of state.messages) {
        const existing = $("timeline").querySelector(`article[data-id="${m.id}"]`);
        if (existing &&
            existing.dataset.context === String(state.all) &&
            existing.dataset.deleted === String(!!m.deleted)) {
            updateReactions(existing, m);
            profilePicture(existing.querySelector(".avatar"), state.profiles.find((p) => p.id === m.author_id), m.author || "");
            rows.push(existing);
            continue;
        }
        const row = node("article", undefined, "message" + (m.author_id === state.profile ? " own" : ""));
        row.dataset.id = String(m.id);
        row.dataset.context = String(state.all);
        row.dataset.deleted = String(!!m.deleted);
        row.classList.toggle("deleted", !!m.deleted);
        const authorName = m.author || "";
        const avatar = node("div", authorName
            .split(/\s+/)
            .map((s) => s[0])
            .slice(0, 2)
            .join("")
            .toUpperCase(), "avatar");
        const body = node("div", undefined, "message-body");
        const head = node("div", undefined, "message-head");
        profilePicture(avatar, state.profiles.find((p) => p.id === m.author_id), authorName);
        head.append(node("b", m.author_id === state.profile ? "Você" : authorName), node("time", new Date(m.timestamp).toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
        })));
        if (!m.deleted) {
            const tools = node("div", undefined, "message-tools");
            const react = node("button", undefined, "action-icon");
            react.append(icon("smile"));
            react.title = "Reagir à mensagem";
            react.setAttribute("aria-label", "Reagir à mensagem");
            react.onclick = (event) => {
                event.stopPropagation();
                openEmojiPicker(m.id, react.getBoundingClientRect());
            };
            tools.append(react);
            if (m.author_id === state.profile) {
                const remove = node("button", undefined, "action-icon danger");
                remove.append(icon("trash"));
                remove.title = "Excluir mensagem";
                remove.setAttribute("aria-label", "Excluir mensagem");
                remove.onclick = async (event) => {
                    event.stopPropagation();
                    try {
                        await api("/messages/" + m.id, { method: "DELETE" });
                        state.fingerprint = "";
                        poll();
                    }
                    catch (error) {
                        status(error.message);
                    }
                };
                tools.append(remove);
            }
            head.append(tools);
        }
        if (state.all) {
            const label = m.channel_id
                ? "# " +
                    (state.channels.find((c) => c.id === m.channel_id)?.name || "canal")
                : "Mensagem privada";
            head.append(node("small", label));
            row.onclick = () => selectRoom(m.channel_id || null, m.channel_id
                ? null
                : m.author_id === state.profile
                    ? m.recipient_id || null
                    : m.author_id || null);
        }
        body.append(head);
        let card = null;
        let cardTop = null;
        if (!m.deleted && (m.kind === "bible" || m.kind === "song")) {
            const shared = node("div", undefined, "shared");
            const details = node("details");
            details.append(node("summary", m.kind === "bible" ? "Ver passagem" : "Ver letra"), node("pre", m.text));
            const source = m.shared || {};
            cardTop = node("div", undefined, "resource-top");
            cardTop.append(resourceHeader(m.kind === "bible" ? "book" : "song", m.kind === "bible" ? "TEXTO BÍBLICO" : "LOUVOR", source.reference || source.title || m.title || "", source.version ? "Bíblia · " + source.version : source.artist || ""));
            shared.append(cardTop, details);
            body.append(shared);
            card = shared;
        }
        else if (m.text)
            body.append(node("div", m.text, "message-content"));
        if (m.file) {
            card = fileElement(m);
            body.append(card);
        }
        if (m.shared) {
            const actions = node("div", undefined, "shared-actions inline");
            const playable = !m.file ||
                /^(audio|video|image)\//.test(m.file.mime) ||
                /\.(mp4|mov|webm|mkv|avi|m4v|png|jpe?g|webp|gif|bmp|mp3|wav|ogg|m4a|flac|aac|wma)$/i.test(m.file.name);
            for (const [action, label] of [
                [
                    "project",
                    m.file?.mime.startsWith("audio/") ? "Abrir no player" : "Projetar",
                ],
                ["shortcut", "Fixar nos atalhos"],
                ["playlist", "Adicionar à lista"],
            ]) {
                if (action === "project" && !playable)
                    continue;
                const button = node("button", undefined, action === "project" ? "action-primary" : "action-icon");
                button.append(icon(action === "project"
                    ? m.file?.mime.startsWith("audio/")
                        ? "song"
                        : "play"
                    : action));
                if (action === "project")
                    button.append(node("span", label));
                button.title = label;
                button.setAttribute("aria-label", label);
                button.onclick = (event) => {
                    event.stopPropagation();
                    contentAction(m.id, action, button);
                };
                actions.append(button);
            }
            const link = node("a", undefined, "action-icon");
            link.append(icon("link"));
            link.title = "Copiar link do conteúdo";
            link.setAttribute("aria-label", "Copiar link do conteúdo");
            link.href = "/chat#message=" + m.id;
            link.onclick = (event) => {
                event.preventDefault();
                event.stopPropagation();
                if (navigator.clipboard?.writeText)
                    navigator.clipboard
                        .writeText(location.origin + "/chat#message=" + m.id)
                        .then(() => status("Link copiado. Somente os participantes da conversa podem abri-lo."))
                        .catch(() => openMessage(m.id));
                else
                    openMessage(m.id);
            };
            actions.append(link);
            if (m.file) {
                const download = node("a", undefined, "action-icon");
                download.append(icon("download"));
                download.href = "/api/chat/files/" + encodeURIComponent(m.file.id);
                download.download = m.file.name;
                download.title = "Baixar arquivo";
                download.setAttribute("aria-label", "Baixar arquivo");
                actions.append(download);
            }
            if (cardTop)
                cardTop.append(actions);
            else if (card)
                card.prepend(actions);
        }
        row.append(avatar, body);
        updateReactions(row, m);
        rows.push(row);
    }
    if (!state.messages.length)
        rows.push(node("p", state.query
            ? "Nenhuma mensagem encontrada."
            : "Este é o começo da conversa. Envie uma mensagem para a equipe.", "empty"));
    const timeline = $("timeline");
    const retained = new Set(rows);
    for (const child of [...timeline.children])
        if (!retained.has(child))
            child.remove();
    let cursor = timeline.firstElementChild;
    for (const row of rows) {
        if (row === cursor)
            cursor = cursor.nextElementSibling;
        else
            timeline.insertBefore(row, cursor);
    }
    $("older").hidden = !state.hasOlder;
    if (nearBottom)
        holder.scrollTop = holder.scrollHeight;
}
function updateReactions(row, message) {
    let bar = row.querySelector(".reactions");
    if (!bar) {
        bar = node("div", undefined, "reactions");
        row.querySelector(".message-body")?.append(bar);
    }
    bar.replaceChildren();
    for (const reaction of message.reactions || []) {
        const button = node("button", reaction.emoji + " " + reaction.users.length, "reaction-chip" +
            (reaction.users.includes(state.profile) ? " selected" : ""));
        button.title = reaction.users
            .map((id) => state.profiles.find((p) => p.id === id)?.name || id)
            .join(", ");
        button.setAttribute("aria-label", "Reagir com " + reaction.emoji);
        button.onclick = (event) => {
            event.stopPropagation();
            toggleReaction(message.id, reaction.emoji);
        };
        bar.append(button);
    }
    bar.hidden = !(message.reactions || []).length;
}
async function toggleReaction(id, emoji) {
    try {
        const message = await api("/messages/" + id + "/reactions", {
            method: "POST",
            json: { emoji },
        });
        const row = document.querySelector(`article[data-id="${id}"]`);
        if (row)
            updateReactions(row, message);
        state.fingerprint = "";
        poll();
    }
    catch (error) {
        status(error.message);
    }
}
window.toggleReaction = toggleReaction;
async function poll() {
    if (!state.authenticated || state.loading)
        return;
    state.loading = true;
    const generation = state.generation;
    const atBottom = $("messages").scrollHeight -
        $("messages").scrollTop -
        $("messages").clientHeight <
        90;
    try {
        const params = new URLSearchParams();
        if (!state.all) {
            if (state.peer)
                params.set("peer", state.peer);
            else
                params.set("channel", state.channel);
        }
        if (state.query)
            params.set("q", state.query);
        const snapshot = await api("/state?" + params);
        if (generation !== state.generation)
            return;
        state.profiles = snapshot.profiles;
        state.channels = snapshot.channels;
        if (!state.peer &&
            !state.channels.some((channel) => channel.id === state.channel))
            state.channel = "general";
        state.notifications = snapshot.notifications || state.notifications;
        notifyIncoming(state.notifications);
        renderNavigation();
        if (state.history) {
            const first = snapshot.messages[0]?.id ?? Infinity;
            snapshot.messages = [
                ...state.messages.filter((m) => m.id < first),
                ...snapshot.messages,
            ];
        }
        else
            state.hasOlder = snapshot.messages.length === 100;
        const fingerprint = JSON.stringify(snapshot.messages);
        if (state.fingerprint !== fingerprint) {
            state.messages = snapshot.messages;
            state.fingerprint = fingerprint;
            renderMessages();
        }
        for (const message of state.messages) {
            const avatar = document.querySelector(`article[data-id="${message.id}"] .avatar`);
            if (avatar)
                profilePicture(avatar, state.profiles.find((p) => p.id === message.author_id), message.author || "");
        }
        $("connection").textContent =
            state.profiles.filter((p) => p.online).length + " online · rede local";
        if (atBottom &&
            document.hasFocus() &&
            !document.hidden &&
            !state.query &&
            !state.all &&
            !state.history) {
            const count = state.peer
                ? state.notifications.private?.[state.peer]
                : state.notifications.channels?.[state.channel];
            if (count) {
                state.notifications = await api("/read", {
                    method: "POST",
                    json: {
                        channel_id: state.peer ? undefined : state.channel,
                        peer_id: state.peer || undefined,
                        upto: snapshot.messages.at(-1)?.id,
                    },
                });
                renderNavigation();
            }
        }
        if ($("status").dataset.connection === "error") {
            status("");
            delete $("status").dataset.connection;
        }
    }
    catch (e) {
        status(e.message);
        $("status").dataset.connection = "error";
        $("connection").textContent = "Reconectando…";
    }
    finally {
        state.loading = false;
    }
}
setInterval(poll, 1000);
window.addEventListener("online", poll);
document.addEventListener("visibilitychange", () => {
    if (!document.hidden)
        poll();
});
$("older").onclick = async () => {
    if (!state.messages.length)
        return;
    const generation = state.generation;
    $("older").disabled = true;
    try {
        const params = new URLSearchParams({ before: String(state.messages[0].id) });
        if (!state.all) {
            if (state.peer)
                params.set("peer", state.peer);
            else
                params.set("channel", state.channel);
        }
        if (state.query)
            params.set("q", state.query);
        const snapshot = await api("/state?" + params);
        if (generation !== state.generation)
            return;
        const current = state.messages;
        state.history = true;
        state.hasOlder = snapshot.messages.length === 100;
        state.messages = [...snapshot.messages, ...current];
        renderMessages();
    }
    catch (e) {
        status(e.message);
    }
    finally {
        $("older").disabled = false;
    }
};
let searchTimer;
$("search-toggle").onclick = () => {
    const open = Boolean($("search").hidden);
    $("search").hidden = !open;
    $("search-all").hidden = !open;
    $("search-toggle").setAttribute("aria-expanded", String(open));
    $("search-toggle").classList.toggle("selected", open);
    if (open)
        $("search").focus();
    else {
        $("search").value = "";
        state.query = "";
        state.all = false;
        state.history = false;
        state.generation++;
        state.fingerprint = "";
        poll();
    }
};
$("search").oninput = () => {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
        state.query = $("search").value.trim();
        state.history = false;
        state.fingerprint = "";
        state.generation++;
        poll();
    }, 250);
};
$("search-all").onclick = () => {
    state.all = !state.all;
    state.history = false;
    state.generation++;
    state.fingerprint = "";
    poll();
};
function composer() {
    const text = $("input").value.trim();
    $("record").hidden = !!text && !state.recorder;
    $("send").hidden = !text || !!state.recorder;
    $("input").style.height = "auto";
    $("input").style.height = Math.min($("input").scrollHeight, 150) + "px";
}
$("input").oninput = composer;
$("input").onkeydown = (event) => {
    if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        sendText();
    }
};
async function send(data, destination = target()) {
    const result = await api("/messages", {
        method: "POST",
        json: { ...destination, ...data },
    });
    state.fingerprint = "";
    await poll();
    $("messages").scrollTop = $("messages").scrollHeight;
    return result;
}
async function sendText() {
    const text = $("input").value.trim();
    if (!text || state.busy)
        return;
    state.busy = true;
    $("send").disabled = true;
    try {
        await send({ kind: "text", text });
        $("input").value = "";
        composer();
    }
    catch (e) {
        status(e.message);
    }
    finally {
        state.busy = false;
        $("send").disabled = false;
    }
}
$("send").onclick = sendText;
for (const b of document.querySelectorAll("[data-close]"))
    b.onclick = () => b.closest("dialog")?.close();
function selectChannelTool(value) {
    $("channel-tool").value = value;
    for (const button of document.querySelectorAll("[data-tool]")) {
        button.classList.toggle("selected", button.dataset.tool === value);
        button.setAttribute("aria-pressed", String(button.dataset.tool === value));
    }
}
for (const button of document.querySelectorAll("[data-tool]")) {
    const toolName = button.dataset.tool || "general";
    button.prepend(icon({ general: "chat", bible: "book", songs: "song", media: "file" }[toolName] || "chat"));
    button.onclick = () => selectChannelTool(button.dataset.tool || "general");
}
$("new-channel").onclick = () => {
    selectChannelTool($("channel-tool").value || "general");
    $("channel-dialog").showModal();
};
$("channel-form").onsubmit = async (event) => {
    event.preventDefault();
    try {
        const channel = await api("/channels", {
            method: "POST",
            json: {
                name: $("channel-name").value,
                tool: $("channel-tool").value,
                description: $("channel-description").value,
            },
        });
        $("channel-dialog").close();
        $("channel-form").reset();
        selectChannelTool("general");
        selectRoom(channel.id, null);
    }
    catch (e) {
        status(e.message);
    }
};
let songTimer;
$("song-search").oninput = () => {
    clearTimeout(songTimer);
    ++songRequest;
    $("song-results").replaceChildren();
    songTimer = setTimeout(searchSongs, 120);
};