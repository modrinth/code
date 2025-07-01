---
title: 'This week in Modrinth development: Filters and Fixes'
summary: 'After a great first week since Modrinth launched out of beta, we have continued to improve the user interface based on feedback.'
date: 2022-03-09
---

It's officially been a bit over a week since Modrinth launched out of beta. We have continued to make improvements to the user experience on [the website](https://modrinth.com).

## New features

We've added a number of new features to improve your experience.

### Click to expand gallery images

![The new expanding gallery images](./expand-gallery.jpg)

In the gallery page of a project, you can now click on the images to expand the image and view it more closely. You can also use the left arrow, right arrow, and Escape keyboard keys to aid navigation.

### Filters for the 'Changelog' and 'Versions' pages

![The new changelog and versions filtering options](./version-filters.jpg)

Versions on the Changelog and Versions page can now be filtered by mod loader and Minecraft version.

### More easily access the list of projects you follow

![The new 'Following' button in the profile dropdown](./following.jpg)

The link to the list of your followed projects is now listed in your profile dropdown.

## Fixes and Changes

While new features are great, we've also been working on a bunch of bugfixes. Below is a list of some of the notable fixes, but it is not a comprehensive list.

- Improved the layout of the search page's search bar and options card to more dynamically adjust to screen size
- Changed the tab indicator to be rounded
- Changed the download icon to be more recognizable
- Changed the profile dropdown caret to use an SVG instead of a text symbol for better font support
- Changed the styling on text fields to be more consistent with the design language of the site
- Changed the styling on disabled buttons to use an outline to reduce confusion
- Changed the styling on links to be more consistent and obvious
- Changed the wording of the options that move the sidebars to the right
- Changed the green syntax highlighting in code blocks to match the brand color
- Fixed the styling on various buttons and links that were missing hover or active states
- Fixed the inconsistent rounding of the information card on the home page
- [[GH-370]](https://github.com/modrinth/knossos/issues/370) Fixed download buttons in the changelog page
- [[GH-384]](https://github.com/modrinth/knossos/issues/384) Fixed selecting too many Minecraft versions in the search page covering the license dropdown
- [[GH-390]](https://github.com/modrinth/knossos/issues/390) Fixed the hover state of checkboxes not updating when clicking on the label
- [[GH-393]](https://github.com/modrinth/knossos/issues/393) Fixed the padding of the donation link area when creating or editing a project
- [[GH-394]](https://github.com/modrinth/knossos/issues/394) Fixed the rounding radius of dropdowns when opening upwards

## Minotaur fixes

[Minotaur](https://github.com/modrinth/minotaur), our Gradle plugin, has also received a few fixes. This isn't going to be relevant to most people, but is relevant to some developers using this tool to deploy their mods.

- Debug mode (enabled through `debugMode = true`) allows previewing the data to be uploaded before uploading
- Fix edge case with ForgeGradle due to broken publishing metadata
- Fix game version detection on Fabric Loom 0.11
- Fix `doLast` and related methods not being usable because the task was registered in `afterEvaluate`

These fixes should have been automatically pulled in, assuming you're using Minotaur `2.+`. If not, you should be upgrading to `2.0.2`.

Need a guide to migrate from Minotaur v1 to v2? Check the migration guide on the [redesign post](../redesign/#minotaur).
