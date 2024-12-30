---
title: Minotaur (Gradle plugin)
description: Guide for contributing to Modrinth's gradle plugin
---

[Minotaur][minotaur] is the Gradle plugin used to automatically publish artifacts to Modrinth. To run your copy of the plugin in a project, publish it to your local Maven with `./gradlew publishToMavenLocal` and add `mavenLocal()` to your buildscript.

Minotaur contains two test environments within it - one with ForgeGradle and one with Fabric Loom. You may tweak with these environments to test whatever you may be trying; just make sure that the `modrinth` task within each still functions properly. GitHub Actions will validate this if you're making a pull request, so you may want to use [`act pull_request`](https://github.com/nektos/act) to test them locally.

[minotaur]: https://github.com/modrinth/minotaur
