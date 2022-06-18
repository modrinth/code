---
title: Introduction
---

## What is Omorphia?

Omorphia is Modrinth's style and reusable component library for use in all of its frontend applications, including [knossos](https://github.com/modrinth/knossos) (modrinth.com), [theseus](https://github.com/modrinth/theseus) (launcher), and planned projects such as Modrinth's in-house auth and ad-server.

It uses [Svelte](https://svelte.dev/) to deliver the best performance with the least boilerplate.

## Getting started

Adding Omorphia to your project is as easy as:

```bash
pnpm add omorphia
```

### Components

Use a component by importing from `omorphia`. For example, use the [Button component](/components/Button) like so:

```svelte example raised
<script lang="ts">
	import { Button } from 'omorphia'
</script>

<Button color="primary">I'm a button!</Button>
```

For more information on each component, check out the pages on the sidebar navigation.

> To get Svelte language support in your code editor, [use this list of extensions.](https://sveltesociety.dev/tools#editor-support)

### Using icons and styles

Follow the guides on the sidebar to learn how to use [icons](/getting-started/icons) and general concepts.
