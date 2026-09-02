---
name: review-changelog
description: Review the latest packages/blog/changelog.ts entry against the Modrinth changelog standard. Use before a pull request or when asked to review or lint a changelog entry.
---

# Review a Changelog Entry

Read [the changelog standard](../../../standards/maintaining/CHANGELOG.md) in full before the review.

1. Open `packages/blog/changelog.ts`.
2. Find the first entry in the `VERSIONS` array.
3. If the request names `web`, `hosting`, or `app`, review the latest entry for that product.
4. Otherwise, review the latest entry and all adjacent entries with the same date.

Check the entry structure:

- `date` contains a valid ISO 8601 timestamp.
- `product` is `web`, `hosting`, or `app`.
- An `app` entry has a `version` value.
- A `web` or `hosting` entry does not have a `version` value.
- Standard headings are `## Added`, `## Changed`, `## Fixed`, and `## Security`.
- A featured release can use a linked heading.
- Flag the legacy `## Improvements` heading.

Check each bullet:

- The voice and tense agree with the section.
- The first verb agrees with the section.
- The bullet describes user-visible behavior, not implementation.
- The bullet identifies the applicable page, tab, modal, or feature.
- The bullet contains one sentence, uses sentence case, and ends with a period.
- Product and UI names use the public labels.
- The bullet does not contain filler, vague intensifiers, apologies, or internal references.
- The bullet is not a duplicate detail of a larger listed change.

Group findings by entry. For each finding, show the original bullet and a proposed replacement.

If the entry has no findings, state this result. Do not edit the changelog unless the user asks you to apply fixes.

When the user asks for fixes, preserve tab indentation and template-literal formatting.
