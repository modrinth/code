---
title: Modrinth Website
description: Guide for contributing to Modrinth's frontend
sidebar:
  order: 2
---

The [Modrinth Website], codename Knossos, is a Nuxt.js frontend. You will need to install [pnpm] and run the standard commands:

## Setup

### 1. Install prerequisites

- Install the package manager [pnpm](https://pnpm.io/)

### 2. Install dependencies & set up .env

- Clone [`https://github.com/modrinth/code`](https://github.com/modrinth/code) and run `pnpm install` in the workspace root folder.
- In `apps/frontend` you should be able to see `.env.prod`, `.env.staging` — for basic work, it's recommended to use `.env.prod`. Copy the relevant file into a new `.env` file within the `apps/frontend` folder.

### 3. Run the frontend

- Run `pnpm web:dev` in the workspace root folder. Once that's done, you'll be serving the website on `localhost:3000` with hot reloading.

## Ready to open a PR?

If you're prepared to contribute by submitting a pull request, ensure you have met the following criteria:

- `pnpm prepr:frontend` has been run.

[Modrinth website]: https://github.com/modrinth/code/tree/main/apps/frontend
[pnpm]: https://pnpm.io
