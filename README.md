# Omorphia

*The Modrinth component library, in Svelte*

---

### ⚠️ Currently in-progress ⚠️

When creating a component, start with [GitHub's Primer styles](https://github.com/primer/css/tree/main/src) for that component, and go from there. Global variables and colors are incomplete and currently identical to Primers'.

#### Components

- [ ] Button
- [ ] Select menu
- [ ] Checkboxes
- [ ] Text input
- [ ] Pagination
- [ ] Link
- [ ] Blankslate
- [ ] Breadcrumbs
- [ ] Markdown
- [ ] Textarea
- [ ] Select

...and others

## Using

Install the package:

```bash
npm install omorphia # or pnpm inst...
```

Import a component:
```svelte
<script>
    import { Button } from "omorphia"
</script>

<Button> Click me! </Button>
```

## Developing

The library lives in the `src/lib` folder, and the documentation lives in the `src/routes` folder.

```bash
pnpm install # Install dependencies
pnpm dev # Run dev server
```

## Packaging

```bash
pnpm package
```

## Building

To build the documentation site, run:

```bash
pnpm build
```