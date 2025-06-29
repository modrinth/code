---
title: 'Modrinth Modpacks: Now in alpha testing'
summary: After over a year of development, we're happy to announce that modpack support is now in alpha testing.
date: 2022-05-15
---

After over a year of development, Modrinth is happy to announce that modpack support is now in alpha testing!

What does alpha mean, exactly? Principally, it means that **modpack support is still unstable** and that not everything is perfect yet. However, we believe it to be complete enough that it can be released for general use and testing.

From this point forward, Modrinth has shifted development effort from modpacks to creator payouts. This long-anticipated feature means that mod developers, modpack creators, and anyone else who uploads content to Modrinth will be eligible to get the ad revenue generated from their project pages.

## Where can I find them?

Right next to mods on the site! URLs to modpacks are the same as mods, just with `mod` replaced with `modpacks`, so you can find the search at https://modrinth.com/modpacks.

Over a dozen modpacks have already been created by our early pack adopters, and those are available for download right now!

## Wait, so how do I download them?

At this point in time, the only stable way to download modpacks and use them is through [ATLauncher]. You can also install Modrinth packs if you switch to the development branch of [MultiMC]. We're hoping to be supported by more launchers in the future, including our own launcher, which is still in development. Our [documentation for playing modpacks] will always have an up-to-date listing of the most popular ways to play packs.

## How do I create packs?

You can either use [ATLauncher] or [packwiz] to create modpacks. The [Modrinth format] is unique for our purposes, which is specifically in order to allow mods from multiple platforms to be in a pack. Our [documentation for creating modpacks] will always have an up-to-date listing of the most popular ways to create packs.

## Can I use CurseForge mods in my modpack?

Yes! The [Modrinth format] uses a link-based approach, meaning that theoretically, mods from any platform are usable. In practice, we are only allowing links from **Modrinth**, **CurseForge**, and **GitHub**. In the future, we may allow other sites.

## What happened to Theseus?

For a while, we've been teasing Theseus, our own launcher. While lots of progress has been made on it, we haven't yet gotten it to a usable state even for alpha testing. Once we think it's usable, we will provide alpha builds -- however, for now, our main focus will be shifting to payouts, with Theseus development ramping up once that is out.

Remember: Modrinth only has a small team, and we have a lot of real-life responsibilities too. If you have experience in Rust or Svelte and would like to help out in developing it, please feel free to shoot a message in the `#launcher` channel in our [Discord].

## Conclusion

All in all, this update is quite exciting for everyone involved. Just like with [the redesign](/packages/blog/articles/redesign.md), this is the culmination of months upon months of work, and modpack support is really a big stepping stone for what's still yet to come.

Remember: alpha means that it's still unstable! We are not expecting this release to go perfectly smoothly, but we still hope to provide the best modding experience possible. As always, the fastest and best way to get support is through our [Discord].

Next stop: creator payouts!

[ATLauncher]: https://atlauncher.com
[MultiMC]: https://multimc.org
[packwiz]: https://github.com/packwiz/packwiz
[Modrinth format]: https://docs.modrinth.com/docs/modpacks/format_definition/
[documentation for creating modpacks]: https://docs.modrinth.com/docs/modpacks/creating_modpacks/
[documentation for playing modpacks]: https://docs.modrinth.com/docs/modpacks/playing_modpacks/
[`packwiz cf import`]: https://packwiz.infra.link/reference/commands/packwiz_curseforge_import/
[`packwiz mr export`]: https://packwiz.infra.link/reference/commands/packwiz_modrinth_export/
[Discord]: https://discord.gg/EUHuJHt
