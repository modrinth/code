---
name: review-changelog
description: Review the latest changelog entry in packages/blog/changelog.ts against the project's changelog style guide and flag bullets that need rewriting. Use when checking a freshly added changelog entry before opening a PR, or when the user asks to review/lint the latest changelog.
argument-hint: [product?]
---

Refer to the standard: @standards/maintaining/CHANGELOG.md

## Steps

1. **Locate the latest entry:**
   - Open `packages/blog/changelog.ts`.
   - The latest entries are at the top of the `VERSIONS` array.
   - If `$ARGUMENTS` specifies a product (`web`, `hosting`, `app`), review the most recent entry for that product. Otherwise, review the most recent entry overall, plus any sibling entries sharing the same `date` (coordinated releases ship together).

2. **Read the standard above** in full before reviewing. The bullet rules, section/verb agreement, and "Don't" list are the source of truth.

3. **Check the entry shell:**
   - `date` is a valid ISO 8601 timestamp.
   - `product` is one of `web`, `hosting`, `app`.
   - `version` is present for `app` entries and omitted for `web`/`hosting`.
   - Section headings use `## Added`, `## Changed`, `## Fixed`, `## Security` (or a featured-release linked heading). Flag legacy `## Improvements`.

4. **Review each bullet** against the standard. For each bullet, check:
   - Voice/tense matches the section heading.
   - Opening verb agrees with its section.
   - Describes observable behavior, not implementation.
   - Specific enough to identify the surface (names the tab/page/modal).
   - One sentence, ends with a period, sentence case.
   - Uses branded names (Modrinth App, Modrinth Hosting) correctly.
   - No filler ("issue with", "issue where", "various", "some"), no vague intensifiers, no apologies, no PR/commit references (unless crediting a third-party contributor with a linked GitHub profile).
   - Not a duplicate sub-fix of a bigger change already listed.

5. **Report findings** as a short list grouped by entry. For each problem bullet, show the original line and a suggested rewrite. If the entry is clean, say so explicitly. Do not edit the file unless the user asks - this skill is a review pass, not a rewrite pass.

6. **If the user then asks to apply fixes**, edit `packages/blog/changelog.ts` directly using the suggested rewrites. Preserve tab indentation and template literal formatting.
