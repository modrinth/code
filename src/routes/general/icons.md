<svelte:head>
    <title>Icons - Kleos</title>
</svelte:head>

# Icons

Use [unplugin-icons](https://github.com/antfu/unplugin-icons) to import icons as Svelte components.

## Installation

Install the package with:

```bash
pnpm add -D unplugin-icons
```

Import the plugin in your `svelte.config.js` file:

```js
import Icons from 'unplugin-icons/vite';

const config = {
    kit: {
        vite: {
            plugins: [
                Icons({
                    compiler: 'svelte',
                }),
            ],
        },
    },
};
```

## Choosing icons

`heroicons-outline` is the most used icon pack, and should be preferred above all others. If you can't find a good icon, or are looking for an alternative, use `lucide`, `fa-regular` (FontAwesome), `heroicons-solid`, and `carbon`, in that order. [Browse icons...](https://icones.js.org/collection/heroicons-outline)

To install an icon pack, run:

```bash
pnpm add -D @iconify-json/heroicons-outline
```

Replacing `heroicons-outline`, with the pack you are trying to install.

## Using icons

Import an icon in the `<script>` tag of your component:

```js
  import IconHeart from 'virtual:icons/heroicons-outline/heart'
```

Then use the icon as if it were a Svelte component:

```html
<IconHeart />
```