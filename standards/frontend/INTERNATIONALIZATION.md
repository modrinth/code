- [Internationalization](#internationalization)
	- [Translatable strings](#translatable-strings)
	- [Message definitions](#message-definitions)
	- [Render messages](#render-messages)
	- [ICU message format](#icu-message-format)
	- [Write strings for translation](#write-strings-for-translation)
	- [Rich-text messages](#rich-text-messages)
	- [Vue and ICU delimiter conflicts](#vue-and-icu-delimiter-conflicts)
	- [Imports](#imports)
	- [Reference examples](#reference-examples)

# Internationalization

Use the `@modrinth/ui` localization system for all user-visible strings in Vue single-file components (SFCs).

Do not put hard-coded English text in templates or scripts. Get all user-visible text from `formatMessage` or `<IntlFormatted>`.

## Translatable Strings

Translate these user-visible items:

- Inner text.
- `alt` and `placeholder` attributes.
- Button and dropdown-option labels.
- Notification and error messages.

Do not translate dynamic expressions, HTML tag names, CSS classes, internal identifiers, or log messages.

In `{{ user.name }}`, only the static text around the expression needs translation.

## Message Definitions

Use `defineMessage` or `defineMessages` from `@modrinth/ui` in `<script setup>`.

Give each message a unique `id`. Put the English text in `defaultMessage`:

```ts
const messages = defineMessages({
	welcomeTitle: { id: 'auth.welcome.title', defaultMessage: 'Welcome' },
	welcomeDescription: { id: 'auth.welcome.description', defaultMessage: 'You are now part of the community.' },
})
```

Use descriptive, stable message IDs, such as `error.generic.default.title`. Put related messages in one `defineMessages` object.

## Render Messages

Use `useVIntl()` from `@modrinth/ui` to format simple strings:

```ts
const { formatMessage } = useVIntl()
```

```vue
<button>{{ formatMessage(messages.welcomeTitle) }}</button>
{{ formatMessage(messages.greeting, { name: user.name }) }}
```

## ICU Message Format

Use ICU placeholders for dynamic values in `defaultMessage`:

- Variable: `'Hello, {name}!'`
- Number, date, or time: `'{price, number, ::currency/USD}'`
- Plural or selection: `'{count, plural, one {# message} other {# messages}}'`

## Write Strings for Translation

ICU supports plurals, selections, and nested expressions. Languages can have different grammar rules.

- Word order changes between languages. Do not assume that `{action} {noun}` operates in all languages.
- Plural forms can change a complete word or phrase. Do not only add an `s` to make a plural.
- Grammatical gender can change articles, adjectives, and verbs. Give translators a separate branch for each content type.

### Guidelines

1. Use `select` for content types. Do not use a bare variable for a content type.

Pass a content-type key. Then, use ICU `select` so translators can write a specific form for each type:

```text
// Incorrect. Translators cannot change the grammar around this rendered noun.
'Delete {count} {itemType}'

// Correct. Translators can write a different phrase for each type.
'Delete {count} {contentType, select, mod {{count, plural, one {mod} other {mods}}} shader {{count, plural, one {shader} other {shaders}}} other {items}}'
```

This structure lets translators write different noun forms in each branch.

2. Use separate messages when ICU branches have different sentence structures.

If singular and plural text have different structures, use two message IDs. Do not make one complex ICU expression.

3. Do not join translated strings.

Do not make a sentence from multiple `formatMessage` calls. Put the complete sentence in one message.

4. Use semantic variable values.

Pass `contentType: 'mod'` as a key. Do not pass `contentType: 'Mod'` as rendered text.

The translator can map each key to the correct form.

5. Test the UI with long strings.

Some translated words can be two or three times longer than the English words. Make sure that the layout remains correct.

## Rich-Text Messages

When a message contains links or markup, put named tags around the applicable text in `defaultMessage`:

```text
"When you create an account, you agree to the <terms-link>Terms</terms-link> and <privacy-link>Privacy Policy</privacy-link>."
```

Use named slots in `<IntlFormatted>` to render the tags:

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

Use this pattern for simple emphasis:

```text
'Welcome to <strong>Modrinth</strong>!'
```

```vue
<template #strong="{ children }">
	<strong><component :is="() => children" /></strong>
</template>
```

Use `normalizeChildren` from `@modrinth/ui` for complex child content:

```vue
<template #bold="{ children }">
	<strong><component :is="() => normalizeChildren(children)" /></strong>
</template>
```

## Vue and ICU Delimiter Conflicts

If an ICU placeholder ends immediately before `}}`, add a space. Use `} }` to prevent a Vue parser error.

## Imports

Get all internationalization utilities from `@modrinth/ui`:

- `defineMessage` and `defineMessages`: Define messages.
- `useVIntl`: Supplies `formatMessage`.
- `IntlFormatted`: Renders rich-text messages.
- `normalizeChildren`: Normalizes complex rich-text slot children.

## Reference Examples

- Variables and plurals: `apps/frontend/src/pages/frog.vue`.
- Rich text with link tags: `apps/frontend/src/error.vue`.
