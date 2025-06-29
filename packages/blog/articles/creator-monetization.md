---
title: Creators can now make money on Modrinth!
summary: "Yes, you read the title correctly: Modrinth's creator monetization program, also known as payouts, is now in an open beta phase. Read on for more information!"
date: 2022-11-12
---

Yes, you read the title correctly: Modrinth's Creator Monetization Program, also known as payouts, is now in an open beta phase. All of the money that project owners have earned since August 1st is available to claim **right now**!

This includes even projects other than mods! Modpacks, plugins, and resource packs also generate payouts for creators to claim.

Alongside this, the frontend also got a facelift across the entire site, most notably on the settings, notifications, and user profile pages.

## Motivation

Since the start, Modrinth has been a platform created by Minecraft content creators for Minecraft content creators. Allowing creators to earn a bit of money for their hard work and dedication to their content has been a goal since we started, and we are so incredibly ecstatic to finally be able to release this for everyone.

Whether it's used for buying coffee, paying for server costs, or to get that luxury pair of socks, we hope that creators will be able to use their payouts on whatever keeps them running. We want to encourage creators to keep making content for everyone to enjoy, with the hope that everyone will eventually be able to call Modrinth their go-to destination for Minecraft modded content.

## How it works

For every project uploaded to Modrinth, we keep track of its page views and downloads through an internal system we call [ariadne](https://github.com/modrinth/ariadne). Through our payouts algorithm ([source code](https://github.com/modrinth/labrinth/blob/master/src/routes/admin.rs#L95)), we distribute 100% of ad revenue earned from creator pages to the creators behind these projects. Project owners can decide how to split it (or how not to split it) between their team members.

Modpacks are a bit different, with revenue split 80% to the Modrinth dependencies on the pack and 20% to the modpack author. This split is subject to change and will be evaluated periodically to ensure the split is reasonably fair.

After taking the search pages into account, around 10% of the site's ad revenue ends up going to us, mainly to cover hosting and personnel expenses, and 90% to creators.

While payouts will be small at first, we're working on improving our ads system to better fund the program. We've also got big projects coming soon to continue our trajectory of making the monetization program and the site better!

## How do I earn money?

When a project of yours on Modrinth gets approved, you are automatically enrolled into the program. You will start to incur a balance, which you can view from the [Monetization dashboard](https://modrinth.com/dashboard). You can claim your first payout via PayPal or Venmo as soon as you enter your credentials and have the minimum balance of 0.26 USD.

Even though the minimum is low, you will want to wait some time to allow your balance to build up before claiming. Each payment processor has its own fees which depend upon whether you're within the United States, which are detailed on the dashboard's [revenue tab](https://modrinth.com/dashboard/revenue).

Once you request a transfer, you may have to confirm the transfer via email if you don't already have a PayPal account. If you do not confirm using the link in the email within 30 days, or the transfer fails for whatever reason, the amount requested will be returned to your Modrinth balance, though the processor's fees may already have been deducted by that point.

### For residents outside the United States

Since Modrinth is a US-based company, all amounts are stored, displayed, and paid out in US dollars. PayPal will convert the amount to your local currency once you begin the process of transferring from your Modrinth balance to your PayPal account.

We're aware of some extenuating circumstances for creators living in areas affected by geopolitical conflict. As such, we are looking into alternative ways to allow payouts to continue in these regions.

At the moment, there are no mechanisms in place to make your Modrinth balance expire after some time, though this is likely to be added in the future for creators who do not claim their balance after several years. Rest assured, we will have processes in place to make sure that your money doesn't go poof just because you weren't able to claim it in time.

## Frontend facelift

The website frontend has had some "small" changes of around 12,322 lines of code to accommodate payouts and many other changes. Many of these changes were inspired by the experiments done on the SvelteKit Rewrite, progress on which is paused for the time being. Navigate around the main site for a bit to discover some of these changes! Highlights include:

- Improved project creation and report filing workflow via modals
- Improved 404 page
- Deduplicate identical version changelogs
- Cleaner user profile pages
- Easier to navigate settings and notifications
- Spacing, font, and accessibility tweaks
- And plenty more!

## Conclusion

This is a jam-packed update, and it would be impossible to list all the changes in this post. Feel free to explore the site, claim your funds, and give us feedback on [Discord](https://discord.modrinth.com). If you suspect you've found any critical bugs or exploits, please email us immediately at [support@modrinth.com](mailto:support@modrinth.com) - otherwise, for non-critical bugs, report them [on GitHub](https://github.com/modrinth).

👑
