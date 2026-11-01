"use strict";

// Start the session only after every feature has registered its controls.
(async () => {
  // The optional HTTPS port serves voice/chat; control is on the main web port.
  const controlLink = document.querySelector(".header-control");
  if (controlLink && location.protocol === "https:" && location.port) {
    const url = new URL("/control", location.href);
    url.protocol = "http:";
    url.port = String(Number(location.port) - 1);
    controlLink.href = url.href;
  }
  const hash = new URLSearchParams(location.hash.slice(1));
  if (hash.has("token")) {
    state.token = hash.get("token");
    state.profile = hash.get("profile") || state.profile;
    history.replaceState(null, "", location.pathname);
  } else if (!state.token && sessionStorage.getItem("chat-token")) {
    state.token = sessionStorage.getItem("chat-token");
    state.profile = sessionStorage.getItem("chat-profile") || state.profile;
  }
  await loadProfiles();
  try {
    const session = await api("/session", { method: "POST" });
    state.profile = session.profile_id;
    enter();
  } catch {
    showLogin();
  }
  if (window.self !== window.top) document.body.classList.add("embedded");
  paintIcons();
  loadChatIcons();
  composer();
})();
