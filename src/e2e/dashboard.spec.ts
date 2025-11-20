import { test, expect } from '@playwright/test'

test.describe('Dashboard E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should display dashboard page', async ({ page }) => {
    await expect(page.locator('h2')).toContainText('Dashboard')
  })

  test('should navigate to strategies page', async ({ page }) => {
    await page.click('text=Strategies')
    await expect(page).toHaveURL('/strategies')
  })

  test('should navigate to backtest page', async ({ page }) => {
    await page.click('text=Backtest')
    await expect(page).toHaveURL('/backtest')
  })

  test('should navigate to trading page', async ({ page }) => {
    await page.click('text=Trading')
    await expect(page).toHaveURL('/trading')
  })

  test('should navigate to risk page', async ({ page }) => {
    await page.click('text=Risk')
    await expect(page).toHaveURL('/risk')
  })

  test('should navigate to settings page', async ({ page }) => {
    await page.click('text=Settings')
    await expect(page).toHaveURL('/settings')
  })
})
