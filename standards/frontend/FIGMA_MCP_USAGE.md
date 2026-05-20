- [Figma MCP Usage](#figma-mcp-usage)
	- [Available Tools](#available-tools)
	- [Adapting Figma Output](#adapting-figma-output)

# Figma MCP Usage

When the Figma MCP server is connected, it can be used to translate Figma designs into production-ready Vue components for this monorepo.

## Available Tools

| Tool                 | Purpose                                                                                                    |
| -------------------- | ---------------------------------------------------------------------------------------------------------- |
| `get_design_context` | Primary tool. Returns reference code, a screenshot, and metadata for a given node. Always start here.      |
| `get_screenshot`     | Returns a visual screenshot of a node without full code context.                                           |
| `get_variable_defs`  | Returns the design tokens applied to a node.                                                               |
| `get_metadata`       | Returns an XML overview of node IDs, layer types, names, positions, and sizes for understanding structure. |

Node IDs come from Figma URLs. For `https://figma.com/design/:fileKey/:fileName?node-id=1-2`, the node ID is `1:2` (replace `-` with `:`).

```
get_design_context(nodeId: "1:2", clientLanguages: "typescript,html,css", clientFrameworks: "vue")
```

## Adapting Figma Output

The Figma MCP returns generic reference code. It must be adapted to match the Modrinth codebase:

1. **Read `packages/ui/CLAUDE.md`** for color usage rules, surface token mapping, and component patterns.
2. **Map Figma color variables to `surface-*` tokens** — never use Figma's aliased names like `bg/default` or `bg/raised` directly. The CLAUDE.md has the full mapping table.
3. **Check `packages/assets/styles/variables.scss`** for tokens not exposed in Figma (brand highlights, semantic backgrounds, shadows).
4. **Check for existing components** in `packages/ui/src/components/` before building from scratch.
5. **Match spacing exactly** — do not approximate values from the design.
