# Popout Windows

Vue component DOM can move into a separate browser window while its JavaScript continues running in the main window. Global `document` and `window` still refer to the main window, so they may differ from the document displaying a component.

## DOM Ownership

For DOM queries, focus checks, viewport measurements, and event listeners, use the relevant anchor, panel, or event target's `ownerDocument` and its `defaultView`.

Teleport overlays into that document's body. Components without an existing trigger element, such as `NewModal` and `ImageViewerEditor`, use a hidden anchor to identify their owning document when opened.

## Event Listeners

Interactions that must work in both the main window and popouts must register listeners with the relevant document or window. Use the feature's shared listener context when available, and release listeners when the component unmounts or the window closes.

## Modals and Scroll Locks

Keep modal stacks and scroll locks separate per document. Otherwise, a modal in one window can suppress Escape handling or focus trapping in another, and closing a modal can leave the wrong document locked.

Window closure must release overlay state and listeners even when normal user closing is disabled. Appearance synchronization must preserve each document's own scroll lock.
