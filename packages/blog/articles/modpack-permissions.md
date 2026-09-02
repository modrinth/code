---
title: Improving Modpack review delays
summary: Reducing the back-and-forth needed to get your modpack approved.
date: 2026-06-23T13:00:00-07:00
authors: ['Dc7EYhxG', 'AJfd8YH6', 'xSQqYYIN', 'vNcGR3Fd']
---

Hey everyone,

As you're likely aware if you've attempted to publish a Modpack on Modrinth lately, the delays in getting your project reviewed have become very extreme, sometimes extending into months. We're here to explain why this is, and what we're changing starting today to fix it.

## TL;DR

- Modpack creators now have to submit proof that you have permission to include all external content _before_ it can be submitted.
- All external content is now automatically attributed for you on your Modpack's version pages, similar to how content on Modrinth is shown.
- Any content that we've seen and collected permission information for in the past will be automatically attributed for you.
- Modpack reviews going forward should no longer be any slower than other project types.
- Existing published modpacks are unaffected for now.

## Problem

When creating a modpack, you may want to include content that isn't available on Modrinth. This means that instead of your modpack simply linking to a file on Modrinth, you need to distribute the content itself inside your Modrinth Pack (`.mrpack`) file. However, when you're redistributing someone else's work, you need to make sure that you have the rights to do so.

Historically, our content moderators would help out by manually searching for the where the file came from, read through the licensing information or modpack permission statements, and record whether the content is free to be distributed in modpacks or not. If any content couldn't be identified or did _not_ allow redistribution, the modpack would be rejected with a note to the creator that they needed to get special permission to include it before they could resubmit. Unfortunately, this system was not built for just how big Modrinth has become, and now with over 600 projects being submitted every day, it became impossible for our moderators to spend so much of the time identifying files included in modpacks.

The biggest issues came down to not keeping creators in the loop, which led to us having out-of-date information, not knowing about the custom mod you may have made for your modpack, or our AutoMod system conflicting with our own human moderators. All these issues have regrettably led to modpacks being a source of review delays. Today there are over 3,000 modpacks waiting in our review queue and it would genuinely not be possible to review them faster than they are being submitted with the current system, so something had to change.

With this update, you, the modpack creator, will get the chance to find issues with your modpack and provide correct information about external content **before** you submit it for approval. Your project page will even automatically credit external content once we have the right info! This means no more gambling with your project being denied just because you included the wrong mods, and no need to worry about manually putting a bunch of links in your description to satisfy license attribution requirements.

## What this means for your projects

New modpack projects and projects that have not yet been approved will be required to provide licensing information before review, and for any new versions uploaded after approval. Over the next day, existing projects awaiting review will be scanned, analyzed, and if it is necessary for you to provide attribution information about where these files came from, you will be notified.

Existing projects that were already published before this release will temporarily be excluded from the licensing information requirement as we work out any possible issues with this brand new and complex system. At some point soon, new versions of ALL modpack projects will also be required to provide licensing information for any unrecognized content. We don't have an exact timeline for this, but we are thinking in a month or two. Now would be a good time to check and make sure that you are not inadvertently including content that you don't have permission to redistribute, or your project may run into issues when this is rolled out to all projects.

## Uploading a modpack with external content

Uploading a modpack with external content will still work mostly the same. You will go through the normal version publishing flow, and if your pack contains external content, the version will be temporarily withheld after upload. You'll see a warning letting you know it was withheld because it contains external content of unknown origin, or unknown permissions.

![A modpack version page showing a withheld version warning for unknown embedded content.](./version-withheld.png)

From there, you can open the new Permissions page to review any content that may be an issue, and provide the required licensing information.

## Permissions page

The new Permissions page shows all external content across every version of your modpack project. These files can be in one of several states:

- **Pending**
  - You still need to provide information about this file’s origin and your permission to distribute it.
- **Completed**
  - Information has already been provided, nothing left to do!
- **Information rejected**
  - Moderators reviewed the information provided and were unable to confirm your ability to distribute the file. You’ll have to remove the file from your Modpack or update it with accurate information.
- **No permission**
  - You’ll need to provide proof of your permission to distribute this file or remove the file from your Modpack.

![The Permissions page showing external content grouped by permission status.](./permissions-page.png)

Pending is where you resolve missing permission information. From here, you can choose from four options and attach additional information, such as a link to the work or a short explanation.

- **License**
  - The project's license allows redistribution.
  - _Some licenses may require you link back to the project, don’t worry, we handle this automatically when you link to the source of the content._
- **Your project**
  - You created the content yourself.
- **Special permission**
  - You have verifiable permission from the content’s owner. Some form of proof is required.
- **No permission**
  - You do not have permission to redistribute the file. This can be used as a placeholder to remind you of which content you need to ask for permission to use or remove.

![A permission details form with options for license, your project, special permission, and no permission.](./permission-details.png)

## How external content works across versions

External content files are grouped when we believe files come from the same project. This means if you upload a new version of your modpack with an updated version of a file you've already identified, you usually won't need to update your permissions again.

Our detection isn't perfect, but it should handle a lot of cases automatically. You can see which files and versions have been grouped together, and manually add or remove files from a group if needed.

## Review process

Once you've filled out the information for all external content, you can submit your project for review.

For now, existing published modpacks won't need to go through this process. We'll roll it out to existing projects in the future.

Thank you for support and uploading your modpacks to Modrinth! We hope this will help you get feedback and publish much quicker than before. 💚
