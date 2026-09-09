/**
 * Module-level owner of the moderation review Picture-in-Picture window.
 *
 * The window is deliberately NOT tied to a component lifecycle: `[type]/[project].vue`
 * remounts on every queue navigation (it is keyed by project id), so a component-owned
 * window would be torn down and re-requested — losing the user-gesture grant and flashing
 * a fresh window — every time the moderator advances the queue. Keeping the `Window` here
 * lets the same PiP window stay alive while its teleported contents re-render against the
 * new project.
 *
 * Document PiP is Chromium-only; `pipSupported` is false elsewhere and `ensurePipWindow()`
 * resolves to null.
 */

export const pipSupported = typeof window !== 'undefined' && 'documentPictureInPicture' in window

let pipWindow: Window | null = null
let styleObserver: MutationObserver | null = null
/** Called when the window is closed by the user / the OS (not by our own code). */
let onExternalClose: (() => void) | null = null

export interface EnsurePipWindowOptions {
	width?: number
	height?: number
	onClose?: () => void
}

export function isPipWindowOpen(): boolean {
	return !!pipWindow && !pipWindow.closed
}

function mirrorSheet(doc: Document, sheet: CSSStyleSheet) {
	try {
		const cssText = Array.from(sheet.cssRules)
			.map((rule) => rule.cssText)
			.join('\n')
		const style = doc.createElement('style')
		style.textContent = cssText
		doc.head.appendChild(style)
	} catch {
		// Cross-origin sheet: re-link it instead of reading rules.
		if (sheet.href) {
			const link = doc.createElement('link')
			link.rel = 'stylesheet'
			link.href = sheet.href
			if (sheet.media?.mediaText) link.media = sheet.media.mediaText
			doc.head.appendChild(link)
		}
	}
}

function copyStyles(win: Window) {
	for (const sheet of Array.from(document.styleSheets)) {
		mirrorSheet(win.document, sheet as CSSStyleSheet)
	}
}

function syncRootAttributes(win: Window) {
	const srcHtml = document.documentElement
	const dstHtml = win.document.documentElement
	dstHtml.className = srcHtml.className
	for (const attr of Array.from(srcHtml.attributes)) {
		if (attr.name.startsWith('data-') || attr.name === 'style' || attr.name === 'lang') {
			dstHtml.setAttribute(attr.name, attr.value)
		}
	}
	win.document.body.className = document.body.className
}

function watchNewStyles(win: Window) {
	styleObserver?.disconnect()
	styleObserver = new MutationObserver((mutations) => {
		for (const mutation of mutations) {
			for (const node of Array.from(mutation.addedNodes)) {
				if (node instanceof HTMLStyleElement) {
					const clone = win.document.createElement('style')
					clone.textContent = node.textContent
					win.document.head.appendChild(clone)
				} else if (node instanceof HTMLLinkElement && node.rel === 'stylesheet') {
					const clone = win.document.createElement('link')
					clone.rel = 'stylesheet'
					clone.href = node.href
					win.document.head.appendChild(clone)
				}
			}
		}
	})
	styleObserver.observe(document.head, { childList: true })
}

function ensureRoot(win: Window): HTMLElement {
	let root = win.document.getElementById('pip-root') as HTMLElement | null
	if (!root) {
		root = win.document.createElement('div')
		root.id = 'pip-root'
		root.style.cssText =
			'display:flex;flex-direction:column;height:100vh;width:100vw;overflow:hidden;'
		win.document.body.appendChild(root)
	}
	return root
}

function handlePagehide() {
	teardown()
	onExternalClose?.()
}

function teardown() {
	styleObserver?.disconnect()
	styleObserver = null
	pipWindow?.removeEventListener('pagehide', handlePagehide)
	pipWindow = null
}

/**
 * Returns the `#pip-root` element to teleport panel content into, opening the window the
 * first time. Reuses the existing window on subsequent calls (e.g. after a page remount).
 * Resolves to null when PiP is unsupported or the request is rejected.
 */
export async function ensurePipWindow(
	options: EnsurePipWindowOptions = {},
): Promise<HTMLElement | null> {
	onExternalClose = options.onClose ?? null

	if (isPipWindowOpen()) return ensureRoot(pipWindow!)
	if (!pipSupported) return null

	const win: Window = await (
		window as unknown as {
			documentPictureInPicture: {
				requestWindow: (opts: { width: number; height: number }) => Promise<Window>
			}
		}
	).documentPictureInPicture.requestWindow({
		width: options.width ?? 720,
		height: options.height ?? 820,
	})

	win.document.body.style.margin = '0'
	copyStyles(win)
	syncRootAttributes(win)
	watchNewStyles(win)
	win.addEventListener('pagehide', handlePagehide)

	pipWindow = win
	return ensureRoot(win)
}

/** Close the PiP window from our side (user hit the toggle / reclaimed its tabs). */
export function closePipWindow() {
	const win = pipWindow
	teardown()
	win?.close()
}
