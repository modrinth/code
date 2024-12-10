---
title: Getting started
description: How can I contribute to Modrinth?
sidebar:
  order: 1
---

# Contributing to Modrinth

Every public-facing aspect of Modrinth, including everything from our [API/backend][labrinth] and [frontend][knossos] to our [Gradle plugin][minotaur] and [launcher][theseus], is released under free and open source licenses on [GitHub]. As such, we love contributions from community members! Before proceeding to do so, though, there are a number of things you'll want to keep in mind throughout the process, as well as some details specific to certain projects.

## Things to keep in mind

### Consult people on Discord

There are a number of reasons to want to consult with people on our [Discord] before making a pull request. For example, if you're not sure whether something is a good idea or not, if you're not sure how to implement something, or if you can't get something working, these would all be good opportunities to create a thread in the `#development` forum channel.

If you intend to work on new features, to make significant codebase changes, or to make UI/design changes, please open a discussion thread first to ensure your work is put to its best use.

### Don't get discouraged

At times, pull requests may be rejected or left unmerged for a variation of reasons. Don't take it personally, and don't get discouraged! Sometimes a contribution just isn't the right fit for the time, or it might have just been lost in the mess of other things to do. Remember, the core Modrinth team are often busy, whether it be on a specific project/task or on external factors such as offline responsibilities. It all falls back to the same thing: don't get discouraged!

### Code isn't the only way to contribute

You don't need to know how to program to contribute to Modrinth. Quality assurance, supporting the community, coming up with feature ideas, and making sure your voice is heard in public decisions are all great ways to contribute to Modrinth. If you find bugs, reporting them on the appropriate issue tracker is your responsibility; however, remember that potential security breaches and exploits must instead be reported in accordance with our [security policy](https://modrinth.com/legal/security).

## Project-specific details

If you wish to contribute code to a specific project, here's the place to start. Most of Modrinth is written in the [Rust language](https://www.rust-lang.org), but some things are written in other languages/frameworks like [Nuxt.js](https://nuxtjs.org) or Java.

Most of Modrinth's code is in our monorepo, which you can find [here](https://github.com/modrinth/code). Our monorepo is powered by [Turborepo](https://turborepo.org).

Follow the project-specific instructions below to get started:

- [Knossos (frontend)](/contributing/knossos)
- [Theseus (Modrinth App)](/contributing/theseus)
- [Minotaur (Gradle plugin)](/contributing/minotaur)
- [Labrinth (API/backend)](/contributing/labrinth)
- [Daedalus (Metadata service)](/contributing/daedalus)
- [Docs (Developer/Contributor Documentation)](/contributing/docs)

[Discord]: https://discord.modrinth.com
[GitHub]: https://github.com/modrinth
[knossos]: https://github.com/modrinth/code/tree/main/apps/frontend
[labrinth]: https://github.com/modrinth/labrinth
[theseus]: https://github.com/modrinth/theseus
[minotaur]: https://github.com/modrinth/minotaur
[docs]: https://github.com/modrinth/code/tree/main/apps/docs
[Rust]: https://www.rust-lang.org/tools/install
[pnpm]: https://pnpm.io
