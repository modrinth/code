---
name: i18n-pass
description: Perform an i18n localization pass on changed files or a pull request, converting hard-coded English strings to the @modrinth/ui i18n system. Use when internationalizing a set of changes, reviewing a PR for untranslated strings, or converting a specific component.
argument-hint: [file-path-or-pr-number]
---

Refer to the standard: @standards/frontend/INTERNATIONALIZATION.md

## Steps

1. **Identify the scope of changes:**
   - If `$ARGUMENTS` is a PR number, run `gh pr diff $ARGUMENTS` to get the changed files.
   - If `$ARGUMENTS` is a file path, use that directly.
   - If no argument, check `git diff` for uncommitted changes.
2. **Read the standard above** for the message definition pattern, ICU format rules, and `IntlFormatted` usage.
3. **Filter to Vue SFCs** — only `.vue` files need i18n passes. Skip non-component files.
4. **For each file, scan for hard-coded strings:**
   - `<template>`: inner text, `alt`, `placeholder`, `aria-label`, button labels, tooltip text.
   - `<script>`: string literals passed to user-visible UI (notification messages, dropdown labels, error messages).
   - Skip: dynamic expressions, HTML tag names, CSS classes, internal identifiers, log messages.
5. **Define messages** with `defineMessages` — use descriptive, stable `id`s based on the component's domain (e.g. `project.settings.title`).
6. **Replace strings in templates** with `formatMessage()` calls, or `<IntlFormatted>` for strings containing links or markup.
7. **Handle ICU edge cases** — add a space before `}}` if an ICU placeholder ends at a Vue template delimiter boundary.
8. **Verify** no hard-coded English strings remain in the changed templates. Do not alter logic, layout, or reactivity.
