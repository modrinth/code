# Figma MCP Usage

When the Figma MCP server is connected, use it to translate Figma designs into production-ready Vue components for this monorepo.

## Workflow

### 1. Get the design context

Use `get_design_context` with the node ID from a Figma URL. If the URL is `https://figma.com/design/:fileKey/:fileName?node-id=1-2`, the node ID is `1:2`.

```
get_design_context(nodeId: "1:2", clientLanguages: "typescript,html,css", clientFrameworks: "vue")
```

This returns reference code, a screenshot, and metadata. Always start here.

### 2. Get a screenshot for visual reference

Use `get_screenshot` if you need to see the design without full code context:

```
get_screenshot(nodeId: "1:2")
```

### 3. Get variable definitions

Use `get_variable_defs` to see what design tokens are applied to a node:

```
get_variable_defs(nodeId: "1:2")
```

### 4. Get metadata for structure overview

Use `get_metadata` to get an XML overview of node IDs, layer types, names, positions and sizes — useful for understanding the structure of a complex frame before diving into individual nodes.

## Adapting Figma Output

The Figma MCP returns generic reference code. Adapt it to match the Modrinth codebase:

1. **Read `packages/ui/CLAUDE.md`** for color usage rules, surface token mapping, and component patterns.
2. **Map Figma color variables to `surface-*` tokens** — never use Figma's aliased names like `bg/default` or `bg/raised` directly. The CLAUDE.md has the full mapping table.
3. **Check `packages/assets/styles/variables.scss`** for tokens not exposed in Figma (brand highlights, semantic backgrounds, shadows).
4. **Check for existing components** in `packages/ui/src/components/` before building from scratch.
5. **Match spacing exactly** — do not approximate values from the design.
