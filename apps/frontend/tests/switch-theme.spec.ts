import { expect, test, type Page } from '@playwright/test'

async function hasThemeOnElement(page: Page, element: string) {
	const htmlElement = page.locator('html')
	return await htmlElement.evaluate((el, theme) => el.classList.contains(theme), element)
}

test('test', async ({ page }, testInfo) => {
	await page.goto('http://localhost:3000/')
	await page.getByRole('link', { name: 'Settings' }).click()
	await page.locator('html').click()

	await page.getByRole('button', { name: 'Light' }).click()
	const lightMode = await page.screenshot()
	expect(await hasThemeOnElement(page, 'light-mode')).toBe(true)

	await page.waitForTimeout(1000)

	await page.getByRole('button', { name: 'Dark' }).click()
	const darkMode = await page.screenshot()
	expect(await hasThemeOnElement(page, 'dark-mode')).toBe(true)

	await page.waitForTimeout(1000)

	await page.getByRole('button', { name: 'OLED' }).click()
	const oledMode = await page.screenshot()
	expect(await hasThemeOnElement(page, 'oled-mode')).toBe(true)

	testInfo.attach('test-results/light-mode', { body: lightMode, contentType: 'image/png' })
	testInfo.attach('test-results/dark-mode', { body: darkMode, contentType: 'image/png' })
	testInfo.attach('test-results/oled-mode', { body: oledMode, contentType: 'image/png' })
})
