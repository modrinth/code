# Changelog Style Guide

## Core Rule

Each bullet describes one user-visible change. Write one plain-language sentence from the perspective of the user.

Do not add a change that you can explain only with internal code, component, or refactor details.

## Voice and Tense

- Use the past tense with an implicit subject. The section heading supplies the context for the bullet.
	- Correct: `Fixed a missing gap between the project filter tabs and the project list.`
	- Correct: `Added support for Java 25.`
	- Incorrect: `We fixed...`, `This fixes...`, `Fixes...`, or `Will fix...`.
- Do not use the first person. A featured release that links to a blog post is an exception.
- Use the second person only for a direct user action.

Example of a direct action: `Joining a server downloads the required content and opens the server.`

## Section and Verb Agreement

Make the first verb agree with its section. Do not put a `Fixed` bullet in `## Added`.

| Section       | Typical first words                                                        |
| ------------- | -------------------------------------------------------------------------- |
| `## Added`    | Added, Introduced, New                                                      |
| `## Changed`  | Refreshed, Redesigned, Moved, Renamed, Updated, Consolidated, Improved     |
| `## Fixed`    | Fixed                                                                      |
| `## Security` | Fixed, with a clear security context                                       |

You can omit `Added` in the `## Added` section because the heading supplies it:

- `Server statistics in an information card inside the server settings modal.`
- `Confirmation modal for server resubscription.`

Keep `Fixed` in most `## Fixed` bullets because it makes the text clear. Use one pattern in each entry.

## Content

Describe the result that the user can see. Do not describe the implementation.

- Correct: `Server CPU and memory graphs no longer freeze after a hard crash or out-of-memory termination.`
- Incorrect: `Refactored the metrics polling hook to clear stale state after a socket disconnection.`

- Correct: `Historical log files now load in the background, so selection between files is immediate.`
- Incorrect: `Moved log file fetching into a background worker.`

Do not list a refactor that has no user-visible result.

You can list an internal change when it gives a visible improvement in performance, reliability, or consistency.

## Specific Terms

Give sufficient detail for the user to identify the applicable item.

- Vague: `Fixed a bug on the project page.`
- Specific: `Fixed project version rows that extended past the table. Version tags now truncate.`

- Vague: `Improved the UI.`
- Specific: `Refreshed the server cards for visual consistency.`

Name the applicable page, tab, modal, or feature. Examples include the Content tab, server panel header, Worlds tab, and project page.

## Length

- Write one sentence in each bullet.
- Use a second bullet when the change needs a second sentence.
- Use fewer than 25 words when possible.
- Use tab-indented sub-bullets when one change has multiple related parts.

Refer to the `## Added` section in the v0.12.0 app release for a sub-bullet example.

## Punctuation

- End each bullet with a period.
- Use sentence case, not title case.
- Use straight quotation marks, not curly quotation marks: `"foo"`.
- Use code formatting for filenames, flags, and literal strings: `.log` and `Restart`.

Historical entries do not always use periods. Use periods in all new entries.

## Product and UI Names

- Use the public names `Modrinth App`, `Modrinth Hosting`, and `Modrinth`.
- Do not use deprecated names, such as `Modrinth Servers`.
- Use the labels that appear in the UI.
- Capitalize a tab or page name when you refer to its label.
- Use lowercase when you refer to a generic action, such as `browse content`.

Examples of UI labels include Content tab, Worlds tab, Files tab, Logs page, server panel, project page, and Discover page.

## Prohibited Content

- Do not assign blame. Describe the correction without the release that caused the problem.
- Do not refer to pull requests, issues, or commits.
- Do not refer to internal team members or processes.
- Do not apologize or add an opinion about the change.
- Do not use vague intensifiers. Give a measurement when possible, or remove the adverb.
- Do not list each small correction from one larger change.
- Do not use `issue with` or `issue where` as filler.

You can credit a notable community contribution with a link to the contributor's GitHub profile.

Example: `Added support for Java 25. Thanks to [@username](https://github.com/username)!`

Replace `Fixed an issue with misaligned buttons` with `Fixed misaligned buttons.`

## Weak-Bullet Rewrites

| Weak                                                       | Better                                                                                |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `Fixed a bug.`                                             | `Fixed excessive brightness on project icons during hover.`                           |
| `Various improvements to the server panel.`                | Divide it into specific bullets, or remove it.                                        |
| `Refactored the logs page to use a new component.`         | `Redesigned the Logs page to match the Modrinth Hosting server panel.`                |
| `Fixed an issue where the server address was not copyable.` | `The server address in the panel header now copies to the clipboard when selected.`  |
| `Made some changes to the Content tab.`                    | List each user-visible change, or remove the bullet.                                  |
| `Fixed UX issues.`                                         | Name the specific user-experience problem.                                            |

## Featured Release Bullets

A featured release has a linked blog-post heading, such as `## [Introducing Server Projects](/news/article/...)`.

Use one to four lines below the heading to summarize the primary changes. Then, link to the blog post.

The bullets do not need to contain all details. The blog post contains the complete information.

## Bullet Checklist

Before you commit a bullet, make sure that it meets these requirements:

1. A user who is not a developer can understand it.
2. It describes behavior, not implementation.
3. Its verb uses the correct tense for the section.
4. It identifies the applicable tab, page, modal, or feature.
5. It contains one sentence and ends with a period.
6. It replaces vague words with specific terms.
