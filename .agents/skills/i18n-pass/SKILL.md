---
name: i18n-pass
description: Convert hard-coded English text in changed Vue components to the @modrinth/ui localization system. Use for an i18n pass, untranslated-string review, pull request, or component migration.
---

# Do an Internationalization Pass

Read the applicable `AGENTS.md` files before you edit code.

Read [the internationalization standard](../../../standards/frontend/INTERNATIONALIZATION.md) in full.

1. Identify the scope from the request.
2. For a pull request, use `gh pr diff <number>` to identify changed files.
3. For a file path, inspect that file.
4. When the request gives no scope, inspect the current uncommitted diff.
5. Limit the pass to changed `.vue` files.
6. Find user-visible text in templates and scripts.

Check inner text, `alt`, `placeholder`, `aria-label`, buttons, tooltips, notifications, dropdown labels, and error messages.

Do not change dynamic expressions, HTML tag names, CSS classes, internal identifiers, or log messages.

1. Define stable message IDs with `defineMessage` or `defineMessages`.
2. Replace simple text with `formatMessage()` calls.
3. Use `<IntlFormatted>` for text that contains links or markup.
4. Use ICU selections and plurals when grammar depends on a value.
5. Add a space before `}}` when an ICU placeholder ends at the Vue delimiter.
6. Do not change component logic, layout, or reactivity.
7. Do not edit localization JSON files. The user maintains those files.
8. Check the changed templates again for hard-coded English text.

Run only the checks that the user or the applicable `AGENTS.md` permits.
