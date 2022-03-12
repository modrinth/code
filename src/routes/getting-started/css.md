---
title: CSS Configuration
---

Use [PostCSS](https://postcss.org/) to process your css in components and `.postcss` files.

Install PostCSS with:

```bash
pnpx svelte-add@latest postcss
```

This is the recommended `postcss.config.cjs` setup:

```js
const config = {
  plugins: [
    require('postcss-import'),
    require('postcss-strip-inline-comments'),
    require('postcss-nested'),
    require('postcss-preset-env'),
    require('autoprefixer'),
    require('postcss-extend-rule'),
    process.env.NODE_ENV === 'development' && require('cssnano')({
      preset: 'default',
    })
  ],
};

module.exports = config;
```