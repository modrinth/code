---
title: 'Now showing on Modrinth: A new look!'
summary: 'After months of relatively quiet development, Modrinth has released many new features and improvements, including a redesign. Read on to learn more!'
date: 2022-02-27
---

After months of relatively quiet development, Modrinth has released many new features and improvements, including a redesign. While we've been a bit silent recently on the website and blog, our [Discord server][Discord] has activity on the daily. Join us there and follow along with the development channels for the very latest information!

For both those who aren't in the Discord and for those who are, this serves as a status update for what exactly has been going on in our silence. There have been an unparalleled amount of changes, improvements, bug fixes, and new features being worked on since April 2021, and we are incredibly excited to share them with everyone. There are still many things we're still working on, such as modpacks, but we've decided to hold back on that as there is still some fine-tuning that needs to be done on that front.

## New and improved design

The [frontend](https://github.com/modrinth/knossos) has received a considerable facelift. With designs made in part by [falseresync](https://modrinth.com/user/falseresync) (and a sprinkle of bikeshedding), we present to you, the redesign!

As they say, a picture tells somewhere around nine-hundred odd words. As such, this section will be heavily focused on screenshots of the pages rather than long descriptions.

### Project pages

![The new page design, shown for Iris](./iris.jpg)

_A beautiful project page for Iris to match its beautiful shaders_

On project pages, much of the focus has been shifted to the extended description rather than the metadata, which has been put over on the side. We've also added an option to switch this from the left side to the right side in your user settings, if you so desire.

### Gallery

![A preview of the gallery functionality](./consistency.jpg)

_Pictures... pretty!_

Developers can now add a Gallery section on each project page! Each uploaded image or GIF can have a title and description associated with them.

### Changelog

![The changelog page](./adorn.jpg)

_A changelog page for showing the difference between updates!_

Version changelogs are automatically compiled together into a large changelog list. These are put in reverse chronological order, and are separated for Fabric and Forge versions.

### Version creation and dependencies

![The version creation page](./version-creation.jpg)

_Version creation has gotten an overhaul!_

While dependencies have existed in the backend for a while, their implementation was a bit haphazard and was never widely used due to never being in the frontend. Thus, all previous dependencies have been wiped, and they have been redone better(TM). And hey, now you can add and see dependencies in the frontend!

### Profile settings & dashboard

![The profile settings page](./profile-settings.jpg)

_The new settings panel for managing your profile and other visual settings_

The dashboard has been reworked and reorganized: the "My mods" section has been merged into the profile page itself, and the "Settings" page has been split into "Profile" and "Security". There are also options for switching the project and search information from the left side of the screen to the right.

![A user's profile](./jellysquid.jpg)

The notifications page is also now its own page separate from the dashboard, accessible only from the header. The notifications page also has a highly-requested "Clear all" button.

![The notifications page](./notifications.jpg)

## Backend changes and API v2

There have been a number of breaking changes in this update, and as such, the API number has been bumped. The `/api/` prefix has also been removed, as it's redundant when the base API URL is `api.modrinth.com`. This means the production URL is now `api.modrinth.com/v2` instead of `api.modrinth.com/api/v1`.

The major changes include the universal rename of `mod` to `project`, as well as the move of the `mod` endpoint to `search`. While version 1 will be supported until January 2024 and won't be removed until July 2024, we still highly recommend that applications migrate as soon as possible. For full migration instructions, see the migration guide [on the docs site](https://docs.modrinth.com/docs/migrations/v1-to-v2/).

## Minotaur

[Minotaur](https://github.com/modrinth/minotaur) is the tool for mod developers to upload their mod directly to Modrinth automated through Gradle. Minotaur received a considerable facelift and is now a lot more user-friendly. Previously, an example buildscript might look like this:

```groovy
task publishModrinth(type: com.modrinth.minotaur.TaskModrinthUpload) {
  onlyIf {
    System.getenv().MODRINTH_TOKEN
  }
  token = System.getenv().MODRINTH_TOKEN
  projectId = 'AABBCCDD'
  versionNumber = version
  versionName = "[$project.minecraft_version] Mod Name $project.version"
  releaseType = 'alpha'
  changelog = project.changelog
  uploadFile = remapJar
  addGameVersion('1.18')
  addGameVersion('1.18.1')
  addGameVersion('1.18.2')
  addLoader('fabric')
}
```

This exact same buildscript snippet, in Minotaur 2.0.0, can be written as the following:

```groovy
modrinth {
  projectId = 'AABBCCDD'
  versionName = "[$project.minecraft_version] Mod Name $project.version"
  releaseType = 'alpha'
  changelog = project.changelog
  uploadFile = remapJar
  gameVersions = ['1.18', '1.18.1', '1.18.2']
  loaders = ['fabric']
  dependencies = [
          new ModDependency('P7dR8mSH', 'required') // Creates a new required dependency on Fabric API
  ]
}
```

Notice how it's now in a `modrinth {...}` block instead of creating a new task. The `modrinth` task is automatically created.

The `loaders` declaration in the new version isn't even needed if you're using Fabric Loom or ForgeGradle. The project version can be detected automatically, and the token uses the `MODRINTH_TOKEN` environment variable by default now. The game version and loader listings actually make sense now, and dependencies are possible!

## More miscellanea

Along with the major headlining features, there are also a number of smaller features, fixes, and improvements coming with the big update. Most of these need no more than a bullet to describe, so here's a bullet list of the smaller things!

- If you are the owner of a project, you can now transfer the ownership to another user, as long as they have already accepted an invitation to be a member. In the frontend, this can be done on the Settings page, under the "Team members" section.
- iframes for YouTube videos are now allowed. Any iframes from elsewhere are still not allowed.
- Files are now validated to ensure they contain a valid Forge or Fabric mod, or are in the correct modpack format.
- When changing the status of a project, file moderators are now able to add a message (heading and body separate) to be seen by project team members.
- Versions must now always have a file attached.
- Projects will only be able to have `draft` status if they contain no versions. Additionally, a new `archived` status has been added.
- Donation URLs have been re-enabled.
- Fix: Markdown checkboxes will no longer render strangely ([knossos#291](https://github.com/modrinth/knossos/pull/291))
- Fix: [Maven](https://docs.modrinth.com/docs/tutorials/maven/) will no longer randomly break ([labrinth#264](https://github.com/modrinth/labrinth/pull/264) and [labrinth#252](https://github.com/modrinth/labrinth/pull/252))
- ...and many other smaller things!

## What happened to modpacks?

We've been teasing modpacks for a long time now. While they're done for the most part, we've decided to hold back on their release for the time being. We're working hard to get those done some time soon, and there'll be another post when those are ready for general consumption.

## Conclusion and a call for developers

In conclusion, we hope that you're excited about this update as much as we are. We believe that, with how much work has been put into this update, it has definitely been worth the wait.

On a separate note, are you looking to contribute to Modrinth? Have you got experience with Rust or Svelte? We're hiring! Please reach out to `Geometrically#8387` on Discord to apply for a position.

Thank you for reading, and may your dreams be filled with pineapples, tiny potatoes, and squirrels.

[Discord]: https://discord.gg/EUHuJHt
