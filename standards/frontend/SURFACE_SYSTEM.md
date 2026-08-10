# Surface System

Use `surface-*` variables to show UI elevation and separation. The scale starts at the page base and ends at strong strokes.

## Layers

| Token       | Use                                                               |
| ----------- | ----------------------------------------------------------------- |
| `surface-1` | Use for the page background.                                      |
| `surface-2` | Use for raised surfaces, table rows, and standard card backgrounds. |
| `surface-3` | Use for header bands, inputs, dropdowns, and card hover states.    |
| `surface-4` | Use for standard strokes, outlines, and table outlines.            |
| `surface-5` | Use for strong strokes that need more separation.                  |

## Strokes

Use `surface-4` for standard outlines and dividers. Use it for table borders and row separators.

Use `surface-5` for modal frames, strong separators, and hover states above `surface-4`.

## Backgrounds

Use `surface-1` for page backgrounds. Use `surface-2` for standard raised content.

Use `surface-3` for header strips, inputs, and temporary elevation. A hover state is an example of temporary elevation.

Use `surface-4` only for controls or badges that must appear above adjacent content.

Do not use legacy aliased background variables in new UI. Use explicit `bg-surface-*` and `border-surface-*` utilities.

These utilities show the intended layer in the component.
