# Buttons

Use the button components in `packages/ui/src/components/base/buttons/` for actions,
navigation, file selection, and button-owned menus. They share the same sizing,
interaction states, focus treatment, and visual variants.

Do not recreate button styling with raw `<button>` or `<a>` elements. A raw element is
appropriate only when an interaction has semantics or behaviour that the shared
components cannot represent.

`ButtonFrame` is an internal styling primitive. Use one of the public components instead
of importing it directly.

## Choosing a component

| Component                  | Use                                                                                 |
| -------------------------- | ----------------------------------------------------------------------------------- |
| `Button`                   | An action, including form submission                                                 |
| `ButtonLink`               | Navigation to an internal route or external URL                                      |
| `IconButton`               | An icon-only action                                                                  |
| `FileButton`               | Opening a file picker, with optional drag-and-drop support                           |
| `ButtonGroup`              | Visually joining related buttons                                                     |
| `SplitButton`              | A primary action with an attached overflow menu                                      |
| `TeleportOverflowMenu`     | An icon button that opens a menu of actions and links                                |
| `TeleportPopoutMenu`       | A button or icon button that opens an arbitrary panel                                |

Use the component that matches the interaction's semantics. Do not use a click handler
on `ButtonLink` for an action, and do not perform navigation from a `Button`.

The new `Button` currently shares its name with the legacy root export. Until the legacy
component is removed, import it directly:

```ts
import Button from '@modrinth/ui/src/components/base/buttons/Button.vue'
import { ButtonLink, IconButton } from '@modrinth/ui'
```

## Basic usage

Use `Button` for actions. Its native type defaults to `button`, so set
`native-type="submit"` explicitly when it submits a form.

```vue
<Button type="colored" @click="save">
	<SaveIcon aria-hidden="true" />
	Save changes
</Button>

<Button native-type="submit" :loading="saveMutation.isPending.value">
	Save
</Button>
```

Use `ButtonLink` for navigation. Pass exactly one of `to` or `href`:

```vue
<ButtonLink to="/settings">Settings</ButtonLink>

<ButtonLink href="https://example.com" target="_blank" type="outlined">
	Documentation
	<ExternalIcon aria-hidden="true" />
</ButtonLink>
```

`to` creates a Vue Router link. `href` creates a standard anchor. External links opened
in a new tab receive `rel="noopener noreferrer"` unless a different `rel` is provided.

Use `IconButton` when the visible icon is the entire button content. The `label` prop is
required and supplies the button's accessible name:

```vue
<IconButton label="Open settings" type="quiet" @click="openSettings">
	<SettingsIcon aria-hidden="true" />
</IconButton>
```

Do not replace `label` with a tooltip or rely on the icon to communicate the action.
User-visible labels, including `IconButton` labels and menu option labels, must use the
localization system.

## Visual types

All button components use the same four visual types:

| Type       | Use                                                                 |
| ---------- | ------------------------------------------------------------------- |
| `base`     | Default neutral actions                                             |
| `colored`  | The highest-emphasis action in a section                            |
| `outlined` | Secondary actions that need a visible boundary                      |
| `quiet`    | Low-emphasis actions in toolbars, cards, and other compact contexts |

`base` is the default. Use `colored` deliberately: a group should normally have only one
highest-emphasis action.

The available colors are `brand`, `red`, `orange`, `green`, `blue`, `purple`, and
`medal_promotion`. Color is supported by `colored` and `quiet`; `base` and `outlined`
use their standard neutral treatment.

```vue
<Button type="colored" color="brand">Create project</Button>
<Button type="colored" color="red">Delete project</Button>
<Button type="outlined">Cancel</Button>
<Button type="quiet" color="red">Remove</Button>
```

Use red only for destructive or dangerous actions. Do not use color as the only way to
communicate an action's meaning.

## Sizes

| Size | Height | Typical use                                      |
| ---- | ------ | ------------------------------------------------ |
| `sm` | 32px   | Dense toolbars and constrained secondary actions |
| `md` | 36px   | Default application controls                     |
| `lg` | 40px   | Prominent actions and roomier layouts            |
| `xl` | 48px   | Large, high-emphasis actions                     |

`md` is the default. `IconButton` uses the same value for its width and height. Buttons
that appear together must use the same size.

Prefer the size prop over overriding height, padding, border radius, font weight, gap, or
icon dimensions with utility classes. Width and layout utilities such as `w-full` are
fine when required by the surrounding layout.

```vue
<Button size="sm">Compact action</Button>
<Button>Default action</Button>
<Button size="lg">Prominent action</Button>
<Button size="xl">Large action</Button>
```

## States

Use `disabled` when an action is unavailable. Use `loading` while a `Button` or
`IconButton` action is in progress:

```vue
<Button
	type="colored"
	:loading="createMutation.isPending.value"
	@click="createMutation.mutate()"
>
	Create
</Button>
```

`loading` disables the underlying button and sets `aria-busy`. Keep the label stable so
the action remains understandable while it is running.

Disabled links are made non-navigable and removed from the tab order by `ButtonLink`.
Do not reimplement disabled-link click handling in consumers.

## Icons and content

Place icons directly inside the component so the shared styles can size and space them:

```vue
<Button>
	<DownloadIcon aria-hidden="true" />
	Download
</Button>

<Button>
	Continue
	<RightArrowIcon aria-hidden="true" />
</Button>
```

