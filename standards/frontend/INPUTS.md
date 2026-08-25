# Inputs

Use the input components in `packages/ui/src/components/base/inputs/`. Use them for
text, numbers, and dates. These components have the same sizes, focus style,
appearances, disabled state, and error state.

Do not make input styles with raw `<input>` or `<textarea>` elements. Use a raw control
only if a shared component does not have the necessary function.

`InputFrame` and `InputClearButton` are internal components. Do not import them directly
into application code.

## Select a component

| Component    | Use                                                                        |
| ------------ | -------------------------------------------------------------------------- |
| `Input`      | Single-line text, email, password, number, URL, or search entry            |
| `Textarea`   | Multi-line text entry                                                       |
| `DateInput`  | A native browser control for a date or a local date and time               |
| `DatePicker` | A styled calendar for ranges, multiple dates, time, or inline presentation |

Use a dedicated component for each other type of control. Do not use `Input` instead
of a select, checkbox, radio group, toggle, or file picker.

Import the public input components from `@modrinth/ui`. In `packages/ui`, import them
from `#ui/components/base/inputs`:

```ts
import { DateInput, DatePicker, Input, Textarea } from '@modrinth/ui'
```

## Labels and field descriptions

The input components do not include a label, description, or validation message. Give
each control an accessible name.

For a visible label, set the same value for the label `for` and the control `id`:

```vue
<label for="project-name">Project name</label>
<Input
	id="project-name"
	v-model="projectName"
	name="name"
	autocomplete="off"
/>
```

An input does not have a `label` prop. Do not use a placeholder as the only label. The
placeholder is not visible after the user enters a value.

If a visible label repeats other content, use an accessible name such as `aria-label`:

```vue
<Input
	v-model="query"
	:icon="SearchIcon"
	type="search"
	aria-label="Search projects"
	placeholder="Search projects..."
/>
```

Connect descriptions and validation messages to the control with `aria-describedby`.
The component sends additional attributes to the native control:

```vue
<label for="project-slug">Project URL</label>
<Input
	id="project-slug"
	v-model="slug"
	:error="Boolean(slugError)"
	aria-describedby="project-slug-help project-slug-error"
/>
<p id="project-slug-help">Use lowercase letters, numbers, and dashes.</p>
<p v-if="slugError" id="project-slug-error">
	{{ slugError }}
</p>
```

Use the localization system for all user-visible text. This requirement also applies
to accessible labels.

## Basic use

Bind the field value with `v-model`. For text types, the `Input` model is a string. For
`type="number"`, the model is a number or `undefined`:

```vue
<Input
	v-model="email"
	type="email"
	name="email"
	autocomplete="email"
	placeholder="name@example.com"
/>

<Input
	v-model="serverCount"
	type="number"
	:min="1"
	:max="100"
	:step="1"
	clamp
/>
```

An empty number input sets the model to `undefined`. The `clamp` prop keeps the number
between `min` and `max`. Do not set `clamp` if validation must show the original value.

Set the native input type that agrees with the value. Set `name` and `autocomplete` if
a form or autofill uses the field.

Set `inputmode` to select the correct virtual keyboard. Use it when the data format and
the native input type are different.

Use `Textarea` for multi-line content:

```vue
<Textarea
	id="project-summary"
	v-model="summary"
	name="summary"
	:maxlength="500"
	:rows="4"
	resize="vertical"
/>
```

`Textarea` has three rows by default. The default value of `resize` is `none`. The other
values are `vertical` and `both`.

Use `vertical` if the layout width must not change.

`Input`, `Textarea`, and `DateInput` have a `focus()` method. Use this method when a
workflow must move keyboard focus:

```ts
const nameInput = ref<InstanceType<typeof Input> | null>(null)

nameInput.value?.focus()
```

## Appearances

`Input`, `Textarea`, and `DateInput` have the same three appearances:

| Appearance    | Use                                                                 |
| ------------- | ------------------------------------------------------------------- |
| `surface`     | Standard fields on pages, cards, and modals. This is the default.  |
| `button`      | Compact fields in panels that use the button surface.              |
| `transparent` | Fields in a container that already shows the visible field border. |

```vue
<Input v-model="name" appearance="surface" />
<Input v-model="filter" appearance="button" size="small" />
<Input v-model="value" appearance="transparent" />
```

