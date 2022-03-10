<svelte:head>
    <title>Introduction - Kleos</title>
</svelte:head>

# Introduction

> Kleos is in early development, and not ready for use in any application. Contribute to it on [GitHub](https://github.com/modrinth/kleos).

### What is Kleos?

Kleos is Modrinth's style and reusable component library for use in all of its frontend applications, including [knossos](https://github.com/modrinth/knossos) (modrinth.com), [theseus](https://github.com/modrinth/theseus) (launcher), and planned projects such as Modrinth's in-house auth and ad-server.

It uses [Svelte](https://svelte.dev/) to deliver the best performance with the least boilerplate.

### Getting started

Adding Kleos to your project is as easy as:

```bash
pnpm add kleos
```

#### Components

Import a component with:
```js
import { Button } from "kleos"
```

Then, use it in your HTML:

```html
<Button color="primary">Click me!</Button>
```

For more information on each component, check out the pages on the sidebar navigation.

> To get Svelte language support in your code editor, [use this list.](https://sveltesociety.dev/tools#editor-support)

#### Icons, Styles, and more

Follow the guides on the sidebar to learn how to use [icons](/getting-started/icons) and general concepts.