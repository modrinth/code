---
applyTo: '**/*.vue'
---

You are given a Nuxt/Vue single-file component (.vue). Your task is to convert every hard-coded natural-language string in the <template> into our localization system using vue-i18n with utilities from `@modrinth/ui`.

Please follow these rules precisely:

1. Identify translatable strings

- Scan the <template> for all user-visible strings (inner text, alt attributes, placeholders, button labels, etc.). Do not extract dynamic expressions (like {{ user.name }}) or HTML tags. Only extract static human-readable text.
- There may be strings within the <script> block, e.g dropdown option labels, notifications etc.

2. Create message definitions

- In the <script setup> block, import `defineMessage` or `defineMessages` from `@modrinth/ui`.
- For each extracted string, define a message with a unique `id` (use a descriptive prefix based on the component path, e.g. `auth.welcome.long-title`) and a `defaultMessage` equal to the original English string.
  Example:
  const messages = defineMessages({
  welcomeTitle: { id: 'auth.welcome.title', defaultMessage: 'Welcome' },
  welcomeDescription: { id: 'auth.welcome.description', defaultMessage: 'You're now part of the community…' },
  })

3. Handle variables and ICU formats

- Replace dynamic parts with ICU placeholders: "Hello, ${user.name}!" → `{name}` and defaultMessage: 'Hello, {name}!'
- For numbers/dates/times, use ICU options (e.g., currency): `{price, number, ::currency/USD}`
- For plurals/selects, use ICU: `'{count, plural, one {# message} other {# messages}}'`

4. Rich-text messages (links/markup)

- In `defaultMessage`, wrap link/markup ranges with tags, e.g.:
  "By creating an account, you agree to our <terms-link>Terms</terms-link> and <privacy-link>Privacy Policy</privacy-link>."
- Render rich-text messages with `<IntlFormatted>` from `@modrinth/ui` using named slots:
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
- For simple emphasis: `'Welcome to <strong>Modrinth</strong>!'` with a slot:
  <template #strong="{ children }">
    <strong><component :is="() => children" /></strong>
  </template>
- For more complex child handling, use `normalizeChildren` from `@modrinth/ui`:
  <template #bold="{ children }">
    <strong><component :is="() => normalizeChildren(children)" /></strong>
  </template>

5. Formatting in templates

- Import and use `useVIntl()` from `@modrinth/ui`; prefer `formatMessage` for simple strings:
  `const { formatMessage } = useVIntl()`
  `<button>{{ formatMessage(messages.welcomeTitle) }}</button>`
- Pass variables as a second argument:
  `{{ formatMessage(messages.greeting, { name: user.name }) }}`

6. Naming conventions and id stability

- Make `id`s descriptive and stable (e.g., `error.generic.default.title`). Group related messages with `defineMessages`.

7. Avoid Vue/ICU delimiter collisions

- If an ICU placeholder would end right before `}}` in a Vue template, insert a space so it becomes `} }` to avoid parsing issues.

8. Update imports and remove literals

- Ensure imports from `@modrinth/ui` are present: `defineMessage`/`defineMessages`, `useVIntl`, `IntlFormatted`, and optionally `normalizeChildren`.
- Replace all hard-coded strings with `formatMessage(...)` or `<IntlFormatted>` and remove the literals.

9. Preserve functionality

- Do not change logic, layout, reactivity, or bindings—only refactor strings into i18n.

Use existing patterns from our codebase:

- Variables/plurals: see `apps/frontend/src/pages/frog.vue`
- Rich-text link tags: see `apps/frontend/src/pages/auth/welcome.vue` and `apps/frontend/src/error.vue`

When you finish, there should be no hard-coded English strings left in the template—everything comes from `formatMessage` or `<IntlFormatted>`.
