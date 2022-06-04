---
title: Icons
---

## Choosing icons

The follwing icon packs are included with omorphia:

`heroicons-outline` `lucide` `fa-regular` `heroicons-solid` `carbon`

Aim to find icons from `heroicons-outline` first, and then from the following packs if you can't find what you are looking for. [Browse icons...](https://icones.js.org/collection/heroicons-outline)

## Using icons

Import an icon in the `<script>` tag of your component.

Then use the icon as if it were a Svelte component:

> The `height` and `width` props are optional, and take CSS compatible values

```svelte example
<script lang="ts">
	import IconHeart from 'virtual:icons/heroicons-outline/heart'
</script>

<p>That's lovely <IconHeart height="14px" />!</p>
```
