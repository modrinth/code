---
name: figma-mcp
description: Convert a Figma design into a Modrinth Vue page or component. Use when a request provides a Figma URL or asks to implement a Figma layout.
---

# Implement a Figma Design

Read the applicable `AGENTS.md` files before you edit code.

Read these files in full:

- [Figma MCP usage](../../../standards/frontend/FIGMA_MCP_USAGE.md)
- `packages/ui/AGENTS.md`

1. Get the Figma URL from the request.
2. Extract the `fileKey` and `nodeId`. Replace each `-` in the node ID with `:`.
3. Load any available Figma design-to-code instructions before you call `get_design_context`.
4. Call `get_design_context` first. Use `clientLanguages: "typescript,html,css"` and `clientFrameworks: "vue"`.
5. Map Figma color variables to the applicable `surface-*` and `text-*` tokens.
6. Do not use aliased Figma names directly.
7. Search `packages/ui/src/components/` for applicable components before you make new components.
8. Read `packages/assets/styles/variables.scss` when Figma does not supply a required token.
9. Use exact spacing values from the design.
10. Use `get_screenshot` for more visual detail when necessary.
11. Use `get_variable_defs` when a token is not clear.
12. Implement the result as a Vue SFC with Tailwind classes and the existing component library.

Run only the checks that the user or the applicable `AGENTS.md` permits.
