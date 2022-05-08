---
title: Configure
---

To make use of the built-in icons, styles, and plugins in omorphia, you will need to update your project's config files.

## SvelteKit 

Add the following parts to your `svelte.config.js` file:

```js
import { preprocess, plugins } from 'omorphia/config/svelte.config'

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: [
        preprocess,
    ],

    kit: {
        vite: {
            plugins: [
                ...plugins,
            ],
        },
    },
};

export default config;
```

## PostCSS

Create a `postcss.config.cjs` file in the root of your project.

Add the following line to that file:

```js
module.exports = require('omorphia/config/postcss.config.cjs')
```