Icons next to a visible label are decorative and should use `aria-hidden="true"`. Use
`IconButton` instead of an empty text button for icon-only actions.

Button labels should describe the action directly. Avoid vague labels such as "Yes",
"OK", or "Submit" when a specific label such as "Delete version" or "Save settings"
would be clearer.

## File buttons

`FileButton` owns the file input and emits validated files:

```vue
<FileButton
	prompt="Select images"
	accept="image/*"
	:max-size="MAX_IMAGE_SIZE"
	multiple
	@change="handleFiles"
>
	<UploadIcon aria-hidden="true" />
</FileButton>
```

The `change` event receives a `File[]`. `allow-drop` defaults to `true`; set it to
`false` when the surrounding interface should not accept dropped files.

Use `FileButton` for a compact file-selection action. Use a dedicated drop area when
drag-and-drop is the primary interaction or the interface needs previews, progress, or
more detailed instructions.

## Button groups and split buttons

Use `ButtonGroup` only for closely related controls. Pass a localized `label` when the
relationship is not already clear from surrounding accessible content:

```vue
<ButtonGroup label="Pagination">
	<Button type="outlined">Previous</Button>
	<Button type="outlined">Next</Button>
</ButtonGroup>
```

`ButtonGroup` joins the buttons visually, so its direct children must be components from
this button system.

Use `SplitButton` when one action is primary and closely related alternatives belong in
an attached menu:

```vue
<script setup lang="ts">
import { PlayIcon, SettingsIcon, TrashIcon } from '@modrinth/assets'
import type { OverflowMenuOption } from '@modrinth/ui/src/components/base/buttons/types'

const options: OverflowMenuOption[] = [
	{
		id: 'settings',
		label: 'Server settings',
		icon: SettingsIcon,
		action: openSettings,
	},
	{ type: 'divider' },
	{
		id: 'delete',
		label: 'Delete server',
		icon: TrashIcon,
		tone: 'red',
		action: deleteServer,
	},
]
</script>

<template>
	<SplitButton
		menu-label="More server actions"
		group-label="Server actions"
		type="colored"
		:options="options"
		@click="startServer"
	>
		<PlayIcon aria-hidden="true" />
		Start server
	</SplitButton>
</template>
```

Use `primary-disabled` and `menu-disabled` when only one side is unavailable. Use
`disabled` when the entire split button is unavailable.

## Overflow menus

`TeleportOverflowMenu` is for a menu of discrete actions and navigation targets. Define
its options as `OverflowMenuOption[]`:

```ts
const options: OverflowMenuOption[] = [
	{
		id: 'download',
		label: 'Download',
		icon: DownloadIcon,
		action: download,
	},
	{
		id: 'project-page',
		label: 'Open project page',
		icon: ExternalIcon,
		type: 'link',
		to: projectRoute,
	},
	{ type: 'divider' },
	{
		id: 'delete',
		label: 'Delete',
		icon: TrashIcon,
		tone: 'red',
		action: remove,
	},
]
```

Each non-divider option needs a stable, unique `id` and a localized `label`.

- Use an action option for behaviour and a link option for navigation.
- Use `shown: false` to remove an option conditionally.
- Use `disabled` with a `tooltip` when the user needs to understand why an option is
  unavailable.
- Use `remainOpen` only when selecting the option should not dismiss the menu.
- Use `tone: 'red'` for destructive options.
- Use dividers sparingly to separate meaningful groups.

The component implements menu keyboard navigation, focus management, typeahead, and
Escape handling. Consumers should not add competing keyboard or focus behaviour.

## Popout menus

Use `TeleportPopoutMenu` for arbitrary interactive content that belongs to a button,
such as a compact settings panel. Use `TeleportOverflowMenu` instead when the content is
only a list of actions or links.

```vue
<TeleportPopoutMenu label="Configure versions" placement="bottom-start">
	<template #trigger>
		<SettingsIcon aria-hidden="true" />
		Configure
	</template>

	<template #panel="{ close }">
		<div class="flex w-72 flex-col gap-4">
			<StyledInput v-model="versionName" label="Version name" />
			<Button type="colored" @click="close()">Apply</Button>
		</div>
	</template>
</TeleportPopoutMenu>
```

Set `icon-only` when the trigger has no visible text; `label` then becomes the trigger's
accessible name. The panel role defaults to `dialog`; use `panel-role="region"` only
when the content is non-modal supplementary content.

The panel slot receives `close`, and the component manages initial focus, Escape, and
focus restoration.

## Legacy components

Do not introduce new uses of the legacy `packages/ui/src/components/base/Button.vue` or
`ButtonStyled.vue` APIs. When migrating:

| Legacy pattern                  | Replacement                                  |
| ------------------------------- | -------------------------------------------- |
| `link` / `external`             | `ButtonLink` with `to`, `href`, and `target` |
| `action`                        | `@click` on `Button`                         |
| `icon-only`                     | `IconButton` with `label`                    |
| `large`                         | `size="lg"` or `size="xl"`                   |
| `outline`                       | `type="outlined"`                            |
| `transparent`                   | `type="quiet"`                               |
| `ButtonStyled` wrapping an element | The matching direct button component      |

Preserve the original element's semantics while migrating. A visual match is not enough
if a link becomes a button or an icon-only action loses its accessible name.
