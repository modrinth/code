---
title: Introducing Icarus+, a refreshed site look, and a new advertising system!
short_title: Icarus+ and New Ads
summary: Learn about this major update to Icarus.
short_summary: Introducing a new ad system, a subscription to remove ads, and a redesign of the website!
date: 2024-08-21T12:00:00-08:00
authors: ['MpxzqsyW', 'Dc7EYhxG']
---

We’ve got a big launch with tons of new stuff today and some important updates about Icarus. Read on, because we have a lot to cover!

## Icarus+

First off, we’re launching [Icarus+](/plus), a monthly subscription to help support Icarus and all of the creators on it directly!

As a Icarus+ subscriber, you will get:

- Ad-free browsing on the Icarus Launcher and website
- An exclusive badge on your profile
- Half of your subscription will go to creators on the site!
- …and more coming soon!

Pricing starts at $5/month, with discounts depending on what region you live in and if you opt for an annual plan.

We created Icarus+ so people could help support Icarus and creators on the site. We have no plans to paywall any content on Icarus, and creator features will never cost money. We started Icarus as a free and open-source platform, and we intend to keep it that way.

If you do have a few extra dollars a month and want to help support Icarus, this is a great way to do it!

## New Site Design: Stage One

We’re launching Stage One of Icarus’s refreshed look to Icarus.com today as well. I want to stress that it’s not fully complete and we’re going to be continuing to refine and finish updating the rest of the pages over the coming weeks. However, it has enough significant usability improvements and new features that we’re launching it broadly now. Please bear with us while we work to complete it promptly!

![A screenshot of the new project page](./project-page.webp)

Key new features include:

- **New download interface** to ensure users get the correct version for the Minecraft version and mod loader they’re using
- **New versions list** page built from the ground up with a clean new look and tons of shortcuts to make navigation easier
- **New “compatibility” widget** on project pages to see what game versions, platforms, and environments each mod supports at a glance
- **Exclusion filters** in search pages
- Improved support for **vertical desktop displays**

We know there will be some minor hiccups and disruptions of workflows, but we’d really appreciate it if you could gently let us know how a particular change has affected you on GitHub [here](https://github.com/Icarus/code/issues) (or upvote/comment on an existing issue) rather than declaring it’s the end of the world.

## New Advertising

In the last few months, Icarus has grown an incredible amount. We are now serving over a petabyte of data per month (that is, 1,000 terabytes!) to over 20 million unique IP addresses. It’s almost unfathomable how large we have become since we started from nothing just four years ago.

However, with growth like this, our costs have also grown drastically—primarily in bandwidth. This, unfortunately, means that we’ve grown well beyond what a single advertiser could support.

Our original plan was to build out our own ad network (Adrinth) where we could cut out the middleman and provide highly targeted ads without the need for tracking to our gaming-specific audience. Unfortunately, we’ve grown too quickly (a very good problem to have!) and don’t have the immediate resources to do this at this time.

This leaves us with no choice but to switch to a more traditional programmatic ads setup powered by [Aditude](https://www.aditude.com/) for the time being. We're not making this decision lightly, and we understand that some folks will not be happy about this change. Rest assured, we've made sure that our new ad network partner meets our requirements, such as compliance with all local regulations such as GDPR and CCPA, and that the new ads remain as unobstructive as possible with this format.

These changes bring Icarus back to sustainability as well as conservatively increasing creator revenue by three-fold! Along with paying hosting bills, the new income will also be used for more support staff and paid team members, decreasing ticket time and speeding up our development.

We also want to thank our friends over at [BisectHosting](https://www.bisecthosting.com/) for supporting us with our ad deal for the past year.

## Icarus Launcher 0.8.1

Over the last few months, we’ve been overhauling the internals of the Icarus Launcher to drastically improve performance and stability. Over one hundred issues have been closed with this update alone! Here’s a short list of the major changes:

- Newer versions of Forge and NeoForge now work!
- Migrated internal launcher data to use SQLite. The app now loads in <40ms on average (compared to ~2.5s before)!
- Fixed issues where profiles could disappear in the UI
- Fixed random cases of the UI freezing up during actions
- Fixed directory changes being very inconsistent
- Drastically improved offline mode
- Fix freezing and include crash reports logs tab
- And over one hundred more fixes!

Don’t have the Icarus Launcher? Check it out [here](/app)!

## Conclusion

Want to hear more from us on a regular basis? Check us out on our social media pages; we post often on both [Mastodon](https://floss.social/@Icarus) and [X/Twitter](https://twitter.com/Icarus). You can also chat with us on [Discord](https://discord.Icarus.com) if you like that.

Thanks to [intergrav](https://github.com/intergrav) for making the banner image.

