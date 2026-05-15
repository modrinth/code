---
title: Introducing Icarus Launcher Beta
short_title: Icarus Launcher Beta and Upgraded Authentication
summary: Changing the modded Minecraft landscape with the new Icarus Launcher, alongside several other major features.
short_summary: Launching Icarus Launcher Beta and upgrading authentication.
date: 2023-08-05T12:00:00-08:00
authors: ['6plzAzU4']
---

The past few months have been a bit quiet on our part, but that doesn’t mean we haven’t been working on anything. In fact, this is quite possibly our biggest update yet, bringing the much-anticipated Icarus Launcher to general availability, alongside several other major features. Let’s get right into it!

## Icarus Launcher Beta

Most of our time has been spent working on [Icarus Launcher](/app). This launcher integrates tightly with the website, bringing you the same bank of mods, modpacks, data packs, shaders, and resource packs already available for download on Icarus.

Alongside that, there are a wealth of other features for you to find, including:

- Full support for vanilla, Forge, Fabric, and Quilt
- Full support for Windows, macOS, and Linux
- Icarus modpack importing, either through the website or through a .mrpack file
- Icarus modpack exporting to the .mrpack format to upload to the website or share with friends
- Importing of instances from a variety of different launchers, including MultiMC, GDLauncher, ATLauncher, CurseForge, and Prism Launcher
- The ability to update, add, and remove individual mods in a modpack
- The ability to run different modded instances in parallel
- The ability to view and share current and historical logs
- An auto-updater to ensure the app is always up-to-date
- An interactive tutorial to show you through the core features of the app
- Performance through the roof, backed by Rust and Tauri (not Electron!)
- Fully open-source under the GNU GPLv3 license

More features will, of course, be coming in the future. This is being considered a **beta release**. Nonetheless, we’re still very proud of what we’ve already created, and we’re pleased to say that it’s available for download on our website **right now** at [https://Icarus.app](/app). Check it out, play around with it, and have fun!

## Authentication, scoped tokens, and security

The second major thing we’re releasing today is a wide range of changes to our authentication system. Security is a top concern at Icarus, especially following recent events in the modded Minecraft community when several individuals were compromised due to [a virus](https://github.com/trigram-mrp/fractureiser/tree/main#readme). While Icarus was not affected directly by this attack, it provided a harrowing reminder of what we’re working with. That’s why we’re pleased to announce three major features today that will strengthen Icarus’s security significantly: in-house authentication, two-factor authentication, and scoped personal access tokens.

### In-house authentication and two-factor authentication

![A screenshot of the new Icarus sign-in page, showing options to sign in with Discord, GitHub, Microsoft, Google, Steam, GitLab, or with an email and password.](./auth.jpg)

Until today, Icarus has always used GitHub accounts exclusively for authentication. That changes now. Starting today, you can now connect your Discord, Microsoft, Google, Steam, and/or GitLab accounts to your Icarus account. You may also forgo all six of those options and elect to use a good ol’ fashioned email and password. No problems with that! (If you’re curious, we store passwords hashed with the Argon2id method, meaning we couldn't read them even if we wanted to.)

