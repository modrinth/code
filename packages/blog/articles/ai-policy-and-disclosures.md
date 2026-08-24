---
title: 'New AI rules and project disclosures'
summary: An update to Modrinth’s Content Rules and our new mandatory content disclosures.
date: 2026-08-13T09:00:00-07:00
authors: ['Dc7EYhxG', 'vNcGR3Fd', 'o2Dd4mrX', 'AJfd8YH6']
---

This year, fully AI-generated content has begun _flooding_ Modrinth. Project submissions have gone up from about 2,500 per week at the start of the year to over 5,000 per week now.

We’ve never had a quality threshold for uploading to Modrinth, and we continue to believe that you should not need to make professional, perfect work in order to publish on Modrinth. But we do believe that it has to be _your_ work, and generating it completely with AI is not your work. Modrinth is a community of creators. In order to be creators, you need to actually create something.

Today, we’re announcing a set of new features and policies aimed at informing users about things they may want to know about a project before downloading, eliminating fully AI-generated content, and enforcing transparency for content that has been made in part with generative AI.

In addition, we have made a very minor update to our [Terms of Use](/legal/terms), to prohibit the use of data on Modrinth to train AI models.

## Content disclosures

We have introduced content disclosures to provide a standardized place for certain things that we feel it is important for users to understand before downloading a project on Modrinth.

![In the Details panel on the sidebar of a project, a couple disclosures are listed as an example. Contains paid features: Cosmetic items are purchaseable with Mod Coins; Contains opt-out telemetry: Anonymous launch analytics about game version and mod loaders.](./details.webp)

Here’s the current list of content disclosures available at this time:

- **Contains AI-generated content**
  - Projects must enable this if they:
    - contain a substantial amount of AI-generated code
    - _any_ assets that are substantially AI-generated
    - the project’s functionality relies on the use of generative AI
    - or, if any element of your project’s page such as description or publishing relies on generative AI
  - There are also options available to disclose if you use it for code, assets, text, or functionality within the project.
- **Contains advertising**
  - Projects that contain advertisements, sponsorships, or promotions of other works must enable this disclosure.
  - If the promotion has no direct monetary value **_and_** it is for something that the average person would consider _relevant_ and _unobtrusive_ (such as a link to your Modrinth profile in the corner of the settings page for your own mod), we would not consider that an advertisement for the purposes of this disclosure.
- **Contains paid features**
  - Projects that contain features that can be obtained by spending real-world money.
  - You must provide a list of the types of paid features.
- **Contains telemetry**
  - If your project collects any data on users (sent either to yourself or a third party), it must be disclosed here.
  - There’s three types of telemetry consent options you can select: Opt-in, Opt-out, and Always active
  - You must provide a list of the types of data that is collected.
- **Contains derivative content**
  - If your project is a fork or uses substantial parts of another work, you must declare it as a derivative work of the source work.
  - You must at least provide the name of the work, and if available, a link. You may also provide a small explanation of how it is a derivative work, such as a brief summary of what is different about your fork.
- **Photosensitivity warning**
  - If your project contains anything that you think may be dangerous to certain people who are sensitive to flashing lights or patterns, you must enable the photosensitivity disclosure.
- **Contains external system interactions**
  - Projects that read or edit things on the user’s system outside of the game must enable this disclosure.
  - You must describe precisely how it interacts with the user’s system in the required note.
  - The most common usage of this is in some “horror” mods, where they may do harmless but spooky things on your computer such as adding creepy files to your desktop.
- **Archived**
  - This isn’t new, but we’re transitioning it from a project visibility status to a disclosure.
  - This will allow you to have an Unlisted project also be archived, previously archived projects were always Public.
  - You can also include a note to explain why your project has been archived.

![In project settings, there is a new Disclosures tab. On it, you can toggle disclosures on and off, and some of them have additional fields you need to fill out.](./settings.webp)

Projects with disclosures you’d like to avoid can be filtered out in search under the “Advanced exclusions” options. When filtering out projects with photosensitivity warnings, we will warn the user that not all projects without the warning are guaranteed to be safe for users with photosensitivities. Advanced exclusion filters will be persistent across searches in your browser so that you don't have to apply them every time you search.

![At the bottom of the search filters, there is an "Advanced exclusions" section. You can select different disclosures to exclude from your search results.](./filters.webp)

## New AI policy

We’ve added Section 6 to the [Content Rules](/legal/rules), which is as follows:

<div id="rule-6"></div>

To summarize:

- AI images as project icons, banners, gallery images, or in their descriptions are no longer allowed.
- Fully or nearly fully AI-generated projects are no longer allowed.
- AI-assisted projects, where the human creator contributed in a primary and significant way, are allowed but must disclose that AI assistance was used.

## Enforcement

Obviously, this is a big change and creators will need to take some time to update their projects to have all necessary disclosures or take down projects that are no longer allowed. For this reason, we will have a 45 day grace period to give everyone enough time to get their projects in order.

During this grace period, we will not be accepting reports for projects in violation of the new AI policy or for missing content disclosures to give them a fair chance to update them. The 45-day grace period ends on **September 27, 2026**.

In addition, we will not tolerate witch-hunting on the Modrinth platform. These disclosures are to inform users and provide transparency between user and creator. Anyone using them to shame, insult, belittle, or harass others will be swiftly and permanently removed from the Modrinth platform and community Discord server. Be kind to each other.

## Terms of Use update

Modrinth's [Terms of Use](/legal/terms) have been updated in a very minor way to prohibit the use of data on Modrinth to train AI models. The exact change is as follows:

> Additionally, you agree not to:
>
> - [...]
> - Use any data, content, or materials available on or through the Service to train, develop, or improve any artificial intelligence or machine learning models.

A larger TOU update will be coming in the future to address other issues since the last update was in 2023, but we felt it was important to address this sooner rather than later.

—

Thank you for your patience with us on this feature. We know it’s been long-requested and overdue, but we wanted to make sure we shipped something comprehensive, enforceable, and more than just a publicity stunt.

Thank you as always for continuing to support us. It means a lot. 💚
