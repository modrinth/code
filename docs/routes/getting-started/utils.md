---
title: Built-in utilities
---

## Markdown

Use the markdown utilities to parse markdown text into HTML. Both markdown parsers have HTML sanitization built-in.

### Body parser

The `markdown` parser is designed for bodies of markdown text and supports images, tables, lists, and youtube `<iframe>`s.

> Use the `.markdown` class on the element containing your parsed markdown.

````svelte example
<script lang="ts">
	import { markdown } from 'omorphia/utils'

	const source =
		'## Example markdown \n\
  This is **some** *text*! \n\
  `console.log("test")` \n\
  #### An image \n\
  <img src="https://cdn.modrinth.com/data/YL57xq9U/images/d382106b9a2b943d06107c31c139c77849f1a0e8.png" />'
</script>

<div class="card markdown">
	{@html markdown(source)}
</div>
<div class="card markdown">
	{@html markdown('test\n```js\nconsole.log("test")\n```')}
</div>
````

### Inline parser

The `markdownInline` parser is perfect for translations and short bios. It doesn't allow complex elements such as images or tables.

```svelte example raised
<script lang="ts">
	import { markdownInline } from 'omorphia/utils'

	const source = 'This is some **bolded** and *italicized* text.'
</script>

<p>{@html markdownInline(source)}</p>
```

## Human readable "ago" times

```svelte example raised
<script lang="ts">
	import { ago } from 'omorphia/utils'
</script>

<p>Something happened {ago(Date.now())}.</p>
<p>Something happened {ago(new Date(Date.now() - 1000 * 60 * 60 * 2))}.</p>
<p>Something happened {ago(new Date(Date.now() - 1000 * 60 * 60 * 24 * 7))}.</p>
<p>Something happened {ago(new Date(Date.now() - 1000 * 60 * 60 * 24 * 7 * 5))}.</p>
```

## Permissions

The `Permissions` class provides an easy way to manage user permissions.

```svelte example raised
<script lang="ts">
	import { Permissions } from 'omorphia/utils'

	const permissions = new Permissions(128) // Can be integer or 'ALL'
</script>

<p>
	<input type="checkbox" bind:checked={permissions.uploadVersions} id="ex-1" />
	<label for="ex-1">Can edit versions</label><br />
	<input type="checkbox" bind:checked={permissions.deleteProject} id="ex-2" />
	<label for="ex-2">Can delete project</label><br /><br />

	Integer: {permissions.toInteger()}<br />
	Can access settings page: {permissions.settingsPage}
</p>
```

## Versions

The `formatVersions` function provides an easy way to parse a project's versions into a readable string.

```svelte example raised
<script lang="ts">
	import { formatVersions } from 'omorphia/utils'
</script>

<p>{formatVersions(['1.18', '1.18.1', '1.18.2', '1.17.1'])}</p>
```
