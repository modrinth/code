---
title: 'Changes to Modrinth Modpacks'
summary: 'CurseForge CDN links requested to be removed by the end of the month'
date: 2022-05-28
---

CurseForge CDN links requested to be removed by the end of the month

Modrinth's alpha launch of modpacks has been highly successful in the nearly two weeks it has been live, with over forty
packs launched to the platform. However, a number of these packs include links to download mods from CurseForge's CDN,
which has caught the attention of CurseForge. On May 24th, 2022, a representative from Overwolf sent email correspondence
to us demanding us to remove all modpacks and documentation that contain references to CurseForge CDN links by the end
of the month. The message was vague, and didn't specify whether or not they were making a legal threat against us or not,
so we responded in attempt to clarify what would happen if we chose not to comply. In response, they told us that they
would "consider next steps."

Modrinth has every intention of complying with their demands, despite our belief that this is a huge loss for the
community. However, CurseForge's immediate "next steps" were to message launcher developers, requesting that they break
support for Modrinth packs that contain CurseForge CDN links, and claiming to them that we outright refused to remove the
packs containing the links from our platform ourselves when we did not refuse.

To be clear, Modrinth condemns the anti-competitive behaviors that CurseForge are engaging in, however, we do not wish
for CurseForge or authors who have elected to opt-out of third party downloads from their platform to be our enemies.
Modrinth is and will always remain a project in support of open source software, with open and free APIs for all to use,
and encouraging of much needed competition and diversity in the mod hosting space.

Unfortunately, in order to comply with their request, all Modrinth modpacks must now use override JARs in place of any
links to CurseForge's CDN. Specifically, CDN links to `edge.forgecdn.net` and `media.forgecdn.net` will no longer be part
of the `.mrpack` [specification](https://docs.modrinth.com/docs/modpacks/format_definition/#downloads), effective today.
Of course, modpack authors must ensure that they are properly licensed to redistribute any mods that are not hosted on
the Modrinth platform. While this is a huge blow to modpack creators and users of our platform for now, relying on
CurseForge CDN links has always been unreliable as a long-term solution, because they could choose to change the links
at any time, and it leaves variables outside of our control. In the long run, packs containing mostly mods hosted on
Modrinth will be better for the growth of our platform and for the stability of modpacks.

In order to use mods exclusively hosted on CurseForge as override JARs, pack creators must ensure that either of the
following conditions must be met:

1. The mod is licensed under terms that allow for redistribution. The pack author is responsible for following the terms of the license.
2. General or individual permission is granted from the mod author. This can be in the form of a message from the author or a statement made on a mod's project description granting permission to use it in modpacks.

In order to aid in this process, Modrinth will be building a third party mod license database and automated tools that
will help pack creators with the hassle that will be ensuring all of the mods in their packs are properly licensed.
In addition, packs will continue to be hand-reviewed by Modrinth moderation staff and verified. Do note that in this
transition time, the review process for modpack projects may experience significant delays. Authors of existing modpacks
on the platform will be reached out to in order to help them convert their existing packs to compliant packs.

For those wondering, our next steps as a company are:

1. Mod license database for Modpack authors
2. Creator monetization
3. The Modrinth launcher for downloading and creating modpacks.
