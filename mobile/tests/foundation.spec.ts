import { expect, test } from "@playwright/test";

test.describe("mobile foundation", () => {
  test("boots a usable, accessible readiness shell at phone dimensions", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("heading", { name: "Your day, held together." })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Mobile foundation" })).toBeVisible();

    const checklist = page.getByRole("list");
    await expect(checklist).toBeVisible();
    await expect(checklist.getByRole("listitem")).toHaveText([
      "Shared Rust core",
      "Local encrypted data",
      "Foreground and resume sync"
    ]);

    await expect(page.getByRole("status")).toHaveText(
      "Daily flows unlock after the shared core is connected."
    );

    const viewport = page.viewportSize();
    expect(viewport).not.toBeNull();
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(
      viewport!.width
    );

    const shell = page.locator("main.mobile-shell");
    const status = page.getByRole("status");
    await expect(shell).toBeVisible();
    await expect(status).toBeInViewport();

    const statusBox = await status.boundingBox();
    expect(statusBox).not.toBeNull();
    expect(statusBox!.x).toBeGreaterThanOrEqual(0);
    expect(statusBox!.x + statusBox!.width).toBeLessThanOrEqual(viewport!.width);
  });
});
