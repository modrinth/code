# Copying Guidelines

This repository is a **fork of [modrinth/code](https://github.com/modrinth/code)** rebranded and extended as **Owyx**. Package source licenses are **unchanged** from upstream — see each package’s `LICENSE` / `LICENSE.txt` (the desktop app and related packages are **GNU GPL v3**). A copy of GPL-3 is also at the repository root [`LICENSE`](./LICENSE).

For detailed information, consult each package's `COPYING.md`, `LICENSE.txt`, or `LICENSE` file, if available.

You may fork, modify, and redistribute the **source code** under the applicable package licenses (typically GPL-3 for the launcher). That license covers the software — **not** the product brands below.

---

## Modrinth Branding

The use of Modrinth branding elements, including but not limited to the wrench-in-labyrinth logo, the landing image, and any variations thereof, is strictly prohibited without explicit written permission from Rinth, Inc. This includes trademarks, logos, or other branding elements.

> All rights reserved. © 2020-2025 Rinth, Inc.

Upstream listed (among others) these trademark assets; **they are removed from this fork** and must not be restored:

- `.idea/icon.svg`
- `.github/assets/api_cover.png`
- `.github/assets/app_cover.png`
- `.github/assets/monorepo_cover.png`
- `.github/assets/web_cover.png`

If you fork this repository further, you must continue to omit Modrinth branding assets.

---

## Owyx Branding

**Owyx** product branding is **not** licensed under GPL-3 for reuse in forks, derivatives, or redistributed builds. Source code may be used under GPL-3; the Owyx name, marks, and assets may not.

### Where branding lives

| Location | Contents |
|----------|----------|
| [`brand/`](./brand/) | Canonical design system, logos, icons, wordmarks, hero art (`DESIGN.md`, `v1/`, `v2/`) |
| [`apps/app/icons/`](./apps/app/icons/) | Launcher install icons (Windows/macOS/Linux) |
| [`apps/app/dmg/`](./apps/app/dmg/) | macOS DMG artwork |
| [`owyxsite/frontend/public/`](./owyxsite/frontend/public/) | Site favicons and public brand files |
| Launcher / site strings & themes | Product name **Owyx**, cyan accent `#00e5ff`, dark cosmic palette, Sora wordmark usage |

### What you must not do without written permission from the Owyx maintainer

- Use the name **Owyx**, **Owyx Team**, **owyx.site**, **api.owyx.site**, or confusingly similar names as the product name of your fork, binary, installer, website, or Discord/community.
- Ship or redistribute files from [`brand/`](./brand/) (or copies of those marks) in a product that is not an official Owyx build.
- Use Owyx logos, crystal icons, wordmarks, Discord RPC art, installer icons, or site favicons as your own brand.
- Imply affiliation with, endorsement by, or identity as the official Owyx project or Owyx Team.

### What forks should do

1. Replace product name, window titles, installer metadata, update endpoints, and API hosts with your own.
2. Replace icons under `apps/app/icons/` and any site favicons with your own artwork.
3. Do **not** copy or redistribute [`brand/`](./brand/) — leave it out of your public product, or delete it from your fork’s release artifacts.
4. Keep omitting Modrinth branding as required above.
5. Keep GPL-3 notices and upstream attribution for code you redistribute.

> Owyx branding: all rights reserved. © Owyx Team. Code: see package licenses / root [`LICENSE`](./LICENSE).

Official design tokens and asset inventory: [`brand/DESIGN.md`](./brand/DESIGN.md), [`brand/README.md`](./brand/README.md).
