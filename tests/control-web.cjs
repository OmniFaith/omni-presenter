const { chromium, devices } = require("playwright");
const assert = require("node:assert/strict");

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: 1600, height: 1000 },
    locale: "pt-BR",
  });
  page.setDefaultTimeout(15000);
  const errors = [],
    missing = [],
    base = process.env.CONTROL_URL;
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => {
    if (response.status() === 404) missing.push(response.url());
  });
  const row = (title) =>
    page.locator("#library-items .resource-row").filter({ hasText: title });
  const state = () =>
    page.evaluate(async () => (await fetch("/api/control/state")).json());
  const waitTitle = (type, title) =>
    page.waitForFunction(
      ({ type, title }) =>
        document.getElementById(type + "-title").textContent === title,
      { type, title },
    );
  const screenshot = async (name, target = page) => {
    if (!process.env.CONTROL_SCREENSHOT_DIR) return;
    await target.waitForTimeout(600);
    return target.screenshot({
      path: process.env.CONTROL_SCREENSHOT_DIR + "/" + name + ".png",
    });
  };
  async function login(target) {
    await target.locator("#control-login").waitFor({ state: "visible" });
    await target.locator("#control-profile option").first().waitFor({ state: "attached" });
    await target.locator("#control-password").fill("omni-presenter");
    await target.locator('#control-login-form button[type="submit"]').click();
    await target.locator("#control-login").waitFor({ state: "hidden" });
  }
  async function checkNativeGeometry(layout) {
    const root = await page.locator("#native-layout").boundingBox();
    const scale = (await state()).scale / 100;
    async function visit(value, rect) {
      if (value.Leaf) {
        const actual = await page
          .locator(`[data-leaf="${value.Leaf}"]`)
          .boundingBox();
        for (const [key, expected] of Object.entries({
          x: rect.x + 2 * scale,
          y: rect.y + 2 * scale,
          width: rect.width - 4 * scale,
          height: rect.height - 4 * scale,
        })) {
          assert.ok(
            Math.abs(actual[key] - expected) < 4,
            `${value.Leaf}.${key} must follow the native layout: ${actual[key]} vs ${expected}`,
          );
        }
        return;
      }
      const split = value.Split;
      if (split.direction === "Horizontal") {
        const size = rect.width * split.fraction;
        await visit(split.first, { ...rect, width: size });
        await visit(split.second, {
          ...rect,
          x: rect.x + size + 6 * scale,
          width: rect.width - size - 6 * scale,
        });
      } else {
        const size = rect.height * split.fraction;
        await visit(split.first, { ...rect, height: size });
        await visit(split.second, {
          ...rect,
          y: rect.y + size + 6 * scale,
          height: rect.height - size - 6 * scale,
        });
      }
    }
    await visit(layout.root, root);
  }
  try {
    await page.goto(base + "/control");
    for (const path of ["/api/control/state", "/api/control/library", "/api/state/__control_preview", "/screen/__control_live"]) {
      assert.equal((await page.request.get(base + path)).status(), 401, path + " requires a session");
    }
    await page.locator("#control-password").fill("senha-incorreta");
    await page.locator('#control-login-form button[type="submit"]').click();
    await page.locator("#control-login-error").filter({ hasText: "senha" }).waitFor();
    await login(page);
    await page.waitForFunction(
      () => document.getElementById("connection").textContent === "Conectado",
    );
    await row("Canção de Esperança").waitFor();
    await page.evaluate(() => document.fonts.ready);
    assert.equal(
      await page.locator(".app-heading h1").textContent(),
      "Atalhos",
    );
    assert.equal(await page.locator("#native-layout [data-leaf]").count(), 7);
    assert.equal(await page.locator("#player-panel").isVisible(), true);
    assert.equal(
      await page.evaluate(() => document.body.dataset.lyricsMode),
      "list",
    );
    await checkNativeGeometry((await state()).layout);

    const initial = (await state()).layout.root.Split.fraction;
    const handle = await page.locator('[data-split-path="root"]').boundingBox();
    await page.mouse.move(handle.x + handle.width / 2, handle.y + 180);
    await page.mouse.down();
    await page.mouse.move(handle.x + 45, handle.y + 180, { steps: 5 });
    await page.mouse.up();
    await page.waitForFunction(
      async (previous) =>
        (await (await fetch("/api/control/state")).json()).layout.root.Split
          .fraction >
        previous + 0.01,
      initial,
    );
    const target = await page.locator("#playlist-panel").boundingBox();
    await page
      .locator("#library-panel .panel-heading")
      .dragTo(page.locator("#playlist-panel"), {
        targetPosition: { x: target.width / 2, y: target.height / 2 },
      });
    await page.waitForFunction(
      async () =>
        (await (await fetch("/api/control/state")).json()).layout.root.first
          ?.Leaf === "Playlist" ||
        (await (await fetch("/api/control/state")).json()).layout.root.Split
          .first.Leaf === "Playlist",
    );
    await page.locator("#layout-reset").click();
    await page
      .getByRole("button", { name: "Restaurar painéis", exact: true })
      .click();
    await page.getByRole("button", { name: "Fechar", exact: true }).click();
    await page.waitForFunction(
      async () =>
        (await (await fetch("/api/control/state")).json()).layout.root.Split
          .first.Leaf === "Selection",
    );
    await checkNativeGeometry((await state()).layout);

    await row("Canção de Esperança").locator(".resource-title").click();
    await page.locator("#project-preview").click();
    await page.waitForFunction(
      () => document.body.dataset.presenting === "true",
    );
    await page.locator('[data-direction="next"]').click();
    await page.waitForFunction(
      () => document.getElementById("live-count").textContent === "2 / 4",
    );
    await page.locator("#native-slide-jump").fill("3");
    await page.locator("#native-slide-jump").press("Enter");
    await page.waitForFunction(
      () => document.getElementById("live-count").textContent === "3 / 4",
    );
    await row("Próxima música").locator(".resource-title").click();
    await waitTitle("preview", "Próxima música");
    assert.equal(
      await page.locator("#live-title").textContent(),
      "Canção de Esperança",
    );
    await page
      .locator("#slides .slide-card")
      .nth(1)
      .locator(".project-slide")
      .click();
    await waitTitle("live", "Próxima música");
    assert.equal((await state()).live.index, 1);
    await page.locator("#hide-toggle").click();
    await page.waitForFunction(
      () =>
        document.getElementById("live-status").textContent === "Texto oculto",
    );
    await page.locator("#hide-toggle").click();
    await page.locator("#stop").click();
    await page.waitForFunction(
      () => document.body.dataset.presenting === "false",
    );

    for (const name of ["Canção de Esperança", "Próxima música"])
      await row(name)
        .getByRole("button", { name: "Adicionar ao repertório", exact: true })
        .click();
    await page.waitForFunction(
      () => document.querySelectorAll("#playlist .playlist-row").length === 2,
    );
    await page
      .locator("#playlist .playlist-row")
      .first()
      .click({ button: "right" });
    await page
      .getByRole("menuitem", { name: "Mover para baixo", exact: true })
      .click();
    await page.waitForFunction(() =>
      document
        .querySelector("#playlist .resource-title")
        .textContent.includes("Próxima música"),
    );
    await page
      .locator("#playlist .playlist-row")
      .first()
      .locator(".resource-title")
      .click();
    await page
      .locator("#playlist-panel > .native-toolbar")
      .getByRole("button", { name: "Apresentar", exact: true })
      .click();
    await waitTitle("live", "Próxima música");
    await page
      .locator("#playlist .playlist-row")
      .nth(1)
      .locator(".resource-title")
      .dblclick();
    await waitTitle("live", "Canção de Esperança");
    await row("Canção de Esperança")
      .getByRole("button", { name: "Fixar", exact: true })
      .click();
    await page
      .locator("#quick-items button")
      .filter({ hasText: "Canção de Esperança" })
      .waitFor();
    await screenshot("control-native-home");

    await page.locator('.app-rail [data-nav-kind="bible"]').click();
    await page
      .locator("#native-books")
      .getByRole("button", { name: "Salmos", exact: true })
      .click();
    await page
      .locator("#native-books .native-book")
      .filter({
        has: page.getByRole("button", { name: "Salmos", exact: true }),
      })
      .getByRole("button", { name: "119", exact: true })
      .click();
    await page.waitForFunction(
      () => document.querySelectorAll("#native-verses button").length === 176,
    );
    await page.locator("#native-bible-search-toggle").click();
    await page.locator("#bible-query").fill("jo3.16-18");
    await page.waitForFunction(() =>
      document
        .getElementById("bible-passage")
        .textContent.includes("João 3:16–18"),
    );
    await page.locator("#bible-query").press("Enter");
    await waitTitle("preview", "João 3");
    assert.equal(
      await page.locator("#live-title").textContent(),
      "Canção de Esperança",
    );
    await page.locator("#bible-project").click();
    await waitTitle("live", "João 3");
    assert.equal((await state()).live.slides.length, 3);
    await page.locator("#bible-add").click();
    await page.waitForFunction(
      () => document.querySelectorAll("#playlist .playlist-row").length === 3,
    );
    const original = (await state()).live.theme.id;
    const next = (await state()).themes.find((theme) => theme.id !== original);
    await page
      .locator("#themes")
      .getByTitle(next.name, { exact: true })
      .click();
    await page.waitForFunction(
      (id) => document.querySelector("#themes .active").dataset.id === id,
      next.id,
    );
    assert.equal((await state()).live.theme.id, original);
    await page
      .locator("#themes")
      .getByTitle(next.name, { exact: true })
      .click({ button: "right" });
    await page
      .getByRole("menuitem", { name: "Aplicar ao vivo", exact: true })
      .click();
    await page.waitForFunction(
      async (id) =>
        (await (await fetch("/api/control/state")).json()).live.theme.id === id,
      next.id,
    );
    await screenshot("control-native-bible");

    await page.locator("#notice-open").click();
    await page.locator("#text-title").fill("Aviso da reunião");
    await page.locator("#text-content").fill("Primeiro aviso\n\nSegundo aviso");
    await page.locator('#text-form button[value="project"]').click();
    await page.locator("#text-dialog").waitFor({ state: "hidden" });
    await waitTitle("live", "Aviso da reunião");
    await page.locator(".app-rail .rail-home").click();
    await page.locator("#song-new").click();
    await page.locator("#song-title").fill("Música nova");
    await page.locator("#song-artist").fill("Equipe");
    await page.locator("#song-content").fill("Letra nova\n\nNovo final");
    await page
      .locator("#song-form")
      .getByRole("button", { name: "Salvar", exact: true })
      .click();
    await page.locator("#song-dialog").waitFor({ state: "hidden" });
    await waitTitle("preview", "Música nova");
    assert.equal(
      await page.locator("#live-title").textContent(),
      "Aviso da reunião",
    );

    await page.locator(".app-rail .rail-home").click();
    await page.locator('[data-kind="text"]').click();
    await page
      .locator("#library-items .folder-row")
      .getByText("Avisos", { exact: true })
      .click();
    await row("Bem-vindos").locator(".resource-title").click();
    await waitTitle("preview", "Bem-vindos");
    await page.locator('[data-kind="media"]').click();
    await row("Imagem do encontro")
      .getByRole("button", { name: "Apresentar", exact: true })
      .click();
    await waitTitle("live", "Imagem do encontro");
    await page.locator('[data-kind="design"]').click();
    await row("Cronômetro")
      .getByRole("button", { name: "Apresentar", exact: true })
      .click();
    await page.locator("#design-controls").waitFor({ state: "visible" });
    await page.locator('[data-design-op="pause"]').click();
    await page.waitForFunction(
      () =>
        document.querySelector("#timers .timer").dataset.running === "false",
    );
    await page.locator('[data-design-op="start"]').click();
    await page.waitForFunction(
      () => document.querySelector("#timers .timer").dataset.running === "true",
    );

    const touchContext = await browser.newContext({
      ...devices["Pixel 7"],
      locale: "pt-BR",
    });
    const touch = await touchContext.newPage();
    touch.on("pageerror", (error) => errors.push(error.message));
    await touch.goto(base + "/control");
    await login(touch);
    await touch.waitForFunction(
      () => document.getElementById("connection").textContent === "Conectado",
    );
    await touch.locator('.mobile-nav [data-mobile-kind="song"]').tap();
    await touch.locator("#library-search").fill("cancao esperanca");
    await touch.waitForFunction(
      () =>
        document.querySelectorAll("#library-items .resource-row").length === 1,
    );
    await touch.locator("#library-items .resource-title").tap();
    await touch.waitForFunction(
      () => document.body.dataset.mobileMonitor === "preview",
    );
    await touch.locator("#project-preview").tap();
    await touch.waitForFunction(
      () => document.body.dataset.mobileMonitor === "live",
    );
    await touch.locator('[data-direction="next"]').tap();
    await touch.waitForFunction(
      () => document.getElementById("live-count").textContent === "2 / 4",
    );
    const transport = await touch.locator(".performance-bar").boundingBox();
    assert.ok(transport.y + transport.height < touch.viewportSize().height);
    assert.equal(
      await touch.evaluate(
        () => document.documentElement.scrollHeight <= innerHeight + 1,
      ),
      true,
    );
    await screenshot("control-native-mobile", touch);
    await touch.locator('.mobile-nav [data-mobile-kind="bible"]').tap();
    await touch.locator("#native-bible-search-toggle").tap();
    await touch.locator("#bible-query").fill("João 3:16-18");
    await touch.waitForFunction(
      () => !document.getElementById("bible-project").disabled,
    );
    await touch.locator("#bible-query").press("Enter");
    await touch.locator("#bible-project").tap();
    await touch.waitForFunction(
      () => document.getElementById("live-title").textContent === "João 3",
    );
    await touch.locator("#mobile-more-open").tap();
    await touch
      .locator("#mobile-quick-items button")
      .filter({ hasText: "Canção de Esperança" })
      .tap();
    await touch.waitForFunction(
      () =>
        document.getElementById("preview-title").textContent ===
        "Canção de Esperança",
    );
    assert.equal(await touch.locator("#live-title").textContent(), "João 3");
    await touchContext.close();
    await Promise.all([
      page.waitForNavigation({ waitUntil: "domcontentloaded" }),
      page.locator("#control-logout").click(),
    ]);
    await page.locator("#control-login").waitFor({ state: "visible" });
    assert.equal((await page.request.get(base + "/api/control/state")).status(), 401);
    assert.deepEqual(errors, []);
    assert.deepEqual(missing, []);
    console.log(
      "Native layout geometry, fonts, controls, live/preview separation, setlist, Bible, media, designs and touch operation verified.",
    );
  } catch (error) {
    console.error("Browser failure:", error);
    console.error("Runtime errors:", errors, "Missing:", missing);
    console.error("UI error:", await page.locator("#error").textContent({ timeout: 1000 }).catch(() => "Page is navigating or not initialized"));
    await screenshot("control-failure");
    throw error;
  } finally {
    await browser.close();
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
