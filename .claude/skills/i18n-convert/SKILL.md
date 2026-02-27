# i18n String Conversion

Convert hard-coded natural-language strings in Vue SFCs into the localization system using utilities from `@modrinth/ui`.

## Rules

### 1. Identify translatable strings

- Scan `<template>` for all user-visible strings: inner text, alt attributes, placeholders, button labels, etc.
- Check `<script>` too: dropdown option labels, notification messages, etc.
- Do NOT extract dynamic expressions (`{{ user.name }}`) or HTML tags — only static human-readable text.

### 2. Create message definitions

Import `defineMessage` or `defineMessages` from `@modrinth/ui` in `<script setup>`. Define messages with a unique `id` (descriptive prefix based on component path) and `defaultMessage` equal to the original English string:

```ts
const messages = defineMessages({
	welcomeTitle: { id: 'auth.welcome.title', defaultMessage: 'Welcome' },
	welcomeDescription: { id: 'auth.welcome.description', defaultMessage: "You're now part of the community…" },
})
```

### 3. Handle variables and ICU formats

- Dynamic parts become ICU placeholders: `"Hello, ${user.name}!"` → `defaultMessage: 'Hello, {name}!'`
- Numbers/dates/times use ICU options: `{price, number, ::currency/USD}`
- Plurals/selects use ICU: `'{count, plural, one {# message} other {# messages}}'`

### 4. Rich-text messages (links/markup)

Wrap link/markup ranges with tags in `defaultMessage`:

```
"By creating an account, you agree to our <terms-link>Terms</terms-link> and <privacy-link>Privacy Policy</privacy-link>."
```

Render with `<IntlFormatted>` from `@modrinth/ui` using named slots:

```vue
<IntlFormatted :message-id="messages.tosLabel">
	<template #terms-link="{ children }">
		<NuxtLink to="/terms">
			<component :is="() => children" />
		</NuxtLink>
	</template>
	<template #privacy-link="{ children }">
		<NuxtLink to="/privacy">
			<component :is="() => children" />
		</NuxtLink>
	</template>
</IntlFormatted>
```

For simple emphasis: `'Welcome to <strong>Modrinth</strong>!'` with a slot:

```vue
<template #strong="{ children }">
	<strong><component :is="() => children" /></strong>
</template>
```

For complex child handling, use `normalizeChildren` from `@modrinth/ui`:

```vue
<template #bold="{ children }">
	<strong><component :is="() => normalizeChildren(children)" /></strong>
</template>
```

### 5. Formatting in templates

Use `useVIntl()` from `@modrinth/ui`; prefer `formatMessage` for simple strings:

```ts
const { formatMessage } = useVIntl()
```

```vue
<button>{{ formatMessage(messages.welcomeTitle) }}</button>
{{ formatMessage(messages.greeting, { name: user.name }) }}
```

### 6. Naming conventions

Make `id`s descriptive and stable (e.g., `error.generic.default.title`). Group related messages with `defineMessages`.

### 7. Avoid Vue/ICU delimiter collisions

If an ICU placeholder ends right before `}}` in a Vue template, insert a space: `} }` to avoid parsing issues.

### 8. Imports

Ensure these are imported from `@modrinth/ui` as needed: `defineMessage`/`defineMessages`, `useVIntl`, `IntlFormatted`, `normalizeChildren`.

### 9. Preserve functionality

Do not change logic, layout, reactivity, or bindings — only refactor strings into i18n.

## Reference Examples

- Variables/plurals: `apps/frontend/src/pages/frog.vue`
- Rich-text link tags: `apps/frontend/src/pages/auth/welcome.vue` and `apps/frontend/src/error.vue`

When finished, there should be no hard-coded English strings left in the template — everything comes from `formatMessage` or `<IntlFormatted>`.
