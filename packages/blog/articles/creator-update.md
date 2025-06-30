---
title: 'Creator Update: Analytics, Organizations, Collections, and more'
short_title: The Creator Update
summary: December may be over, but we’re not done giving gifts.
short_summary: Adding analytics, orgs, collections, and more!
date: 2024-01-06T12:00:00-08:00
---

December may be over, but that doesn’t mean we’re done giving gifts here at Modrinth. Over the past few months, we’ve been cooking up a whole bunch of new features for everyone to enjoy. Now seems like as good of a time as ever to bring you our Creator Update! Buckle up, because this is a big one.

The headlining features include:

- **Analytics** - Allowing Modrinth creators to see statistics from their projects.
- **Organizations** - Better tools to manage shared ownership over multiple projects.
- **Collections** - A system for putting together shared sets of projects, similar to Spotify playlists.
- **New payouts system** - Updates to the existing Creator Monetization Program to better serve creators around the world.
- **New Markdown editor** - Explore a complete reworking of our text editor, making it easy even for those unfamiliar with Markdown.
- **OAuth integrations** - Our own implementation of the OAuth specification, allowing external applications to “log in with Modrinth”.

## Analytics

The long-awaited addition of **analytics** is here for creators! You can view analytics over time for your projects, including downloads, page views, and revenue, all in an effortlessly easy-to-use dashboard.

![The analytics for a project, showing downloads, page views, and revenue, with a breakdown by country.](./project-analytics.jpg)

![A screenshot of the analytics for a user, showing multiple different projects.](./user-analytics.jpg)

The data for analytics have been collected over the course of many months. In fact, the data for revenue goes all the way back to August 2022, and the data for downloads and views back to February 2023.

You can view the analytics for an individual project by going to the settings and clicking “Analytics”. You can view analytics for all of your projects in [the analytics dashboard](/dashboard/analytics).

## Organizations

Isn’t managing permissions across a bunch of different projects pretty tedious? We sure thought so. Just like on GitHub, you can now create organizations on Modrinth to manage permissions across multiple projects.

![A screenshot of the organizations section of the Modrinth dashboard.](./organizations.jpg)

You can create organizations from the [organizations dashboard](/dashboard/organizations). Each organization has a name, a brief summary, and an icon. Just like project members, organization members have a role, a monetization weight, and project permissions, plus permissions for the organization as a whole. Roles, monetization weights, and project permissions can be overridden on a per-project basis.

![A screenshot of a user page, with two organizations shown at the very bottom.](./user-orgs.jpg)

Unlike GitHub, usernames and organization names on Modrinth do not conflict with one another. If you want to have an organization named after yourself, feel free to do so!

## Collections

Just like how Spotify has playlists or how Goodreads has shelves, Modrinth now has collections! Collections are lists of Modrinth projects put together for a common purpose. You can then share these collections with others to view.

![A screenshot of the Project Odyssey suite of mods as a collection.](./collections.jpg)

Your [followed projects](/collection/following) now make up an automatically generated private collection, which you can access from the [“Your collections” section of the dashboard](/dashboard/collections).

### Wait… aren’t those just modpacks?

Not quite! Modpacks are much more complex than collections. Collections are simply lists of projects. Here’s a quick comparison:

| Modpacks                                                                                                                          | Collections                                                                                                                                   |
| --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Created through a launcher, such as the [Modrinth App](/app).                                                                     | Created on the [Modrinth website](/dashboard/collections).                                                                                    |
| Contains options files, configuration files, and optionally files from outside of Modrinth, wrapped together in a `.mrpack` file. | Contains a list of Modrinth projects (mods, plugins, data packs, resource packs, shaders, and modpacks).                                      |
| Has individual releases with version history.                                                                                     | Instantly updates whenever a project is added or removed.                                                                                     |
| Must be reviewed by Modrinth’s staff and approved per [Modrinth’s rules](/legal/rules) before it can be published.                | Does **not** need to be reviewed by Modrinth’s staff. Can go public at any time.                                                              |
| After approval, can be **listed** in search, **archived**, **unlisted**, or **private**.                                          | Can be **public** (shows up on your Modrinth profile), **unlisted** (only accessible by direct URL), or **private** (only you can access it). |

All in all, collections are handy for easily grouping together and sharing Modrinth projects. If you’re bored on the subway heading home, you can look for new mods on your phone and quickly add them to a Modrinth collection. However, for many use cases, spending the time to create a modpack might make more sense. Collections and modpacks are both here to stay—one is not going to replace the other.

## New payouts system

PayPal and Venmo are so 2023. To enter 2024, we are adding support for a bunch of different new payout methods, including ACH (available for direct transfer to a United States bank account) and a couple thousand gift cards. You know, just “a few”.

![The withdrawal screen, with PayPal, Venmo, ACH, Visa, and a preview of two of the available options for the United States (AMC and Airbnb)](./payouts.jpg)

Whether you want Applebee’s in America, Boek & Bladkado in Belgium, or Cineplex in Canada, we’ve got them all and plenty more. Prepaid Visa cards, Amazon gift cards, and Steam gift cards are among the available options. Does anyone want a Home Depot gift card? We’ve got those, too.

## New Markdown editor

For the longest time, Modrinth’s text editor for descriptions, changelogs, reports, and more has just been a box to enter [Markdown syntax](https://en.wikipedia.org/wiki/Markdown). What about people who don’t know Markdown, though? Even for those who do, writing it out by hand gets tedious after a while. That’s why we rebuilt it from the ground up to make it far easier to use.

<iframe width="560" height="315" src="https://www.youtube.com/embed/X07M-IFsqbs?si=pUca7XGdvtdd4XlD" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

Among its features are standard shortcuts (like `Ctrl+B` for **bold**), a monospace font in the editor itself, and buttons for inserting headers, basic formatting, lists, spoilers, block quotes, links, images, and YouTube videos.

Using the image button, you can also now upload images directly, instead of having to use an external host or the Gallery tab of a project. You can still insert images from outside sources, though certain sources (such as the Discord CDN) are blocked. We will notify authors using these blocked sources to replace the images.

## OAuth integrations

Wouldn’t it be nice if other websites or apps could add a “Sign in with Modrinth” feature? We asked ourselves this and thought, yes, it would be nice to add. So we added it.

The [OAuth2 protocol](https://en.wikipedia.org/wiki/OAuth) allows other services to gain a limited amount of access to your Modrinth account without compromising your login information. Maybe you want to create your own analytics dashboard? Or maybe you want to make your own way to add content to collections? How about connecting organization permissions to roles in a Discord server? The possibilities are endless.

![A screenshot of an OAuth app requesting permission to your user profile.](./oauth.jpg)

You can create a new OAuth application in the [Applications](/settings/applications) section of your settings. You can see which applications you’ve granted access to in the [Authorizations](/settings/authorizations) section.

## Conclusion

Want to hear more from us on a regular basis? Check us out on our social media pages; we post often on both [Mastodon](https://floss.social/@modrinth) and [X/Twitter](https://twitter.com/modrinth). You can also chat with us on [Discord](https://discord.modrinth.com) if you like that.

Thanks to [intergrav](https://github.com/intergrav) for making the banner image.
