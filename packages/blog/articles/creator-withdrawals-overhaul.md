---
title: More Ways to Withdraw
summary: 'New withdraw options and a redesigned revenue dashboard'
date: 2025-10-27T16:30:00-07:00
authors: ['AJfd8YH6', 'bOHH0P9Z', 'xSQqYYIN']
---

**Edited November 4, 2025** to correct information about international PayPal withdrawal fees

---

Hey everyone! We've heard your feedback on creator withdrawal options, and we're excited to share that more are finally coming to Modrinth! Over the past month, we've been working on overhauling the withdrawal experience and supporting more withdraw methods.

There's a lot packed into this release, so let's dive in!

<div class="video-wrapper mb-8">
	<video autoplay loop muted playsinline>
		<source src="./revenue-page-from-home.mp4" type="video/mp4" />
	</video>
</div>

## **TL;DR**

- New revenue page and withdrawal flow
- New bank transfer option for 29 countries
- New crypto (USDC) option for hard to reach countries
- PayPal and Venmo moved to Tremendous for international users (lower to no FX fees)
- Email notifications for payout updates (available, successful, failed, etc.)
- New withdrawal compliance

---

## Streamlined Withdrawal Experience

A big goal for this release was to make payouts as clear as possible for creators. The revenue screen and withdrawal process has been completely overhauled.

**Your revenue is now displayed as a balance, with a detailed breakdown below.**

<div class="video-wrapper mb-8">
	<video autoplay loop muted playsinline>
		<source src="./balance-progress-bar.mp4" type="video/mp4" />
	</video>
</div>

> Green shows available revenue, blue and purple show estimated revenue from the last two completed months, and grey shows the current month revenue that's still being processed.

We've also cleaned up the transaction history. You'll now see your most recent transactions directly on the revenue page, along with any deposits from Modrinth into your account.

## New Withdraw Methods

The second big goal of this release was to bring more options, better fees, and new ways to withdraw from countries that were previously hard to reach.

<div class="video-wrapper mb-8">
	<video autoplay loop muted playsinline>
		<source src="./withdraw-example.mp4" type="video/mp4" />
	</video>
</div>

Withdrawing with ease from your balance using our new modal flow. Click the green Withdraw button to get started, and you'll immediately see all available methods and fees for your country, including a few new ones:

### Bank Transfers (29 Countries)

You can now withdraw directly to your bank account via wire or ACH, depending on your country. Supported countries and their fees are listed below.

We're working to expand this list, most notably to Canada, the UK, and countries across Asia, but there's no timeline _yet_.

| Country         | Currency | Transaction Fee | FX Fee            | **Total Fee**     |
| --------------- | -------- | --------------- | ----------------- | ----------------- |
| 🇺🇸 USA          | USD      | $0.50 + 1%      | 0.50%             | **~1.5% + $0.50** |
| 🇪🇺 EU           | EUR      | €1.00 + 1%      | 0.60%             | **~1.6% + €1.00** |
| 🇦🇷 Argentina    | ARS      | $0.00 + 1%      | 0.50%             | **~1.5%**         |
| 🇲🇽 Mexico       | MXN      | $0.50 + 1%      | 0.90%             | **~1.9% + $0.50** |
| 🇧🇷 Brazil       | BRL      | $0.25 + 1%      | 1.30% (incl. IOF) | **~2.3% + $0.25** |
| 🇨🇱 Chile        | CLP      | $1.20 + 1%      | 0.95%             | **~2.0% + $1.20** |
| 🇨🇷 Costa Rica   | CRC      | $0.80 + 1%      | 1.05%             | **~2.1% + $0.80** |
| 🇵🇪 Peru (PEN)   | PEN      | $1.00 + 1%      | 1.15%             | **~2.1% + $1.00** |
| 🇿🇦 South Africa | ZAR      | $1.50 + 1%      | 1.40%             | **~2.4% + $1.50** |
| 🇵🇪 Peru (USD)   | USD      | $5.00 + 1%      | 0.50%             | **~1.5% + $5.00** |
| 🇨🇴 Colombia     | COP      | $0.35 + 1%      | 0.95% (incl. GMF) | **~2.0% + $0.35** |

_Total Fee includes both transaction and FX fees._

> **Supported EU countries:** Austria, Belgium, Cyprus, Estonia, Finland, France, Germany, Greece, Ireland, Italy, Latvia, Lithuania, Luxembourg, Malta, Netherlands, Portugal, Slovakia, Spain

### Crypto Withdrawals (USDC)

We've also added USDC withdrawals on the Polygon network. This option is available worldwide, so everyone can now withdraw funds. Fees are a flat 1% + network fees, making it a great low-cost option.

### PayPal & Venmo

We've moved PayPal and Venmo to two different methods depending on your country:

- **United States:** PayPal and Venmo will remain on the existing system with the same low fees: $0.25 + 2%, capped at $1.00.
- **Outside of the United States:** PayPal and Venmo have been moved to Tremendous, which has zero FX fees and charges approximately 3.84% on the total amount. This should be an improvement over the old method, which had hidden FX fees that could end up much higher.

## New Email Notifications

Earlier this month, we quietly rolled out a new email system that lets us give both creators and users better feedback about what's going on.

This includes emails for things like project status changes, payouts available, and successful withdrawals. We plan to keep building on this and will be adding email preference settings soon!

## Tax Compliance

Last but not least, we're also rolling out our new tax compliance system. We partially rolled this out earlier this month, but the full version is now live.

Creator withdrawals will now be limited to $600.00 USD per calendar year as of 2025, per U.S. regulations. As a creator approaches that threshold, they'll be prompted to fill out a W-8 or W-9 tax form, depending on their country.

![A snippet of the tax form stage of the new withdraw modal.](./tax-compliance.png)

Completing this form helps Modrinth stay compliant and will automatically unlock withdrawals again once submitted. For non-US users, these details are not automatically sent to the US government! They are for our own records in the case of an audit and we need to prove where we're sending money to.

---

Thank you to all the creators and players supporting Modrinth, we hope you enjoy this long deserved update! 💚
