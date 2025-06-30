---
title: Plugins and Resource Packs now have a home on Modrinth
summary: 'A small update with a big impact: plugins and resource packs are now available on Modrinth!'
date: 2022-08-27
---

With the addition of modpacks, creating new project types has become a lot easier. Our first additions to our new system are plugins and resource packs. We'll also be working on adding datapacks, shader packs, and worlds after payouts are released.

Don't worry - this hasn't taken away an awful lot of development time from author payouts. Those are still being worked on!

## Plugins

With plugins, we're supporting five loaders and three proxies: Bukkit, Spigot, Paper, Purpur, Sponge, BungeeCord, Waterfall, and Velocity.

Several new categories have specifically been added for plugins, though mod categories can be used for plugins and vice versa.

[Go browse our plugin section!](https://modrinth.com/plugins)

### Why add plugins?

This is a question we've received quite often since we first announced our intention to host plugins, so let's break it down a bit.

Currently, there are three main platforms on which plugins can be downloaded from: Bukkit, Spigot, and Sponge's Ore. Notice the main issue there? These sites are bound to a specific loader. This isn't inherently _bad_ - however, as forks and new projects spawn, there is a noticeable lack of flexibility in what can be hosted on a given platform. For example, Spigot is unable to host plugins which specifically depend on the exclusive features provided by Paper's API. Paper's solution to this is to build their own platform, but this simply perpetuates the same problem.

The best solution here is to create a separate platform which is unbiased and flexible enough to adapt to a changing ecosystem. Modrinth is the perfect candidate for this - after all, plugins are mods under a different name, and likewise mods are plugins under a different name.

No matter the situation, authors are always allowed to upload their plugins to multiple sites. Build automation is incredibly easy to set up, especially with "set it and forget it" build tools such as [Minotaur](https://github.com/modrinth/minotaur).

### Will paid plugins be supported?

No. Modrinth does not have the infrastructure to support this, and it's not currently planned. Author payouts are still being worked on.

### What about mods that have plugin versions and vice versa?

Modrinth is taking a unique approach to this. While the search pages are separate, in reality, the backend is the same. You can select plugin loaders when creating a mod and you can select mod loaders when creating a plugin. The split only exists on the frontend so that projects like [Chunky](https://modrinth.com/mod/chunky) can share a single page across their versions.

Plugins which also have versions for mod loaders will be displayed under the `/mod/` URL on the frontend. Plugins without mod loader versions are displayed under `/plugin/`.

## Resource packs

The other thing we've added support for is resource packs!

Previously we hinted at Bedrock resource packs being supported in addition to Java resource packs. We've decided not to add Bedrock resource packs until we also add support for other Bedrock resources for various technical reasons.

[Go browse our resource pack section!](https://modrinth.com/resourcepacks)

### Secondary categories

Resource packs are capable of adding a wide range of different things, like fonts, sounds, and core shaders. We found that the current category system was inadequate to account for all of these, especially with the three maximum limit. Thus, we've introduced a "secondary category" system, for categories which don't display by default but can still be searched. These secondary categories have a limit of 255 instead of three. Please add as many secondary categories as are relevant!

On search pages, "Features" have been split into their own header. Where categories for resource packs can be accurately described as themes, features instead show what exactly a resource pack adds. Resolutions have also been split into their own header, though selecting a pack resolution is optional.

### What about resource packs that require a mod to function?

Resource packs are able to set dependencies on other projects (even those which aren't resource packs), just like how modpacks are able to set dependencies on mods. It's worth noting that OptiFine is not on the platform, and thus you cannot set a dependency on that; however, you can set a dependency on any of the other alternative mods which _are_ available on Modrinth, including [Entity Texture Features](https://modrinth.com/mod/entitytexturefeatures), [OptiGUI](https://modrinth.com/mod/optigui), [Continuity](https://modrinth.com/mod/continuity), [CIT Resewn](https://modrinth.com/mod/cit-resewn), [Animatica](https://modrinth.com/mod/animatica), or [Custom Entity Models](https://modrinth.com/mod/cem).

## Other miscellaneous changes

### Version number changes

For a long time, version numbers have had a requirement to be unique within the same project. Alongside this update, we found it necessary to remove this restriction on version numbers. Thus, you'll no longer have to use something like `1.2.3+forge` and `1.2.3+fabric` if you have a project on multiple loaders - instead, you can just use `1.2.3`.

To accommodate this, the frontend now appends the loaders and game versions onto the end of a URL if there are duplicates, and the [Modrinth Maven] now supports version IDs.

We do not recommend retroactively changing version numbers to remove this additional metadata, though. If you change your version numbers, the following will break:

- URLs to specific versions
- Buildscripts depending on your project via the [Modrinth Maven]
- Download counters (see labrinth issue [#351](https://github.com/modrinth/labrinth/issues/351))

### LiteLoader support

Modrinth now supports LiteLoader for mods. It's nothing special, but it should help with some archival efforts.

### Misc category deletion

We've also deleted the `Misc` category as no one is going to want to filter by `Misc` in search. If you have any other suggestions for categories, feel free to suggest them in [our Discord][Discord] or [Tweet at us](https://twitter.com/modrinth)!

## Developer/API changes

The changes in this update are rather minimal when it comes to API-related stuff. Two new fields have been added to the [project struct](https://docs.modrinth.com/api-spec/#tag/project_model) - `approved`, which is the timestamp of when the project was approved (null if it's not approved or unlisted), and `additional_categories`, another set of categories which are to be seen as less important than normal categories. You can read the [secondary categories](#secondary-categories) section for more info on it. If you wish to implement the headers in your API integration, the [category list](https://docs.modrinth.com/api-spec/#tag/tags/operation/categoryList) now has a `header` field.

As for the [search result struct](https://docs.modrinth.com/api-spec/#tag/project_result_model), `created` now matches the `approved` date rather than the `published` project field, and `categories` now also includes secondary categories. A new field, `display_categories`, matches only primary categories.

Differences between mod loaders and plugins will need to be hardcoded within your API integration for the time being if you wish to have them shown separately. This will be cleaned up in API v3 alongside a general cleanup of a lot of other small aspects of the API. If you have any suggestions for breaking API v3 changes, feel free to suggest them in [our Discord][Discord]. Development on API v3 is likely to begin before the end of the year.

## Conclusion

We're very happy to be announcing this feature, even if it is minor in comparison to some of our other past and future announcements. Don't worry - author payouts are still being worked on, and will most likely be our next major announcement! We saw this as an opportunity to get a feature out with relatively little new code (since we'd already done everything needed alongside modpacks), so we ran with it.

As always, feel free to provide feedback on [our Discord][Discord], and please report any bugs you come across on [our GitHub](https://github.com/modrinth).

[Discord]: https://discord.modrinth.com
[Modrinth Maven]: https://support.modrinth.com/en/articles/8801191-modrinth-maven
