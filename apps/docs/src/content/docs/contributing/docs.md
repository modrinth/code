---
title: Docs
description: Guide for contributing to Modrinth's developer documentation
---

This project is part of our [monorepo](https://github.com/modrinth/code). You can find it in the `apps/docs` directory.

[Docs] are the very site you are looking at right now.
They are here to help developers and contributors work with Modrinth's codebase and API.

To set up a development environment, you will need to install [pnpm] and [git], if you haven't already, and run the following commands:

```bash
git clone https://github.com/modrinth/code.git modrinth
cd "modrinth"
pnpm install
pnpm run docs:dev
```

When ready, you will have a hot-reloading environment of the docs site running on port 4321.
To open it up, enter http://localhost:4321 on your Browser, or click [here].

## Ready to open a PR?

While there is no linting requirement on Docs, we do ask that you quickly check your writing before contributing.

[docs]: https://github.com/modrinth/code/tree/main/apps/docs
[pnpm]: https://pnpm.io
[git]: https://git-scm.com/
[here]: http://localhost:4321/
