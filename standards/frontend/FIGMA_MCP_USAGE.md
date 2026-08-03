- [Figma MCP usage](#figma-mcp-usage)
	- [Available tools](#available-tools)
	- [Adapt Figma output](#adapt-figma-output)

# Figma MCP Usage

Use the Figma MCP server to convert Figma designs into production Vue components for this monorepo.

## Available Tools

| Tool                 | Purpose                                                                      |
| -------------------- | ---------------------------------------------------------------------------- |
| `get_design_context` | Get reference code, a screenshot, and node metadata. Always use this first.  |
| `get_screenshot`     | Get a node screenshot without full code context.                             |
| `get_variable_defs`  | Get the design tokens that apply to a node.                                  |
| `get_metadata`       | Get an XML summary of node IDs, layer types, names, positions, and sizes.    |

Get the node ID from the Figma URL. In this example, change `1-2` to `1:2`:

`https://figma.com/design/:fileKey/:fileName?node-id=1-2`

```text
get_design_context(nodeId: "1:2", clientLanguages: "typescript,html,css", clientFrameworks: "vue")
```

## Adapt Figma Output

The Figma MCP server gives generic reference code. Adapt this code to the Modrinth codebase:

1. Read `packages/ui/AGENTS.md` for color rules, surface-token mappings, and component patterns.
2. Map Figma color variables to `surface-*` tokens. Do not use aliased Figma names, such as `bg/default` or `bg/raised`.
3. Read `packages/assets/styles/variables.scss` for tokens that Figma does not show.
4. Search `packages/ui/src/components/` for an applicable component before you make a new component.
5. Use the exact spacing values from the design. Do not use approximate values.
