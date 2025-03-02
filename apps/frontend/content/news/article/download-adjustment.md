---
title: Correcting Inflated Download Counts due to Rate Limiting Issue
short_title: Correcting Inflated Download Counts
summary: A rate limiting issue caused inflated download counts in certain countries.
date: 2023-11-10T12:00:00-08:00
---

While working on the upcoming analytics update for Modrinth, our team found an issue leading to higher download counts from specific countries. This was caused by an oversight with regards to rate limiting, or in other words, certain files being downloaded over and over again. **Importantly, this did not affect creator payouts**; only our analytics. Approximately 15.4% of Modrinth downloads were found to be over-counted. These duplicates have been identified and are being removed from project download counts and analytics. Read on to learn about the cause of this error and how we fixed it.

A graph of many Modrinth projects and their download counts, showing a disproportionate amount of downloads from China.

![Notice anything out of the ordinary?](./country-download-counts.jpg)

More specifically, the issue we encountered is that the download counts from China were through the roof compared to the page view statistics.

![A graph of many Modrinth projects and their page views, showing a relatively even distribution across countries.](./country-page-views.jpg)

Upon further investigation, there was one specific launcher that was repeatedly downloading the same files from Modrinth over and over again within a very short time span.

![A table of downloads split into several parts.](./downloads-table.jpg)

Notice how the downloads in each section (delineated by the bold line) have the same path and were created within the same second.

This, to say the least, baffled us. We already had code called [Sisyphus](https://github.com/modrinth/sisyphus) in place to limit the number of downloads that a single source can make over a given span of time. So what gives?

As it turns out, the issue lay in the underlying technology used by Sisyphus. It uses [Cloudflare Workers](https://workers.cloudflare.com/) in order to intercept the request each time that a file is requested to be downloaded. Essentially, it acted like so:

1. A source (whether this be a launcher, someone clicking the download button on the website, etc.) would request a file from Modrinth.
2. Sisyphus would take note of this source’s information, including what it requested, its IP address, and its request headers, and write it to a small database. If this source had not requested this path before, it would add one download to this file. If it had already requested it, it would not.
3. Sisyphus would then give the file that the source requested. It gives the file regardless of whether the download counted or not.

For the most part, this system works fairly well. The main issue comes in step 2: it takes a little while for different Sisyphus instances to sync up with each other. One of the benefits of Cloudflare Workers is that the code is deployed to hundreds of different servers around the world. When multiple requests come in at the same time, they can get routed to different servers in order to allow each request to be handled faster. Cloudflare Workers, however, takes [up to 60 seconds](https://developers.cloudflare.com/kv/concepts/how-kv-works/#consistency) for each server’s information to sync up with each other. A server in Australia might know that a given source has already downloaded something, but a server in Turkey might not. As a result, multiple downloads from the same source might all get counted if they are handled by different servers.

In order to fix this, we entirely rewrote Sisyphus. It still uses Cloudflare Workers, but all of the processing of step 2 has been offloaded to the main Modrinth backend. This not only speeds up downloads (even if only briefly), but also makes download counts more reliable. Over the past few days, we've already implemented the necessary adjustments. Our observations have shown that the results are significantly more consistent in their accuracy. Instead of having strange spikes in activity, the graph of new downloads now follows the expected pattern.

![A graph that is split up into two parts: on the left, a spiky graph with the text "old sisyphus". On the right, a graph with consistent dips and peaks.](./new-sisyphus.jpg)

Notice the spikes on the left? Compare that to the silky-smooth sinusoidal satisfaction on the right!

To reiterate, the issue is now resolved and **payouts were not affected**. Payouts do not take into account downloads from launchers other than the [Modrinth App](/app); therefore, this adjustment has no bearing on payouts.

P.S. Are you curious about why our download counter is called Sisyphus? In Greek mythology, Sisyphus rolls a boulder up a hill for the rest of eternity. Like Sisyphus, our download counter has no point other than to keep increasing for as long as Modrinth exists.
