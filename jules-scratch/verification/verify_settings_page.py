from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_settings_page(page: Page):
    """
    This script verifies the new look of the settings page by navigating from the home page.
    """
    try:
        # Navigate to the home page
        page.goto("http://localhost:1420/")

        # Find and click the settings link in the sidebar header
        settings_link = page.get_by_role("link", name="settings")
        expect(settings_link).to_be_visible(timeout=10000)
        settings_link.click()

        # Wait for the settings page heading to be visible
        heading = page.get_by_role("heading", name="ショートカット設定")
        expect(heading).to_be_visible(timeout=10000)

        # Wait a bit for animations or late-loading elements
        time.sleep(1)

        # Take a screenshot
        page.screenshot(path="jules-scratch/verification/verification.png")
        print("Screenshot taken successfully.")

    except Exception as e:
        print(f"An error occurred: {e}")
        # Save page source for debugging
        with open("page_source.html", "w", encoding="utf-8") as f:
            f.write(page.content())
        print("Page source saved to page_source.html")
        raise

# This is the boilerplate to run the script
with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page()
    verify_settings_page(page)
    browser.close()
