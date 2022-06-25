# Omorphia

_The Modrinth frontend library_

---

Omorphia is Modrinth's component, style, and utility library for Svelte projects. It includes:

- 🧩 Typed components which enhance HTML elements and provide a consistent UI
- 🎨 CSS classes to easily style elements with a coherent style
- 🧰 Typed utilities to solve common tasks quick and dependably
- ⚙️ Configuration for SvelteKit and PostCSS to simplify setups
- 🚚 A Rollup plugin to generate a cache of heavily used API requests and OpenAPI typ

Read the documentation at [omorphia.modrinth.com.](https://omorphia.modrinth.com)

## Developing

The library lives in the `src/` folder, and the documentation lives in the `docs/` folder.

### Getting started

Install [Node (16.5+)](https://docs.volta.sh/guide/getting-started) and [PNPM](https://pnpm.io/installation) prior to developing.

To start the dev server, install dependencies and run `pnpm dev`:

```bash
pnpm install # Install dependencies
pnpm dev # Run dev server
```

To use the git hooks in the repo, which will save you waiting for CI to tell you that you forgot to lint, run this:

```bash
git config core.hookspath .githooks
```

### Adding new components

> Replace `ComponentName` with your component name in the steps below

1. Create a `ComponentName.svelte` file in `src/components`
2. Add an export for your component in [./src/index.ts](./src/index.ts)
   ```
   export { default as ComponentName } from './components/ComponentName.svelte'
   ```
3. Create a `ComponentName.md` file for documentation in `docs/routes/components`
4. Add an example of your component in your `ComponentName.md` file, like so:

   ````md
   ```svelte example raised
   <script lang="ts">
   	import { ComponentName } from 'omorphia'
   </script>

   <ComponentName />
   ```
   ````

## Building

To build the documentation site, run:

```bash
pnpm build
```
