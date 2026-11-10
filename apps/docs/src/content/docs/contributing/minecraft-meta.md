---
title: Minecraft metadata
description: Guide to Modrinth's Minecraft metadata generator and shared models
sidebar:
  order: 5
---

Modrinth's `meta-generator` queries and generates metadata for Minecraft and mod loaders for:

- Performance (Serving static files can be easily cached and is extremely quick)
- Ease for Launcher Devs (Metadata is served in an easy to query and use format)
- Reliability (Provides a versioning system which ensures no breakage with updates)

The generator supports the original Minecraft data and reposting for the Forge, Fabric, Quilt, and NeoForge loaders.

The code lives in `apps/meta-generator`, with shared metadata models in `packages/minecraft-meta-model` and manifest types in `packages/minecraft-meta-manifest`.
