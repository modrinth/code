---
title: Setup
---

## `0.` Prerequisites

First install the following:

- [Node 16.x](https://docs.volta.sh/guide/getting-started) or higher
- [PNPM](https://pnpm.io/installation) (required for Modrinth projects)

## `1.` Create a SvelteKit project

Run the following command to create a SvelteKit project:

```bash
pnpm create svelte
```

Follow the instructions to install dependencies and setup git.

## `2.` Add Omorphia to your project

```bash
pnpm add omorphia
```

## `3.` Setup translations

Install the translations submodule:

```bash
git submodule add https://github.com/modrinth/translations locales/
```

Install `svelte-intl-precompile`:

```bash
pnpm add svelte-intl-precompile -D
```

Add translations in `src/routes/__layout.svelte`:

```html
<script context="module" lang="ts">
	import { init, waitLocale, t, getLocaleFromAcceptLanguageHeader } from 'svelte-intl-precompile'
	import { registerAll, availableLocales } from '$locales'

	registerAll()

	export const load: import('@sveltejs/kit').Load = async ({ session }) => {
		init({
			fallbackLocale: 'en',
			initialLocale: getLocaleFromAcceptLanguageHeader(session.acceptLanguage, availableLocales),
		})
		await waitLocale()

		return {}
	}
</script>
```

## `4.` Configure SvelteKit

Add the following parts to your `svelte.config.js` file:

```js
import adapter from '@sveltejs/adapter-auto'
import { preprocess } from 'omorphia/config/svelte'
import path from 'path'

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: [preprocess],

	kit: {
		adapter: adapter(),

		alias: {
			$generated: path.resolve('./generated'),
			$stores: path.resolve('./src/stores'),
		},
	},
}

export default config
```

Create a `src/stores/account.ts` file with a `token` store export:

```ts
import { writable } from 'svelte/store'

export const token = writable('')
```

## `5.` Configure Vite

Add the following to your `vite.config.js` file:

```js
import { plugins } from 'omorphia/config/vite.js'
import { Generator } from 'omorphia/plugins'
import precompileIntl from 'svelte-intl-precompile/sveltekit-plugin'

/** @type {import('vite').UserConfig} */
const config = {
	plugins: [
		...plugins,
		precompileIntl('locales'),
		Generator({
			gameVersions: true,
			openapi: true,
			// Add more if needed
		}),
	],

	server: {
		fs: {
			allow: ['generated'],
		},
	},
}

export default config
```

## `6.` Configure PostCSS

Create a `postcss.config.cjs` file in the root of your project.

Add the following line to that file:

```js
module.exports = require('omorphia/config/postcss.cjs')
```

## `7.` Setup styles

Import styles in `src/routes/__layout.svelte`:

```html
<script lang="ts">
	import 'omorphia/styles.postcss'
</script>
```

Add the `base` class and a theme to the `<body>` tag in `src/app.html`:

```html
<body class="base theme-light">
	%sveltekit.body%
</body>
```

## `8.` Setup fonts

Copy the the `fonts/` folder from [Omorphia's repository](https://github.com/modrinth/omorphia/blob/main/docs/static/assets/fonts) and place them in the `static/` folder at the root of your project.

Add the following preload tags to your head in `app.html` to speed up font loading:

<!-- prettier-ignore-start -->
```html
<head>
  <link rel="preload" href="/fonts/InterRegular.woff2" as="font" type="font/woff2" crossorigin>
  <link rel="preload" href="/fonts/InterBold.woff2" as="font" type="font/woff2" crossorigin>
  <link rel="preload" href="/fonts/InterSemiBold.woff2" as="font" type="font/woff2" crossorigin>
</head>
```
<!-- prettier-ignore-end -->

## `9.` Using Omorphia

### Developing

Start the development server with:

```bash
pnpm dev
```

> To get Svelte language support in your code editor, [use this list of extensions.](https://sveltesociety.dev/tools#editor-support)

### Components

Use a component by importing from `omorphia`. For example, use the [Button component](/components/Button) like so:

```svelte example raised
<script lang="ts">
	import { Button } from 'omorphia'
</script>

<Button color="primary">I'm a button!</Button>
```

### Utils

Use a utility by importing from `omorphia/utils`.

```svelte example raised
<script lang="ts">
	import { ago } from 'omorphia/utils'
</script>

{ago(Date.now() - 100000)}
```

### Using icons and styles

Follow the guides on the sidebar to learn how to use [icons](/usage/icons) and general concepts.
