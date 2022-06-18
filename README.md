# Omorphia

_The Modrinth component library, in Svelte_

---

Read the documentation at [omorphia.modrinth.com.](https://omorphia.modrinth.com)

Requires Node v16.5+.

## Developing

The library lives in the `package/` folder, and the documentation lives in the `docs/` folder.

```bash
pnpm install # Install dependencies
pnpm dev # Run dev server
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

## Packaging

```bash
pnpm package
```

## Building

To build the documentation site, run:

```bash
pnpm build
```
