---
title: Knossos (Frontend)
description: Guide for contributing to Modrinth's frontend
---

This project is part of our [monorepo](https://github.com/modrinth/code). You can find it in the `apps/frontend` directory.

[knossos] is the Nuxt.js frontend. You will need to install [pnpm] and run the standard commands:

```bash
pnpm install
pnpm run web:dev
```

Once that's done, you'll be serving knossos on `localhost:3000` with hot reloading. You can replace the `dev` in `pnpm run dev` with `build` to build for a production server and `start` to start the server. You can also use `pnpm run lint` to find any eslint problems, and `pnpm run fix` to try automatically fixing those problems.

<details>
<summary>.env variables & command line options</summary>

#### Basic configuration

`SITE_URL`: The URL of the site (used for auth redirects). Default: `http://localhost:3000`
`BASE_URL`: The base URL for the API. Default: `https://staging-api.modrinth.com/v2/`
`BROWSER_BASE_URL`: The base URL for the API used in the browser. Default: `https://staging-api.modrinth.com/v2/`

</details>

#### Ready to open a PR?

If you're prepared to contribute by submitting a pull request, ensure you have met the following criteria:

- `pnpm run fix` has been run.

[knossos]: https://github.com/modrinth/code/tree/main/apps/frontend
[pnpm]: https://pnpm.io
