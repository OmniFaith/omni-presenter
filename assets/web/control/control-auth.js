const byId = (id) => document.getElementById(id);
document.documentElement.dataset.theme =
    localStorage.getItem("control-theme") ||
        (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
async function auth(path, body) {
    const response = await fetch("/api/control/" + path, body
        ? {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        }
        : {});
    const result = (await response.json());
    if (!response.ok)
        throw new Error(result.error || "Não foi possível entrar.");
    return result;
}
async function start() {
    await auth("session");
    await import("/control.js");
    document.body.dataset.locked = "false";
    const login = byId("control-login");
    if (login)
        login.hidden = true;
}
const logoutBtn = byId("control-logout");
if (logoutBtn) {
    logoutBtn.onclick = async () => {
        window.controlSigningOut = true;
        sessionStorage.removeItem("chat-token");
        sessionStorage.removeItem("chat-profile");
        try {
            await auth("logout", {});
        }
        finally {
            location.reload();
        }
    };
}
const loginForm = byId("control-login-form");
if (loginForm) {
    loginForm.onsubmit = async (event) => {
        event.preventDefault();
        const submit = event.submitter;
        if (submit)
            submit.disabled = true;
        const errorEl = byId("control-login-error");
        if (errorEl)
            errorEl.textContent = "";
        try {
            const profileSelect = byId("control-profile");
            const passwordInput = byId("control-password");
            const result = await auth("login", {
                profile_id: profileSelect ? profileSelect.value : "",
                password: passwordInput ? passwordInput.value : "",
            });
            if (result.token)
                sessionStorage.setItem("chat-token", result.token);
            if (result.profile_id)
                sessionStorage.setItem("chat-profile", result.profile_id);
            if (passwordInput)
                passwordInput.value = "";
            await start();
        }
        catch (error) {
            if (errorEl) {
                errorEl.textContent =
                    error instanceof Error ? error.message : String(error);
            }
        }
        finally {
            if (submit)
                submit.disabled = false;
        }
    };
}
try {
    await start();
}
catch {
    try {
        const profiles = await auth("profiles");
        const profileSelect = byId("control-profile");
        if (profileSelect) {
            profileSelect.replaceChildren(...profiles.map((profile) => new Option(profile.name, profile.id)));
        }
        const passwordInput = byId("control-password");
        if (passwordInput)
            passwordInput.focus();
    }
    catch (error) {
        const errorEl = byId("control-login-error");
        if (errorEl) {
            errorEl.textContent =
                error instanceof Error ? error.message : String(error);
        }
    }
}
export {};
