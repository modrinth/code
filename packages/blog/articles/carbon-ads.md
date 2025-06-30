---
title: Modrinth's Carbon Ads experiment
summary: "As a step towards implementing author payouts, we're experimenting with a couple different ad providers to see which one works the best for us."
date: 2022-09-08
---

**Update 10/24:** A month and a half ago we began testing Carbon Ads on Modrinth, and in the end, using Carbon did not work out. After disabling ads with tracking in them, the revenue was about equal to or worse than what we were generating previously with EthicalAds. Effective today, we are switching our ads provider back to EthicalAds for the time being.

As a step towards implementing author payouts, we're experimenting with a couple different ad providers to see which one works the best for us. We've been using [EthicalAds](https://www.ethicalads.io/) for a long time now, but we'd like to now try out [CarbonAds](https://www.carbonads.net/).

In most respects, this is a temporary experiment, but we're hoping that Carbon will become our primary ad provider in the near future.

Over the past week and a half, we've garnered a lot of useful feedback in the `#devlog` channel of [our Discord](https://discord.gg/EUHuJHt). Over those 1,300 or so messages of open discussion and debate, there were also a lot of questions, concerns, and misconceptions. This blog post aims to address the most frequent of those.

## FAQ

### Is Carbon GDPR and CCPA compliant?

Yes. This was confirmed to us via email by a Carbon representative.

### Are the ads intrusive?

No. They fall under the [Acceptable Ads Standard](https://acceptableads.com/standard/); that is, there is only ever one per page, they are less than 120 pixels tall, and they are separate and distinguishable from actual site content.

### Where did the privacy settings go?

Alongside the introduction of Carbon, we have removed the privacy settings that we previously had. These privacy settings controlled whether PII would be sent in our internal analytics and whether you wanted personalized ads to show up. Our analytics do not contain PII and Modrinth does not show personalized ads. Both of those would be intense breaches of your privacy, opt-in or not, and Modrinth intends to respect your privacy.

### Why are you switching before you've released payouts?

We have been using [ariadne](https://github.com/modrinth/ariadne) to take note of page views and ad revenue since August 1st, 2022. While creator payouts cannot yet be claimed, all ad revenue from this date forward will be claimable once payouts are released!

Payouts are not yet done, but this switch is one of the largest things that needs to be done prior to its release.

### Why does Modrinth need to switch away from Ethical?

There are quite a number of reasons why it's not feasible for us to continue using Ethical. In order to be fully transparent, let's go into detail about each of them.

#### In-house ads

Over half of the ads shown by Ethical are their so-called "in-house ads". That is, Ethical does not have enough inventory to always be showing an ad, so instead it shows an advertisement for itself. These self-advertisements make a whopping $0 for Modrinth.

Ethical does provide an option to replace these self-advertisements with our own fallback ads, which we've done for the past month or so. However, negotiating those sorts of deals takes an excruciating amount of time, time that we would rather be spending on developing Modrinth to make it better.

Carbon allows us to have a more hands-off approach with advertising, which is most ideal for us right now.

#### Poor CPM

Ethical gives us an average of $0.24 for every thousand page views (also known as CPM) after taking into account the aforementioned in-house ads. Anyone who knows anything about the advertising business knows that this figure is abysmally low. With Modrinth getting over four million page views in a month's timespan, we make an average of less than $1000 per month with Ethical. This simply isn't sustainable for the thousands of creators on Modrinth.

While we can't quite be sure what our CPM with Carbon will be -- again, this is only a temporary experiment for now -- we have reason to believe that it will be considerably greater than what Ethical can provide.

#### Network in decline

Over the time that Modrinth has used Ethical, we have found that the diversity of the advertisers shown has declined at a rate greater than is sustainable. The vast majority of the ads shown by Ethical, excluding its in-house ads, are for DigitalOcean. If DigitalOcean decided to withdraw from Ethical, that would end up toppling our entire system. Modrinth's payouts simply cannot rest on this house of cards if we wish to grow in any capacity.

### Can I still use my adblocker?

You are still able to access Modrinth using an adblocker, and Modrinth will not force you to disable it to access the site. However, Modrinth's ads are unintrusive and take up no more space than it would otherwise.

When you turn off your adblocker for Modrinth, you are supporting both Modrinth and its creators in the process. 100% of the ad revenue from creator pages, including projects, versions, and users, go directly to creators. The ad revenue from other pages, including search, pay for Modrinth's upkeep costs and allow us to continue to exist.

For the benefit of everyone involved, we humbly request that you turn off your adblocker for Modrinth. We have a full guide for how to turn off your adblocker located [on our docs site](https://docs.modrinth.com/docs/details/carbon/).

## Conclusion

In conclusion, we hope you're as excited about our upcoming release of payouts as we are. Exploring our options for ad providers is quintessential if we wish to be sustainable for payouts, and the best time to do this is now. As always, though, no release ETAs!

Please note that this blog post was not editorialized or reviewed by Carbon prior to publishing. These are the findings and words of Modrinth and Modrinth alone. What's said here about CPMs and other statistics will not be true of other sites, but they are true for Modrinth.
