---
title: Modrinth's Anniversary Update
summary: Marking two years of Modrinth and discussing our New Year's Resolutions for 2023.
date: 2023-01-07
---

Modrinth initially [went into beta](../modrinth-beta) on November 30th, 2020. Just over a month ago was November 30th, 2022, marking **two years** since Modrinth was generally available as a platform for everyone to use. Today, we're proud to announce the Anniversary Update, celebrating both two years of Modrinth as well as the coming of the new year, and we'll be discussing our New Year's Resolutions for 2023.

Before you read this post, though, we recommend taking a look at [our retrospective on Modrinth's history through 2020—2022](../two-years-of-modrinth-history). It just wouldn't be right to take a look at the present and the future without also taking a look at our past, seeing how far we've come from our humble beginnings.

With that out of the way, this post primarily serves to announce a few of the smaller features we've been working on after the release of creator monetization. We've bundled these all together as the **Anniversary Update**.

Looking just at what's already done is boring, though, so we'll also be looking at what's yet to come. Modrinth's future is even brighter than any of us can imagine, so we'll be focusing on what we're gonna do in order to get to that bright future. If you've ever made **New Year's Resolutions**, we're going to briefly discuss our resolutions for 2023.

Without further adieu, let's get right into what's new with this new year!

## Shader packs and data packs

The long-awaited arrival of shader packs and data packs is now here on Modrinth!

Shader packs can be viewed in the [shaders tab](/shaders). This includes shaders that support [Iris](/mod/iris), [Canvas](/mod/canvas), and OptiFine, as well as vanilla core shaders. (Even though they're installed via the resource pack system, we have decided to put Canvas and core shaders in shader packs since most users will not search in resource packs for shader packs, even if that's how they're installed.)

Data packs can be found in the [data packs tab](/datapacks). These are implemented similarly to plugins, in that projects with a mod version can also upload a data pack version (and vice versa). Additionally, data pack authors can choose to have their data packs packaged as a mod using the handy-dandy button on the site.

Data packs can optionally upload a corresponding resource pack as a separate file. We discourage bundling data files and asset files in the same zip file.

## New landing page

The [homepage](/) has been completely remade, featuring a scrolling list of random projects from Modrinth. Feel free to use this to discover new projects—just make sure you refresh occasionally, because they loop after a little while until you refresh!

![A screenshot of the new homepage, with a maze background and projects scrolling across the bottom. Bold across the front is "The place for Minecraft mods".](./landing-page.jpg)

## Project overhaul for creators

We're continuing to bring expansions to the creator dashboard introduced with monetization. The new **Projects** tab allows you to view all of your projects in a table and quickly access their information and settings.

[![The new Modrinth project dashboard](./projects-dashboard.jpg)](/dashboard/projects)

The same page also introduces the ability to bulk-edit the external resource links without having to edit each page individually. For example, if your Discord invite expires, you used to have to edit each of your projects individually to add it back. Now you can just select the projects you want to edit the links for and edit them all at the same time!

![A modal with several input fields for external resource links, listing multiple projects the input changes will be applied to.](./bulk-edit.jpg)

Even better are the changes to the settings page for individual projects. Previously, the project settings page was disorganized and cluttered. The project settings page has been completely redone, inspired by GitHub's repository settings page.

![The new project settings page, shown for Sodium.](./project-settings.jpg)

Draft projects also now have a publishing checklist, making it more clear to authors as to what their next steps should look like. Red asterisks are items that must be completed before submitting and purple light bulbs are suggestions.

![A card with several tasks for a draft project owner to do, such as adding a description and selecting the necessary information.](./publishing-checklist.jpg)

## Version page overhaul

The layout of the individual version page has gotten a complete overhaul. It's much easier to just show the new UI in action rather than trying to explain it!

A screenshot of the way that individual versions look now:

![A screenshot of the way that individual versions look now.](./version-page.jpg)

That's not all, though. Version creation now automatically infers most details after you upload your first file. Try it out sometime—whenever you upload your first file, most stuff should already be filled in. This system is still in-development, so if you find any issues, please file an issue on [GitHub](https://github.com/modrinth/code).

## Project card views

Anywhere which lists projects, namely search and user pages, have gotten a great overhaul. You can choose between the classic list view, the grid view, and the gallery view.

![A screenshot of the default view for the Modrinth shaders search.](./search-gallery-view.jpg)

By default, shader packs and resource packs use the gallery view, user pages use the grid view, and everywhere else use the list view. You can cycle through them near the top of each page or change them in your [display settings](/settings).

The gallery image uses the featured gallery image on a project, so please ensure if you are a shader pack or resource pack author that you set a featured gallery image!

## Gallery image UI for creators

The existing UI for gallery image creation, editing, and deletion was flawed in many ways, so we threw out the old way of doing it and created a whole new system for this. It should be less prone to the many many bugs that plagued the previous implementation.

![The new gallery image editing UI, in a modal](./gallery-ui.jpg)

## New project webhook

Our [Discord server](https://discord.modrinth.com) has a brand new channel: #new-projects. A webhook sends a message to this channel every time a new project gets approved. Check it out when you get a chance!

![A screenshot of the new project webhook for Iris Shaders.](./project-webhook.jpg)

## Miscellaneous additions

- Custom SPDX license identifiers can now be selected, and a license's text is now displayed in a modal if the author has not manually set a license link.
- Each project now has a color associated with it, generated from the icon. This color is used in place of a gallery image in search if the project has no gallery image.
- The [bug with disappearing and duplicated versions](https://github.com/modrinth/code/issues/1748) due to the reuse of version numbers is now fixed.
- Whenever a project gets its status updated (for example from _under review_ to _approved_), the project's team members will now get a notification.
- The ability to manually reorder gallery images has been added via an integer ordering field. In the future, this sorting ability may expand to team members and versions. We also hope to add a drag-and-drop functionality similar to Discord server organization.
- You can also now formally request that your project be marked as unlisted, private, or archived instead of always having it be listed first.
- The ability to schedule the release of projects and versions has been added to the backend and is likely to be added to the frontend in the next few weeks.
- Several other bug fixes and minor features, mainly contributed by community members.

## New Year's Resolutions

Now that we've looked at everything accomplished over the past month and a half, let's take a look at our New Year's Resolutions—things we wish to achieve during 2023.

### Theseus Launcher

During 2023, our main focus will shift to the Modrinth launcher, code-named Theseus. Progress has been off and on for the past year and a half, but we intend to fully launch it before the end of the year. Theseus will bring a next-level experience to Minecraft launchers, bringing first-class support for Modrinth and unique features that would be difficult for other providers to parallel.

The release of the Theseus project will also mark the end of the "alpha" status for Modrinth modpacks. Stay tuned for more information about alpha tests and early adopters programs!

### Continuing to grow creator tools

Another one of our focuses for this year is to put more work into our analytics system and in growing creator monetization through our [Adrinth](https://adrinth.com) ad network. As of today, monetization is now out of beta, but we are still constantly working on ways to make Modrinth even better and easier to use for new and returning creators. Some of these improvements are big, like the project settings overhaul, while others are more subtle quality-of-life improvements, like the fixes to usage of duplicate version numbers.

### API changes

This year, Modrinth hopes to introduce version 3 of [our API](https://docs.modrinth.com/api/) with lots of fixes and smaller changes. While our plans are still work-in-progress for this, one of the things that needs to be done first is the removal of the old API v1, which was deprecated starting in January 2022. Here's our planned timeline for the removal of API v1:

- **January 7th, 2023 (Now):** Begin sending messages to existing API v1 users
- **January 7th, 2023 (Now):** Add a field to each API result telling people to switch
- **February 14th, 2023:** Begin doing flickers of 5-10 minutes of 410 GONE response codes
- **March 1st, 2023:** Begin sending a permanent 410 GONE response for any non-GET routes
- **March 1st, 2023:** Ramp up 410 GONE flickers to last 6-12 hours for GET routes
- **March 15th, 2023:** Replace all remaining GET routes with a permanent 410 GONE response

### Small updates throughout the year

As always, we will be interspersing other, smaller quality-of-life updates throughout the year even as we work on the big stuff. We also want to fix any bugs which might come up alongside any updates.

## Conclusion

Modrinth was founded with the goal of creating a platform which keeps the broader modding community's interests at heart. Modrinth would not exist without the support of our users and of our contributors, and we thank everyone involved immensely for everything. Modrinth's development shall continue as long as the community is willing to support us on the way!

We would love to hear any feedback you might have. Feel free to get in contact on [Discord](https://discord.modrinth.com), on [Twitter](https://twitter.com/modrinth), and on [Mastodon](https://floss.social/@modrinth).
