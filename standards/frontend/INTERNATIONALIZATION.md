- [Internationalization (i18n)](#internationalization-i18n)
	- [Translatable Strings](#translatable-strings)
	- [Message Definitions](#message-definitions)
	- [Rendering Messages](#rendering-messages)
	- [ICU Message Format](#icu-message-format)
	- [Writing Translation-Friendly Strings](#writing-translation-friendly-strings)
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

## Writing Translation-Friendly Strings

ICU gives you powerful tools (plurals, selects, nested expressions), but translators in other languages face constraints that English doesn't have:

- **Word order varies by language.** Don't assume `{action} {noun}` works everywhere — some languages need `{noun} {action}` or require prepositions between them.
- **Plurals aren't just "add an s".** Many languages change internal parts of a word or phrase for pluralization, not just the ending. A simple `{count} {itemType}` breaks if `itemType` is always singular.
- **Grammatical gender affects surrounding words.** Articles, adjectives, and verbs may change based on whether a noun is masculine or feminine. If a variable like `{contentType}` can be "shader" or "mod", translators may need to inflect surrounding text differently for each.

### Guidelines

1. **Use `select` for content types, not bare variables.** When a variable represents different content types (mod, shader, modpack, etc.), pass a key and use ICU `select` so translators can write type-specific forms:

```
// Bad — translators can't inflect around a pre-rendered noun
'Delete {count} {itemType}'

// Good — translators can write entirely different phrases per type
'Delete {count} {contentType, select, mod {{count, plural, one {mod} other {mods}}} shader {{count, plural, one {shader} other {shaders}}} other {items}}'
```

This lets translators write entirely different noun forms per branch, which many languages require.

2. **Prefer separate messages over complex ICU when branches diverge significantly.** If the singular and plural versions of a string are structurally different (not just a noun change), use two separate message IDs rather than one complex ICU expression.

3. **Don't concatenate translated strings.** Never build a sentence by joining multiple `formatMessage` calls — the word order may be wrong in other languages. Put the entire sentence in one message.

4. **Keep variables semantic.** Pass `contentType: 'mod'` (a key), not `contentType: 'Mod'` (a pre-rendered display string). Translators can then map each key to the correct form in their language.

5. **Test with long strings.** German and Finnish words can be 2-3x longer than English equivalents. Ensure UI layouts don't break with longer text.

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
