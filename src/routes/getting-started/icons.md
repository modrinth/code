---
title: Icons
---

## Choosing icons

The follwing icon packs are included with omorphia:

`heroicons-outline` `lucide` `fa-regular` `heroicons-solid` `carbon` `simple-icons`

Aim to find icons from `heroicons-outline` first, and then from the following packs if you can't find what you are looking for. [Browse icons...](https://icones.js.org/collection/heroicons-outline)

## Using icons

Import an icon in the `<script>` tag of your component.

Then use the icon as if it were a Svelte component:

> You can style the icon with the `.icon` class. Note, you will have to use the `:global(.icon)` selector in Svelte components.

```svelte example
<script lang="ts">
	import IconHeart from 'virtual:icons/heroicons-outline/heart'
</script>

<p>That's lovely! <IconHeart /></p>
```
