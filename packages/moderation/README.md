# @modrinth/moderation

This package provides project review definitions, moderation messages, settings, keyboard shortcuts, quick replies, and localized publishing feedback.

## Structure

- `src/data/issues/` defines project review panels and issues.
- `src/data/issues/component-builders/` contains panel builders, control types, issue targets, locations, and priorities.
- `src/data/messages/checklist/messages/` contains the retained project review message templates.
- `src/data/message-library.ts` defines message templates available in the review composer.
- `src/data/quick-replies/` defines replies for reports, technical review, and attribution review.
- `src/data/nags/` maps backend project validation results to localized publishing feedback and settings destinations.
- `src/data/settings.ts` and `src/data/keybinds.ts` define moderation preferences and shortcuts.
- `src/handles/` resolves saved settings and dispatches keyboard shortcuts by scope.
- `src/types/` contains shared moderation and review state types.
- `vite-plugin.ts` loads the project's static Markdown message imports.

## Project Review

The website hosts the review interface at `/moderation/project-review`. Its components and providers live in `apps/frontend/src/components/project-review/` and `apps/frontend/src/providers/project-review/`.

### Panels and issues

Panels group controls for a project field or review area. Define panels with `panel()` and sections with `section()`, then populate them using `.content()`.

Controls include toggles, Markdown editors, text inputs, textareas, and selects. Each issue has a stable ID, title, category, and message. Issues can also specify priorities, actions, locations, and a suggested status.

Use `shown` callbacks and the review context to conditionally display panels, sections, and controls. The context exposes project data, selected issues and toggles, and input values.

Register panels in `src/data/issues/index.ts`. Item panels can be expanded for individual versions, gallery images, or disclosures through the component builders.

### Messages

Issue message callbacks combine Markdown templates and substitute input values. Shared project and policy variables use `%VARIABLE%` placeholders, expanded by the review message provider.

The message library supplies standalone templates to the composer. Quick reply templates remain in `src/data/messages/quick-replies/` and use Vite's `?raw` imports.

### State and shortcuts

The website review session stores per-user, per-project drafts independently of the moderation queue. Shared state values and suggested moderation statuses are defined in `src/types/review.ts`.

Review shortcuts use the `project-review`, `review-actions`, and `review-composer` scopes. Public project copy shortcuts, technical review shortcuts, and global shortcuts have their own scopes. Keep shortcut and setting IDs stable so saved moderator preferences continue to work.

## Publishing Feedback

The publishing checklist presents automated feedback returned by Labrinth's project validation endpoint. Labrinth decides which nags apply and their severity; this package owns localized copy and navigation metadata.

To add feedback, define localized messages in the appropriate `src/data/nags/` area file, register the backend kind in `src/data/nags/index.ts`, and assign its settings destination. Use `defineMessage` or `defineMessages` from `@modrinth/ui` for translatable text.
