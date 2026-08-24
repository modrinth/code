---
name: figma-mcp
description: Convert a Figma design into a Modrinth Vue page or component. Use when a request provides a Figma URL or asks to implement a Figma layout.
---

# Implement a Figma Design

Read the applicable `AGENTS.md` files before you edit code.

Read `packages/ui/AGENTS.md` in full.

1. Load the available Figma design-to-code instructions and follow the MCP tool guidance.
2. Call `get_design_context` first with `clientLanguages: "typescript,html,css"` and `clientFrameworks: "vue"`.
3. Treat the result as reference code and adapt it to the Modrinth codebase.
4. Map Figma color variables to the applicable `surface-*` and `text-*` tokens. Do not use aliased Figma names directly.
5. Reuse applicable components from `packages/ui/src/components/` before creating new ones. Also refer to `standards/frontend/COMPONENT_STRUCTURE.md`
6. Read `packages/assets/styles/variables.scss` when Figma does not supply a required token.
7. Use exact spacing values from the design.
8. Implement the result as a Vue SFC with Tailwind classes and the existing component library.

Run only the checks that the user or the applicable `AGENTS.md` permits.
