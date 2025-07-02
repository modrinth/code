---
title: Creator Updates, July 2025
summary: Addressing recent growth and growing pains that have been affecting creators.
date: 2025-07-01T21:20:00-07:00
---

Hey all,

The last few months have been quite hectic for Modrinth. We've experienced all-time highs in both traffic and new creators and have outgrown a lot of our existing systems, which has led to a lot of issues plaguing creators, especially.

The team has been super hard at work at this, and I'm really glad to announce that we've fixed most of these issues long term.

1. **Upload issues (inputs not showing up, instability, etc)**

   We've tracked these issues down to conflicting code between our ad provider and Modrinth's. For now, we've **disabled ads for all logged in users across the site** while we work on resolving these long term. Both web users and logged-in web users make a very small percentage of our ad revenue (7% for web and 0.05% for logged-in web users) so creators should see a very minimal revenue drop from this, and have a much better experience navigating and uploading to the site.

2. **Moderation and report response times**

   Creators have had to wait, in some cases, weeks to get their projects reviewed. This is unacceptable on our part and we are actively overhauling our moderation tooling to improve the moderation experience (and lowering time spent per project). We've also hired 3 additional moderators/support staff (**bringing our total to 7 and the total team to 17 people!**). We're hoping to see a significant reduction in queue times over the coming weeks.

3. **Ad revenue instability**

   While ad revenue is generally out of our control and tends to fluctuate a lot, on June 4th we noticed a sharp decrease in creator revenue (~35% less than normal levels). While our ad provider initially thought this was a display issue, after further inquiry there were 2 causes: 1) Google AdExchange falsely flagging our traffic as invalid 2) Amazon banning many gaming publishers from their network [due to panic in the gaming ads space](https://www.adweek.com/media/exclusive-ads-from-verizon-shell-and-others-ran-next-to-explicit-videos-on-top-android-app/). While the Amazon ban is now resolved, we no longer are running Google AdExchange in the desktop app due to invalid traffic issues. This will lead to a permanent revenue decrease (AdX contributed to ~20% of our ad revenue). We also updated our prebid version (the underlying tech used to run ad auctions) which has shown a measurable increase, bringing revenue back to "normal" levels. Overall, we are closely monitoring and will keep you all posted. However, despite all the issues, due to some end-of-quarter campaigns, **revenue in June was an all time high, at $227k ($170k paid to creators)**!

4. **Payout outages**

   Creators should be able to withdraw their revenue at all times, but due to slow PayPal clearing times and poor planning by us, we've had multiple week long outages in withdrawals. While we do store funds 1:1, these "outages" happen because we primarily store creator funds in an FDIC-insured bank account, as we wouldn't want a PayPal/Tremendous account suspension to cause creators to lose funds. We've now set up internal reporting which should never cause this to happen again (or, if it does, drastically reduce the time payout outages happen)

5. **Platform Revenue Route**

   Due to some unannounced breaking changes in Aditude's API, the platform revenue API was broken. It is now [working](https://api.modrinth.com/v3/payout/platform_revenue). You can also use `start` and `end` fields to filter any date range!

6. **API and Uptime**

   We've migrated our infrastructure for the website, app, and servers to OVH over our existing non-redundant AWS system. We've hit 99.96% uptime on our API and 99.98% on Modrinth Servers!

Thank you all for your patience! If you are having any more issues or have any questions about all of this, feel free to DM @geometrically on Discord or [start a support chat](https://support.modrinth.com) and we will be happy to help!
