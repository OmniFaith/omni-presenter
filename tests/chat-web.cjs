const { chromium } = require("playwright");
const assert = require("node:assert/strict");

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, colorScheme: "light", locale: "pt-BR" });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.setDefaultTimeout(15000);
  try {
    await page.goto(process.env.CHAT_URL + "/chat");
    await page.waitForFunction(() => document.querySelectorAll("#profile option").length === 2);
    assert.equal(await page.locator(".brand").count(), 0);
    assert.equal(await page.locator('a[href="/api/chat/docs"]').count(), 0);
    await page.locator("#profile").selectOption(process.env.CHAT_PROFILE);
    await page.locator("#code").fill(process.env.CHAT_CODE);
    await page.locator("#login-form button").click();
    await page.locator("#app").waitFor({ state: "visible" });
    await page.locator("#people button").filter({ hasText: "Mesa de som" }).waitFor();

    await page.locator("#song").click();
    await page.locator("#song-search").fill("cancao esperanca");
    await page.locator("#song-results button").filter({ hasText: "Canção de Esperança" }).click();
    await page.locator("#song-dialog").waitFor({ state: "hidden" });
    await page.locator("#timeline .shared").filter({ hasText: "Canção de Esperança" }).waitFor();
    const snapshot = await page.evaluate(async () => (await fetch("/api/chat/state?channel=general")).json());
    assert.ok(snapshot.messages.some((message) => message.kind === "song" && message.text.includes("Graça e paz")));

    await page.locator("#bible").click();
    await page.getByRole("button", { name: "Salmos", exact: true }).click();
    await page.waitForFunction(() => document.querySelector("#bible-stage").textContent === "Capítulo");
    await page.locator("#bible-results").getByRole("button", { name: "119", exact: true }).click();
    await page.waitForFunction(() => document.querySelector("#bible-stage").textContent === "Versículo");
    assert.equal(await page.locator("#bible-results button").count(), 176);
    await page.locator("#bible-results").getByRole("button", { name: "7", exact: true }).click();
    await page.locator("#bible-preview").waitFor({ state: "visible" });
    assert.match(await page.locator("#bible-preview").innerText(), /Salmos 119:7/);

    await page.locator("#bible-reference").fill("1 sa");
    await page.waitForFunction(() => document.querySelector("#bible-reference").value === "1 Samuel ");
    await page.locator("#bible-reference").fill("Atos 02");
    await page.waitForFunction(() => document.querySelector("#bible-reference").value === "Atos 2:");
    await page.locator("#bible-reference").fill("João 999:999");
    await page.waitForFunction(() => document.querySelector("#bible-results").hasChildNodes());
    assert.equal(await page.locator("#bible-send").isDisabled(), true);

    await page.locator("#bible-reference").fill("jo3.16-18");
    await page.waitForFunction(() => document.querySelector("#bible-preview").textContent.includes("João 3:16–18"));
    await page.locator("#bible-send").click();
    await page.locator("#bible-dialog").waitFor({ state: "hidden" });
    await page.locator("#timeline .shared").filter({ hasText: "João 3:16–18" }).waitFor();

    await page.locator("#bible").click();
    await page.locator("#bible-reference").fill("meu pastor");
    await page.locator("#bible-results button").filter({ hasText: "Salmos 23:1" }).click();
    await page.locator("#bible-preview").waitFor({ state: "visible" });
    assert.match(await page.locator("#bible-preview").innerText(), /Salmos 23:1/);
    await page.keyboard.press("Escape");

    await page.locator("#emoji").click();
    await page.locator("#emojis").waitFor({ state: "visible" });
    await page.keyboard.press("Escape");
    await page.locator("#attach").click({ trial: true });
    assert.equal(await page.evaluate(() => typeof document.querySelector("#attach").onclick), "function");
    assert.equal(await page.evaluate(() => typeof document.querySelector("#record").onclick), "function");

    const light = await page.locator("#workspace-header").evaluate((element) => ({ radius: getComputedStyle(element).borderRadius, color: getComputedStyle(element).backgroundColor }));
    assert.equal(light.radius, "10px");
    assert.equal(light.color, "rgb(255, 255, 255)");
    if (process.env.CHAT_SCREENSHOT_DIR) await page.screenshot({ path: process.env.CHAT_SCREENSHOT_DIR + "/chat-light.png" });
    await page.emulateMedia({ colorScheme: "dark" });
    assert.equal(await page.locator("#workspace-header").evaluate((element) => getComputedStyle(element).backgroundColor), "rgb(22, 24, 30)");
    if (process.env.CHAT_SCREENSHOT_DIR) await page.screenshot({ path: process.env.CHAT_SCREENSHOT_DIR + "/chat-dark.png" });
    await page.setViewportSize({ width: 390, height: 844 });
    await page.locator("#menu").click();
    await page.locator("#people button").filter({ hasText: "Mesa de som" }).click();
    await page.locator("#song").click();
    await page.locator("#song-results button").filter({ hasText: "Canção de Esperança" }).click();
    await page.locator("#song-dialog").waitFor({ state: "hidden" });
    await page.locator("#timeline .shared").filter({ hasText: "Canção de Esperança" }).waitFor();
    await page.locator("#bible").click();
    await page.locator("#bible-reference").fill("Sl119:01-07");
    await page.waitForFunction(() => document.querySelector("#bible-preview").textContent.includes("Salmos 119:1–7"));
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    const submit = await page.locator("#bible-send").boundingBox();
    assert.ok(submit.y >= 0 && submit.y + submit.height <= 844, "The mobile Bible send button must stay visible");
    if (process.env.CHAT_SCREENSHOT_DIR) await page.screenshot({ path: process.env.CHAT_SCREENSHOT_DIR + "/chat-mobile-bible.png" });
    assert.deepEqual(errors, [], "Every chat script must initialize without runtime errors");
    console.log("Browser verified: profiles, lyrics in channels/private chat, Bible stages/ranges/text, emoji, light/dark themes, mobile layout.");
  } catch (error) {
    console.error("Browser errors:", errors);
    console.error("Chat status:", await page.locator("#status").textContent());
    if (process.env.CHAT_SCREENSHOT_DIR) await page.screenshot({ path: process.env.CHAT_SCREENSHOT_DIR + "/chat-failure.png" });
    throw error;
  } finally {
    await browser.close();
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
