---
name: figma-mcp
description: Use the Figma MCP server to translate a Figma design into a Vue page or component layout. Use when the user provides a Figma URL, asks to implement a design, or wants to draft a page layout from Figma.
argument-hint: <figma-url>
---

Refer to the standard: @standards/frontend/FIGMA_MCP_USAGE.md
Also read @packages/ui/CLAUDE.md for color token mapping and component conventions.

## Steps

1. **Parse the Figma URL** from `$ARGUMENTS` — extract the `fileKey` and `nodeId`. Convert `-` to `:` in the node ID.
2. **Read the standards above** for the available tools, adaptation rules, and color usage.
3. **Call `get_design_context`** with the extracted `nodeId` and `fileKey`, using `clientLanguages: "typescript,html,css"` and `clientFrameworks: "vue"`. This is always the first tool to call.
5. **Adapt the output to the Modrinth codebase:**
   - Map Figma color variables to `surface-*` / `text-*` tokens — never use Figma's aliased names directly.
   - Check `packages/ui/src/components/` for existing components that match elements in the design (buttons, cards, modals, inputs, etc.).
   - Check `packages/assets/styles/variables.scss` for tokens not exposed in Figma.
   - Match spacing values exactly from the design.
6. **Use `get_screenshot`** if you need a closer visual reference of specific nodes.
7. **Use `get_variable_defs`** to verify which design tokens are applied to ambiguous elements.
8. **Build the component** as a Vue SFC using Tailwind classes and the project's existing component library.