Use `transparent` only if the parent container shows the field border. A field in the
error state uses the standard red border and highlight for all appearances.

Set `appearance` to select the background and border. Do not use utility classes to
make a different background or border.

`DatePicker` has only the standard surface appearance.

## Sizes

`Input` and `DateInput` have four sizes. Select the size for the location. If a button
is adjacent to the input, use the related button size:

| Input size | Height | Button size | Use                                      |
| ---------- | ------ | ----------- | ---------------------------------------- |
| `small`    | 32 px  | `sm`        | Compact panels, toolbars, and popups     |
| `standard` | 36 px  | `md`        | Standard fields. This is the default.    |
| `medium`   | 40 px  | `lg`        | Prominent modal, card, and header fields |
| `large`    | 48 px  | `xl`        | Page-level search and primary filters    |

```vue
<div class="flex items-center gap-2">
	<Input v-model="query" size="medium" class="flex-1" />
	<Button size="lg" type="colored">Search</Button>
</div>
```

Use the same size for fields in a group. Do not use classes to change the height,
padding, border radius, font size, gap, or icon size.

Set `rows` or a layout class to change the `Textarea` height. `DatePicker` uses the
standard input height.

## Leading and trailing content

Set `icon` on `Input` to show a decorative leading icon:

```vue
<Input
	v-model="query"
	:icon="SearchIcon"
	type="search"
	placeholder="Search..."
/>
```

Use the `leading` slot for other decorative content. The component hides this content
from assistive software. Do not put an interactive control or important status in this
slot.

```vue
<Input v-model="domain">
	<template #leading>
		<LinkIcon />
	</template>
</Input>
```

Use the `trailing` slot for content that is part of the field. Give each interactive
control an accessible name. Do not add a control that has the same function as the
field:

```vue
<Input v-model="password" :type="showPassword ? 'text' : 'password'">
	<template #trailing>
		<IconButton
			type="quiet"
			:label="showPassword ? 'Hide password' : 'Show password'"
			@click="showPassword = !showPassword"
		>
			<EyeIcon aria-hidden="true" />
		</IconButton>
	</template>
</Input>
```

Use `trailing` in new code. The `right` slot exists only for compatibility with
existing code.

## Clearable inputs

Set `clearable` if users often clear an optional value. You can set it on `Input`,
`DateInput`, and `DatePicker`.

The clear button is visible only when the field has a value. It is not visible when the
field is disabled or readonly:

```vue
<Input
	v-model="query"
	type="search"
	clearable
	clear-label="Clear search"
	@clear="refreshResults"
/>
```

For `Input` and `DateInput`, set a localized `clear-label`. This label is the accessible
name of the clear button. For `DateInput`, also set a localized `picker-label` for the
calendar button.

After a clear action, a text `Input` or `DateInput` model is an empty string. A number
`Input` model is `undefined`. A single `DatePicker` model is `null`. A multiple or range
`DatePicker` model is an empty array.

The component sends a `clear` event after a clear action. Use this event only if the
application must do an additional action. Other code can use the model change.

Do not add a second clear button to the `trailing` slot when `clearable` is set.

## Disabled, readonly, and error states

Set `disabled` when the value is not available. A disabled control cannot receive focus
and does not submit a value.

For `Input`, `Textarea`, and `DateInput`, set `readonly` when the user must not change
the value. A readonly control can receive focus:

```vue
<Input v-model="generatedId" readonly />
<Input v-model="username" :disabled="accountLocked" />
```

Set `error` after validation finds an invalid value. `Input`, `Textarea`, and
`DateInput` show the shared error style. They also set `aria-invalid` on the native
control:

```vue
<Input
	id="website"
	v-model="website"
	type="url"
	:error="Boolean(websiteError)"
	aria-describedby="website-error"
/>
<p v-if="websiteError" id="website-error">{{ websiteError }}</p>
```

The `error` prop does not show a message and does not validate the value. Show a
specific message. Connect the message to the field. Do not use only color to show an
error state.

You can use native limits such as `required`, `min`, `max`, `step`, and `maxlength`.
These limits do not set the `error` prop. Use application validation to control the
visible error state.

`DatePicker` uses `readonly` for a different function. It prevents text entry, but it
does not prevent calendar selection. Set `disabled` to prevent all changes.

