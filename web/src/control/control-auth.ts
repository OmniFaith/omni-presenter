const byId = <T extends HTMLElement = HTMLElement>(id: string): T | null =>
  document.getElementById(id) as T | null;

document.documentElement.dataset.theme =
  localStorage.getItem("control-theme") ||
  (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");

interface AuthResult {
  token?: string;
  profile_id?: string;
  error?: string;
  [key: string]: unknown;
}

interface ProfileItem {
  id: string;
  name: string;
  avatar_url?: string;
  online?: boolean;
}

const LAST_PROFILE_KEY = "control-last-profile";

async function auth<T = AuthResult>(
  path: string,
  body?: Record<string, unknown>,
): Promise<T> {
  const response = await fetch(
    "/api/control/" + path,
    body
      ? {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        }
      : {},
  );
  const result = (await response.json()) as AuthResult;
  if (!response.ok)
    throw new Error(result.error || "Não foi possível entrar.");
  return result as T;
}

async function start(): Promise<void> {
  await auth("session");
  await import("/control.js");
  document.body.dataset.locked = "false";
  const login = byId("control-login");
  if (login) login.hidden = true;
}

const logoutBtn = byId("control-logout");
if (logoutBtn) {
  logoutBtn.onclick = async () => {
    window.controlSigningOut = true;
    sessionStorage.removeItem("chat-token");
    sessionStorage.removeItem("chat-profile");
    try {
      await auth("logout", {});
    } finally {
      location.reload();
    }
  };
}

const loginForm = byId<HTMLFormElement>("control-login-form");
if (loginForm) {
  loginForm.onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();
    const submit = event.submitter as HTMLButtonElement | null;
    if (submit) submit.disabled = true;
    const errorEl = byId("control-login-error");
    if (errorEl) errorEl.textContent = "";
    try {
      const profileSelect = byId<HTMLSelectElement>("control-profile");
      const passwordInput = byId<HTMLInputElement>("control-password");
      const chosenProfileId = profileSelect ? profileSelect.value : "";
      const result = await auth<AuthResult>("login", {
        profile_id: chosenProfileId,
        password: passwordInput ? passwordInput.value : "",
      });
      if (chosenProfileId) {
        localStorage.setItem(LAST_PROFILE_KEY, chosenProfileId);
      }
      if (result.token) sessionStorage.setItem("chat-token", result.token);
      if (result.profile_id)
        sessionStorage.setItem("chat-profile", result.profile_id);
      if (passwordInput) passwordInput.value = "";
      await start();
    } catch (error) {
      if (errorEl) {
        errorEl.textContent =
          error instanceof Error ? error.message : String(error);
      }
    } finally {
      if (submit) submit.disabled = false;
    }
  };
}

function createProfileCard(
  profile: ProfileItem,
  isSelected: boolean,
  onSelect: () => void,
): HTMLElement {
  const card = document.createElement("button");
  card.type = "button";
  card.className = "control-profile-card" + (isSelected ? " active" : "");
  card.setAttribute("role", "radio");
  card.setAttribute("aria-checked", isSelected ? "true" : "false");
  card.setAttribute("aria-label", profile.name);
  card.dataset.profileId = profile.id;
  card.onclick = () => {
    onSelect();
  };

  card.onkeydown = (e: KeyboardEvent) => {
    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      e.preventDefault();
      const next = card.nextElementSibling as HTMLElement | null;
      if (next) {
        next.focus();
        next.click();
      }
    } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      e.preventDefault();
      const prev = card.previousElementSibling as HTMLElement | null;
      if (prev) {
        prev.focus();
        prev.click();
      }
    }
  };

  const wrap = document.createElement("div");
  wrap.className = "control-profile-avatar-wrap";

  const avatar = document.createElement("div");
  avatar.className = "control-profile-avatar";

  if (profile.avatar_url) {
    const img = document.createElement("img");
    img.className = "control-profile-avatar-img";
    img.src = profile.avatar_url;
    img.alt = profile.name;
    avatar.appendChild(img);
  } else {
    avatar.innerHTML =
      '<svg class="control-profile-avatar-icon" viewBox="0 0 24 24" width="34" height="34" fill="currentColor" aria-hidden="true"><path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z"/></svg>';
  }

  const badge = document.createElement("div");
  badge.className = "control-profile-badge";
  badge.setAttribute("aria-hidden", "true");
  badge.innerHTML =
    '<svg viewBox="0 0 20 20" width="12" height="12" fill="currentColor"><path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" /></svg>';

  wrap.appendChild(avatar);
  wrap.appendChild(badge);

  const name = document.createElement("span");
  name.className = "control-profile-name";
  name.textContent = profile.name;

  card.appendChild(wrap);
  card.appendChild(name);
  return card;
}

try {
  await start();
} catch {
  try {
    const profiles = await auth<ProfileItem[]>("profiles");
    const lastProfileId =
      localStorage.getItem(LAST_PROFILE_KEY) ||
      sessionStorage.getItem("chat-profile") ||
      "";

    // Lembrar a última opção clicada e colocá-la como primeira opção (na web somente)
    if (lastProfileId) {
      const lastIndex = profiles.findIndex((p) => p.id === lastProfileId);
      if (lastIndex > 0) {
        const [lastProfile] = profiles.splice(lastIndex, 1);
        profiles.unshift(lastProfile);
      }
    }

    const profileSelect = byId<HTMLSelectElement>("control-profile");
    if (profileSelect) {
      profileSelect.replaceChildren(
        ...profiles.map((profile) => new Option(profile.name, profile.id)),
      );
    }

    const cardsContainer = byId("control-profile-cards");
    let selectedId = profiles[0]?.id || "";
    if (profileSelect) {
      profileSelect.value = selectedId;
    }

    const selectProfile = (id: string, focusPassword = false) => {
      selectedId = id;
      localStorage.setItem(LAST_PROFILE_KEY, id);
      if (profileSelect) {
        profileSelect.value = id;
      }
      if (cardsContainer) {
        cardsContainer
          .querySelectorAll<HTMLElement>(".control-profile-card")
          .forEach((card) => {
            const active = card.dataset.profileId === id;
            card.classList.toggle("active", active);
            card.setAttribute("aria-checked", active ? "true" : "false");
          });
      }
      const errorEl = byId("control-login-error");
      if (errorEl) errorEl.textContent = "";
      if (focusPassword) {
        const passwordInput = byId<HTMLInputElement>("control-password");
        if (passwordInput) passwordInput.focus();
      }
    };

    if (cardsContainer) {
      cardsContainer.replaceChildren(
        ...profiles.map((profile) =>
          createProfileCard(profile, profile.id === selectedId, () =>
            selectProfile(profile.id, true),
          ),
        ),
      );
    }

    const passwordInput = byId<HTMLInputElement>("control-password");
    if (passwordInput) passwordInput.focus();
  } catch (error) {
    const errorEl = byId("control-login-error");
    if (errorEl) {
      errorEl.textContent =
        error instanceof Error ? error.message : String(error);
    }
  }
}

export {};
