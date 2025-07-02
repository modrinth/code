---
title: 'Two years of Modrinth: a retrospective'
summary: The history of Modrinth as we know it from December 2020 to December 2022.
date: 2023-01-07
---

Let's rewind a bit and take a look at the past two years of Modrinth's history. We've come so far from our pre-beta HexFabric days to today. A good portion of our pre-beta history can be found in the [What is Modrinth](../what-is-modrinth) blog post, but Modrinth obviously is not the same platform it was two years ago.

## December 2020: Modrinth Beta begins

![Modrinth's brand new design, rolling out with the launch of Beta](../modrinth-beta/new-design.jpg)

> Modrinth's brand new design, rolling out with the launch of Beta

December was the release of the initial [Modrinth Beta](../modrinth-beta), bringing a completely different interface and the ability for mods to be created for the first time. This interface has since been completely discarded, but this is what Modrinth looked like for well over a year. It's hard to believe!

December also brought the introduction of the [Minotaur](https://github.com/modrinth/minotaur) Gradle plugin for the first time for upload automation. Minotaur today also looks nothing like it did when it was introduced, but it still accomplishes the same exact thing.

## January 2021: Improvements to mod uploading

An announcement in mid-January brought several essential additions and improvements to Modrinth which we consider commonplace today. Among these were:

- A separate version creation page
- The ability to edit and delete existing versions
- The ability to delete existing mods and users

January also brought the introduction of Google AdSense onto Modrinth. The eventual results, including our switch after to EthicalAds, solidified Modrinth's stance that ads should be unobtrusive and generally friendly to users.

## February-March 2021: Follows, reports, notifications, oh my!

March brought the first introduction of the abilities to follow and report projects, as well as the automatic featuring of some versions depending on loader and Minecraft version. These systems have largely remained unchanged since their introduction, though the notification system will likely be getting a refresh come 2023.

## April-December 2021: Season of silence

After follows, reports, and all that jazz, Modrinth largely went silent for a good while. This time period had some of the largest growth Modrinth had ever seen, and yet it seemed Modrinth's development had ground to a halt. Modrinth Team members were dropping like flies until there was a point where there was a single person on the team. What happened?

For various reasons, whether it be lack of free time or a lack of interest in Minecraft in general, people ended up leaving to pursue other things. It's not quite as apocalyptic or barren as these descriptions make it seem, but it's more fun to describe it like this.

Picking up the remnants from what others had left behind, one man was destined to continue developing for Modrinth. The one who began the whole operation in the first place, Geometrically, stood up and began developing. Thus came the development of project types, gallery images, API v2, and modpacks.

## January 2022: API v2 introduction

Right around the corner came 2022. Perchance this would be the time for the silence to be broken? Indeed, the world would be able to hear about all that was brewing over the past few months.

Of course, this wasn't all introduced at once—it was a gradual rollout over several months. First was the introduction of v2 of Modrinth's API, allowing many breaking changes to occur, including namely the renaming of _mods_ to _projects_. Wait, why was this necessary?

Up until this point, Modrinth only hosted mods. Project types, as we call them, allow projects to be given the designation of something other than _mod_; for example, _modpack_ or _resourcepack_, like we have today. This simple field, alongside all of the infrastructure which was needed to support it, was the first step to allow modpacks on Modrinth.

## February 2022: Redesign

Remember the interface introduced in December 2020? Let's scrap it! Actually, it wasn't entirely scrapped, but it got a treatment similar to the [Ship of Theseus](https://en.wikipedia.org/wiki/Ship_of_Theseus) to the point that it was barely recognizable.

![The Modrinth homepage](../redesign/thumbnail.jpg)

> The former Modrinth homepage

Alongside this was the official announcement of API v2, as well as the introduction of the project gallery, the changelog tab, dependencies, and many other things. [Here's the blog post announcement for the redesign](../redesign)!

February also brought the introduction of several new Modrinth Team members to the fold, including Prospector and triphora, both of whom are still on the team, alongside Hutzdog and venashial, who we thank for helping us through much of 2022.

## March-April 2022: Small changes and preparation for modpacks

A couple weeks after the redesign we pushed out some changes which included improvements to several tabs on project pages and many bug fixes. [The blog post from that can be found here](../knossos-v2.1.0).

The next couple months were spent preparing for the release of modpacks. This is the first introduction of our "early adopters" program, still in use today, allowing a feedback loop of authors and other community members to create the best product that we can. Without early adopters, many of the features on Modrinth which you've come to love, including modpacks, plugins, resource packs, would be less than ideal.

## May 2022: Modpacks in alpha

In May, we finally did the big release of modpacks on Modrinth. Well, in alpha, anyway—but that was less of a marker of instability and more a marker of being incomplete without the launcher. [The modpack alpha release blog post can be found here](../modpacks-alpha).

When we first announced modpacks, the initial format had been set in stone for a couple years, and it had been decided that CurseForge links would be allowed within them. This got turned on its head due to an email sent to us by Overwolf. More information on that can be found on the [Changes to Modrinth Modpacks blog post](../modpack-changes).

![Progryck](../modpack-changes/thumbnail.jpg)

## June-August 2022: Plugins and resource packs

The summer of 2022 was largely dedicated to working on releasing creator monetization. First, though, we made a pit stop to introduce plugins and resource packs to Modrinth. [Find that blog post here](../plugins-resource-packs).

Plugins in particular were tricky since we had to account for projects which had both mod and plugin versions. It was at this point we realized that the project type system isn't entirely what we cracked it up to be, and we're hoping to completely replace it once API v4 rolls around, as far away as that may sound. For now, though, it will suffice.

## September-November 2022: Creator monetization

With plugins and resource packs done, we continued working on creator monetization. This included [a brief experiment](../carbon-ads) with a different ad provider before we eventually switched to creating [our own ad system](https://adrinth.com).

November brought the actual beta release of creator monetization—[here's the blog post for that](../creator-monetization-beta). We are continuing to develop and refine this system to ensure authors continue to earn money from publishing projects on Modrinth.

## December 2022-January 2023: Anniversary Update

That, of course, brings us to today's [Anniversary Update](../two-years-of-modrinth)! Now that you're done reading this, feel free to go back over to that post and read about everything that's new in the Anniversary Update so that I don't have to repeat myself. Take a look at our New Year's Resolutions for 2023 while you're at it, too!
