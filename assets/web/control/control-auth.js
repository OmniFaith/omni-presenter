const byId = (id) => document.getElementById(id);
document.documentElement.dataset.theme = localStorage.getItem("control-theme") || (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
async function auth(path, body) {
  const response = await fetch("/api/control/" + path, body ? {
    method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body),
  } : {});
  const result = await response.json();
  if (!response.ok) throw new Error(result.error || "Não foi possível entrar.");
  return result;
}
async function start() {
  await auth("session");
  await import("/control.js");
  document.body.dataset.locked = "false";
  byId("control-login").hidden = true;
}
byId("control-logout").onclick = async () => {
  window.controlSigningOut = true;
  sessionStorage.removeItem("chat-token");
  sessionStorage.removeItem("chat-profile");
  try { await auth("logout", {}); } finally { location.reload(); }
};
byId("control-login-form").onsubmit = async (event) => {
  event.preventDefault();
  const submit = event.submitter;
  submit.disabled = true;
  byId("control-login-error").textContent = "";
  try {
    const result = await auth("login", { profile_id: byId("control-profile").value, password: byId("control-password").value });
    if (result.token) sessionStorage.setItem("chat-token", result.token);
    if (result.profile_id) sessionStorage.setItem("chat-profile", result.profile_id);
    byId("control-password").value = "";
    await start();
  } catch (error) { byId("control-login-error").textContent = error.message; }
  finally { submit.disabled = false; }
};
try { await start(); } catch {
  try {
    const profiles = await auth("profiles");
    byId("control-profile").replaceChildren(...profiles.map((profile) => new Option(profile.name, profile.id)));
    byId("control-password").focus();
  } catch (error) { byId("control-login-error").textContent = error.message; }
}
