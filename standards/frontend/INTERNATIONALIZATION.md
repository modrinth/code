- [Internationalization (i18n)](#internationalization-i18n)
	- [Translatable Strings](#translatable-strings)
	- [Message Definitions](#message-definitions)
	- [Rendering Messages](#rendering-messages)
	- [ICU Message Format](#icu-message-format)
	- [Rich-Text Messages](#rich-text-messages)
	- [Vue/ICU Delimiter Collisions](#vueicu-delimiter-collisions)
	- [Imports](#imports)
	- [Reference Examples](#reference-examples)

# Internationalization (i18n)

All user-visible strings in Vue SFCs must use the localization system from `@modrinth/ui`. No hard-coded English strings should appear in templates or script — everything comes from `formatMessage` or `<IntlFormatted>`.

## Translatable Strings

User-visible strings include: inner text, `alt` attributes, `placeholder` attributes, button labels, dropdown option labels, notification messages, etc.

Dynamic expressions (`{{ user.name }}`) and HTML tags are not translatable strings — only static human-readable text.

## Message Definitions

Messages are defined with `defineMessage` or `defineMessages` from `@modrinth/ui` in `<script setup>`. Each message has a unique `id` and a `defaultMessage` containing the English string:

```ts
const messages = defineMessages({
	welcomeTitle: { id: 'auth.welcome.title', defaultMessage: 'Welcome' },
	welcomeDescription: { id: 'auth.welcome.description', defaultMessage: "You're now part of the community…" },
})
```

Message `id`s should be descriptive and stable (e.g. `error.generic.default.title`). Group related messages together with `defineMessages`.

## Rendering Messages

Use `useVIntl()` from `@modrinth/ui` for simple string formatting:

```ts
const { formatMessage } = useVIntl()
```

```vue
<button>{{ formatMessage(messages.welcomeTitle) }}</button>
{{ formatMessage(messages.greeting, { name: user.name }) }}
```

## ICU Message Format

Dynamic values use ICU placeholders in `defaultMessage`:

- **Variables:** `'Hello, {name}!'`
- **Numbers/dates/times:** `'{price, number, ::currency/USD}'`
- **Plurals/selects:** `'{count, plural, one {# message} other {# messages}}'`

## Rich-Text Messages

When a message contains links or markup, wrap the relevant ranges with named tags in `defaultMessage`:

```
"By creating an account, you agree to our <terms-link>Terms</terms-link> and <privacy-link>Privacy Policy</privacy-link>."
```

Render with the `<IntlFormatted>` component using named slots:

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

For simple emphasis (`'Welcome to <strong>Modrinth</strong>!'`):

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

## Vue/ICU Delimiter Collisions

If an ICU placeholder ends right before `}}` in a Vue template, insert a space (`} }`) to avoid parsing issues.

## Imports

All i18n utilities come from `@modrinth/ui`:

- `defineMessage` / `defineMessages` — message definitions
- `useVIntl` — composable providing `formatMessage`
- `IntlFormatted` — component for rich-text messages
- `normalizeChildren` — helper for complex rich-text slot children

## Reference Examples

- Variables and plurals: `apps/frontend/src/pages/frog.vue`
- Rich-text with link tags: `apps/frontend/src/pages/auth/welcome.vue` and `apps/frontend/src/error.vue`
