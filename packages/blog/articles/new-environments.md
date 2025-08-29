---
title: 'Creators: Verify Your Environment Metadata'
summary: We've overhauled the environment metadata on Modrinth, and all creators must verify their settings.
date: 2025-08-28T16:50:00-07:00
authors: ['Dc7EYhxG']
---

**Hey creators!**

Over the years, we've taken in lots of feedback regarding how we identify client-side and server-side mods and modpacks on Modrinth. It's a surprisingly nuanced issue, and careful consideration has finally led us to implementing a new system that fixes many of the issues with the old one.

## What do I need to do?

If you want to jump right into what you need to do now, just visit your [Projects page](/dashboard/projects) and look for any of your mod or modpack projects with an orange warning button next to the settings button. This will take you to the new Environments page in your project's settings with a bunch of different options for configuring your project's environment.

![Screenshot of the new Modrinth Environment settings page. A warning message explains that the environment system has been updated. Options are listed with radio buttons, including Client-side only (selected), Server-side only, Client and server, and Singleplayer only, each with descriptions and sub-options.](./env-settings.webp)

**If you do not verify your environments, in the future a warning may display on your project to inform users that the environment information may be outdated or incorrect.**

Read on to learn more about why we've made this change and for a more thorough explanation of each option, in case you find any of them confusing.

## What was wrong with the old system?

Originally, Modrinth's environment metadata came in the form of two fields for Client-side and Server-side compatibility. Each option could be set to 'Required', 'Optional', or 'Unsupported'. This did the job, but it left some ambiguities and led to many confused creators mis-labeling their mods.

1. **Certain combinations of options don't make logical sense**, such as both sides being 'Unsupported', or one side being 'Optional' and the other being 'Unsupported'. To some people, they may feel that since _installing_ the mod is optional, that might be a logical choice, when users and automated tools might be expecting it to be labeled as 'Required'
2. **Terms like 'Unsupported' are interpreted differently** by different people. If something is 'Unsupported' on the server-side, does that mean it crashes when installed on a server?
3. **Most server-side only mods also work in Singleplayer**, even if they don't perform any functions on the client-side directly. Some creators of server-sided mods chose to mark client-side as 'Optional' because of this, even if it did absolutely nothing on the client-side because in order to use it in Singleplayer, you technically install it on the "client"
4. **Not all real-world combinations even could be represented** by this old system. There are some mods that only make sense in a singleplayer environment, or some that only make sense on dedicated servers and _not_ in Singleplayer.
5. **Conflicting information is out there** on what exactly these terms meant. The website told creators to treat the client and server as the _logical_ client and servers, but some other people's guides and tooling treated them as referring to the _physical_ client and server. This includes the Modrinth Pack (.mrpack) specification, which confusingly uses the same required/optional/unsupported terminology to refer to the physical sides when defining which files should be installed in the client and server distributions.

## How does the new system work?

The new system enumerates all expected use-cases into distinct options that can be handled in unique ways by tools like launchers, mod managers, and modpack assemblers.

The new options are as follows:

- **Client-side only** (`client_only`)
  - All functionality is performed exclusively on the client side. Should be compatible with vanilla servers.
  - Example: [Mod Menu](/mod/modmenu). It only adds a menu to view the list of mods installed on your client, which doesn't need to be installed on the server.
- **Server-side only / Works in singleplayer** (`server_only`)
  - All functionality is performed exclusively on the server side. Should be compatible with vanilla clients if only installed on the server. Also works in Singleplayer.
  - Example: [YUNG's Bridges](/mod/yungs-bridges). It only adds structures which don't need to be present on the client-side.
- **Server-side only / Dedicated server only** (`dedicated_server_only`)
  - Only runs on a dedicated server, and not in Singleplayer.
  - Example: [Better Fabric Console](/mod/better-fabric-console). Its functionality does not work in singleplayer, because it modifies the dedicated server console.
- **Client and server / Required on both** (`client_and_server`)
  - Must be installed on both the client and server.
  - Example: [Cobblemon](/mod/cobblemon). It adds entities, blocks, and items that need to be on both the client and server to work.
- **Client and server / Optional on client** (`server_only_client_optional`)
  - Must be on the server, but can be on the client as well for enhanced functionality
  - Example: [Polymer](/mod/polymer). It functions on the server-side, but if installed on the client it can improve the experience when playing on a server running Polymer.
- **Client and server / Optional on server** (`client_only_server_optional`)
  - Must be on the client, but can be on the server as well for enhanced functionality
  - Example: [AppleSkin](/mod/appleskin). It functions on the client-side, but if installed on the server it can provide more accurate saturation information.
- **Client or server / Works best on both** (`client_or_server_prefers_both`)
  - Can be installed on just the client or just the server to function, but functionality is enhanced when it is on both.
  - Example: [No Chat Reports](/mod/no-chat-reports). The mod functions on just the client or just the server, but each comes with drawbacks. For the best functionality, you need to install it on both.
- **Client or server / Works the same on either** (`client_or_server`)
  - Can be installed on just the client or just the server, and either one would enable full functionality. There would be no reason to install it on both.
  - Example: [Entity View Distance](/mod/entity-view-distance). It lets you perform the same functionality of limiting entity view distance on either the client or the server.
- **Singleplayer only** (`singleplayer_only`)
  - Only works in Singleplayer, does not function in a Multiplayer environment.
  - Example: [LAN Server Properties](/mod/lan-server-properties). It modifies a feature that only exists in Singleplayer, the Open to LAN menu.

## What's next

This is a great first step towards us fixing many common issues that have been affecting Modrinth users, such as:

- Client-side mods being installed to Modrinth Servers, causing crashes
- Modpack exporting in Modrinth App and other launchers using the Modrinth API such as Prism Launcher, MultiMC, and ATLauncher not having accurate and reliable metadata to pull from in order to build universal client and server Modrinth Pack files.

However, this is just the first step. Before we can improve the tooling around creating and using modpacks, we need as many Modrinth projects as possible to have accurate metadata.

Currently, the new environment metadata is also only available in the experimental API v3, which is _not_ intended for general use. When we're ready, we plan to integrate this metadata into API v2, so that it can be used in production by third parties. For now, developers using the Modrinth API should not worry about these environment changes, just keep them in mind for the future.

Thank you all for continuing to support Modrinth!

**Prospector**\
_Founding Software Engineer_
