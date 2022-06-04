---
title: Using illustrations
---

Find an illustration from [unDraw](https://undraw.co/illustrations) and download it as an SVG.

Put the illustration in the `src/assets/images/illustrations` folder. Rename it to `undraw_` + the illustration slug.

Replace colors in the SVG with CSS variables such as `var(--color-brand)` and `var(--color-raised)`. For colors that are the same as the font color, use `currentColor`.

Add the `.illustration` class to the SVG

Import the SVG in the `<script>` of your svelte file, and treat the illustration as a Svelte component:

```svelte
<script>
	import NoData from '$assets/images/illustrations/undraw_no_data.svg'
</script>

<NoData />
```
