# Changelog Style Guide

## The core rule

**Each bullet describes one user-visible change, written from the user's perspective, in plain language, as a single sentence.**

If you can't explain the change without referencing internal code, components, or refactors, it probably doesn't belong in the changelog.

## Voice and tense

- **Past tense, implied subject.** The section heading (`## Added`, `## Fixed`, `## Changed`) supplies the verb's mood - bullets read as a continuation of it.
  - Good: `Fixed a missing gap between the project filter tabs and the project list.`
  - Good: `Added support for Java 25.`
  - Avoid: `We fixed...`, `This fixes...`, `Fixes...` (present tense), `Will fix...`
- **No first person.** Don't say "we" or "our" inside a bullet. The exception is featured release callouts that link to a blog post (`We've overhauled the Content tab...`).
- **No second person except for direct user actions.** "You" is fine when describing what the user can now do (`Joining a server from the app downloads the required content and launches you directly into the server.`), but don't address the user gratuitously.

## Section/verb agreement

The opening verb must match the section it lives under. Don't put "Fixed X" bullets inside `## Added`.

| Section       | Typical opening verbs                                                           |
| ------------- | ------------------------------------------------------------------------------- |
| `## Added`    | Added, Introduced, New                                                          |
| `## Changed`  | Refreshed, Redesigned, Moved, Renamed, Updated, Consolidated, Improved, Rebuilt |
| `## Fixed`    | Fixed                                                                           |
| `## Security` | Fixed (security framing)                                                        |

In `## Added`, the leading "Added" is often dropped because it's redundant with the heading:

- `- Server stats inside server settings modal, in info card.`
- `- Confirmation modal for resubscribing to a server.`

In `## Fixed`, the leading "Fixed" is **kept** in most entries - it reads more clearly. Be consistent within a single entry.

## What to write about

Describe the **observable behavior**, not the implementation.

- Good: `Server CPU and memory graphs no longer freeze on the last value after a hard crash or out-of-memory kill.`
- Bad: `Refactored the metrics polling hook to clear stale state on socket disconnect.`

- Good: `Historical log files are now fetched in the background when opening the Logs page, so switching between them is instant.`
- Bad: `Moved log file fetching into a background worker.`

If a refactor has no user-visible effect, **don't list it**. Internal cleanup, dependency bumps, and code moves don't belong in the changelog unless they produce a noticeable difference (perf, reliability, consistency).

## Specificity

Be specific enough that a user reading the changelog can recognize the thing you're talking about.

- Vague: `Fixed a bug on the project page.`
- Better: `Fixed project versions table overflowing outside of table. Version tags will now truncate.`

- Vague: `Improved the UI.`
- Better: `Refreshed the server cards UI for consistency.`

Name the page, tab, modal, or feature you're talking about. "The Content tab", "the server panel header", "the Worlds tab", "the project page" - these give the reader a concrete anchor.

## Length

- **One sentence per bullet.** If you need two sentences, you probably have two bullets, or one bullet plus a sub-bullet.
- Aim for under ~25 words. Long bullets are usually a sign that the change is being over-explained or is actually multiple changes.
- Sub-bullets (indented with a tab) are allowed when one change has several facets - see the `## Added` section in the v0.12.0 app release for a good example.

## Punctuation

- **End every bullet with a period.** This is inconsistent in the historical file, but periods are the more common pattern and the one to follow going forward.
- Use sentence case, not Title Case.
- Use straight quotes, not curly quotes (`"foo"` not `"foo"`).
- Use proper code formatting for filenames, flags, and literal strings: `` `.log` ``, `` `Restart` ``.

## Naming things

- Use the public, branded name: **Modrinth App**, **Modrinth Hosting**, **Modrinth** - not "the app", "servers", "Modrinth Servers" (deprecated). Capitalize product names.
- Refer to UI surfaces by the label the user sees: **Content tab**, **Worlds tab**, **Files tab**, **Logs page**, **server panel**, **project page**, **Discover page**.
- Capitalize tab and page names when referring to them by name (`the Content tab`), but not when used generically (`browse content`).

## Don't

- **Don't blame.** Avoid "fixed a regression introduced in v0.12.0" - just describe the fix.
- **Don't reference PRs, issues, or commits.** The changelog is for users, not contributors - the exception is notable third-party contributions, where you should credit the contributor by linking their GitHub profile (e.g. `Added support for Java 25. Thanks to [@username](https://github.com/username)!`). Sharing credit for community contributions is encouraged.
- **Don't reference internal team members or processes.** No "as requested by support", no "per the design review".
- **Don't apologize or editorialize.** Skip "unfortunately", "finally", "long-awaited", "we know this has been a pain point". State the change.
- **Don't use vague intensifiers.** "Significantly improved", "much better", "vastly faster" - quantify if you can, otherwise drop the adverb.
- **Don't list every sub-fix of a bigger change separately.** If you redesigned the server panel header, write one bullet about the redesign rather than six bullets about each moved element.
- **Don't use "issue with" / "issue where" as filler.** `Fixed an issue where buttons were misaligned` → `Fixed misaligned buttons.`

## Examples - rewriting weak bullets

| Weak                                                       | Better                                                                                |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `Fixed a bug.`                                             | `Fixed project icons becoming extremely bright on hover.`                             |
| `Various improvements to the server panel.`                | Split into specific bullets, or drop entirely.                                        |
| `Refactored the logs page to use a new component.`         | `Redesigned the Logs page to match the Modrinth Hosting server panel.`                |
| `Fixed an issue where the server address wasn't copyable.` | `Server address in the panel header can now be clicked to copy it to your clipboard.` |
| `Made some changes to the content tab.`                    | Either drop, or list each user-visible change as its own bullet.                      |
| `Fixed UX issues.`                                         | Name the specific UX issue.                                                           |

## Featured release bullets

When an entry has a linked blog post heading (e.g. `## [Introducing Server Projects](/news/article/...)`), the bullets underneath summarize the *highlights* in 1–4 lines, then link out. They don't need to be exhaustive - that's what the blog post is for.

## Quick checklist before committing a bullet

1. Would a non-developer user understand it?
2. Does it describe behavior, not implementation?
3. Is the verb in the right tense for its section?
4. Does it name the specific surface (tab/page/modal)?
5. Is it one sentence, ending in a period?
6. Is there a vague word ("issue", "bug", "various", "some") I can replace with something concrete?