## Native date inputs

Use `DateInput` if the browser date control has the necessary function. The component
accepts `type="date"` and `type="datetime-local"`. The model contains the native date
string:

```vue
<label for="scheduled-date">Scheduled date</label>
<DateInput
	id="scheduled-date"
	v-model="scheduledDate"
	type="datetime-local"
	:min="minimumDate"
	clearable
	clear-label="Clear scheduled date"
	picker-label="Open scheduled date picker"
/>
```

Set `min`, `max`, and `step` for native limits. The calendar button gives focus to the
native input. It opens the native picker if the browser has `showPicker()`.

Use `DateInput` for a native date control. Use `DatePicker` for one or more of these
functions:

- A calendar with the Modrinth style.
- A date range or multiple dates.
- Time controls.
- More than one visible month.
- An inline calendar.

## Date picker

`DatePicker` uses a Flatpickr calendar. In single mode, its model is one value. In
`multiple` and `range` modes, its model is an array. The component sends user
selections as formatted strings:

```vue
<DatePicker
	v-model="releaseDate"
	placeholder="Select a release date..."
	:min-date="today"
	clearable
	close-on-select
/>

<DatePicker
	v-model="reportingRange"
	mode="range"
	:show-months="2"
	default-view-date="2026-01-01"
	placeholder="Select a reporting range..."
/>
```

The modes use these model types:

| Mode       | Model type                                   | Value after clear |
| ---------- | -------------------------------------------- | ----------------- |
| `single`   | `string`, `Date`, `null`, or `undefined`     | `null`            |
| `multiple` | An array of `string` or `Date` values        | `[]`              |
| `range`    | An array of `string` or `Date` start and end | `[]`              |

The stored format is `Y-m-d` by default. It is `Y-m-d H:i` when `enable-time` is set.
The visible format is `F j, Y` by default. It is `F j, Y at h:i K` when `enable-time`
is set.

Set `date-format` and `alt-format` to use other Flatpickr formats:

```vue
<DatePicker
	v-model="startsAt"
	enable-time
	time24hr
	date-format="Y-m-d H:i"
	alt-format="j M Y, H:i"
/>
```

Make sure that model strings agree with `date-format`. The `alt-format` prop changes
only the visible value.

`DatePicker` sets `readonly` to true by default. The user selects a date from the
calendar. Set `:readonly="false"` only when users can type a date.

Set `close-on-select` if a single selection must close the popup. Its default value is
`false`.

Use the calendar options as follows:

- Set `default-view-date` to select the month that opens when there is no value. This
  prop does not set the model.
- Set `view-date-alignment="right"` to put the initial date in the right calendar. Use
  this prop when more than one month is visible.
- Set `min-date` and `max-date` to limit the dates that a user can select.
- Set `show-today` to highlight the current day. This prop does not select the day.
- Set `calendar-only` to show an inline calendar without a visible input.
- Set `position` to control the popup position and its alignment with the input.

Set `preserve-day` when a selection must keep the same day number during month changes.
If the day does not exist, `DatePicker` uses the last day of that month. When the day
exists again, `DatePicker` restores the original day.

The component sends a `clamp` event when it uses a different day. Use this event if the
interface must explain the change:

```vue
<DatePicker
	v-model="renewalDate"
	preserve-day
	@clamp="showClampedDayMessage"
/>
```

The picker has `focus()`, `open()`, `close()`, and `clear()` methods. Use these methods
only when a workflow requires control from the parent component.

## Classes and native attributes

`Input`, `Textarea`, and `DateInput` send additional attributes to the native control.
Examples are `required`, `aria-label`, and `aria-describedby`. The outer frame receives
`class` and `style`.

Set `wrapper-class` for field width and parent layout. Set `input-class` only when the
native control needs a layout adjustment:

```vue
<Input v-model="name" wrapper-class="w-full" />
```

The `input-attrs` prop is for input wrapper components. Application components must
pass native attributes directly.

Use the shared styles for borders, backgrounds, focus rings, height, padding, text,
gaps, and icon sizes. Do not replace these styles with utility classes. Width and parent
layout classes are permitted.

`DatePicker` also has a `calendar-class` prop. Use it for layout or state on the
calendar. Do not use it to replace the standard calendar styles.
