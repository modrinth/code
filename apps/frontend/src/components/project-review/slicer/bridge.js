/**
 * Slicer script injected into the embedded slicer.run iframe via its `script` URL parameter.
 * It listens for `load` requests from the Modrinth moderation page, replaces the slicer workspace
 * with the requested files when they differ from what is loaded, and opens the requested entry.
 *
 * Slicer keeps imported scripts and loads them on startup, so several versions of this script may
 * be installed at once. Only the newest one handles messages, and it announces its version in the
 * `ready` handshake so the page knows whether it needs to import an update.
 *
 * The parent origin and version placeholders below are filled in by the page before injection.
 */
const PARENT_ORIGIN = '__MODRINTH_PARENT_ORIGIN__'
const VERSION = Number('__MODRINTH_BRIDGE_VERSION__')
const SOURCE = 'modrinth-slicer-bridge'
const INSTANCE_KEY = '__modrinthSlicerBridge'

const TAB_TYPES_BY_EXTENSION = {
	image: ['jpg', 'jpeg', 'gif', 'png', 'webp'],
	media: ['mp4', 'webm', 'ogg', 'mp3', 'wav', 'flac', 'mov', 'avi', 'm4a', 'aac', 'mkv'],
	dump: ['hprof'],
}

let instance

function post(message) {
	window.parent.postMessage({ source: SOURCE, version: VERSION, ...message }, PARENT_ORIGIN)
}

/** Picks the tab type slicer itself would use when opening an entry with this name. */
function tabType(name) {
	const extension = name.split('.').pop()?.toLowerCase() ?? ''
	const match = Object.entries(TAB_TYPES_BY_EXTENSION).find(([, extensions]) =>
		extensions.includes(extension),
	)
	return match?.[0] ?? 'code'
}

/**
 * Finds a workspace entry by name. Falls back to the only entry sharing the same path below its
 * top-level file, in case the top-level name differs from what the page expected.
 */
function findEntry(context, name) {
	const exact = context.workspace.find(name)
	if (exact) return exact
	const separator = name.indexOf('/')
	if (separator === -1) return null
	const suffix = name.slice(separator)
	const matches = context.workspace.entries().filter((entry) => entry.name.endsWith(suffix))
	return matches.length === 1 ? matches[0] : null
}

/** Whether the workspace's top-level entries are exactly the given file names. */
function isLoaded(context, names) {
	const loaded = context.workspace
		.entries()
		.map((entry) => entry.name)
		.filter((name) => !name.includes('/'))
	return loaded.length === names.length && names.every((name) => loaded.includes(name))
}

export default {
	name: 'Modrinth moderation bridge',
	description: 'Lets the Modrinth project review page load version files into slicer.',
	version: '__MODRINTH_BRIDGE_VERSION__',
	load(context) {
		if (window.parent === window) return
		const existing = window[INSTANCE_KEY]
		if (existing && existing.version > VERSION) return
		existing?.dispose()

		let controller
		let pending = existing?.pending ?? Promise.resolve()
		let loadedFilesKey
		async function onMessage(event) {
			if (event.origin !== PARENT_ORIGIN) return
			// Popout panels still send messages from the main window's JavaScript context.
			if (event.source !== window.parent && event.source !== window.parent.opener) return
			const data = event.data
			if (data?.source !== SOURCE || data.type !== 'load') return
			controller?.abort()
			const request = new AbortController()
			controller = request
			post({ type: 'loading', id: data.id })
			pending = pending.then(async () => {
				if (request.signal.aborted) return
				try {
					const filesKey = JSON.stringify(data.files)
					if (
						filesKey !== loadedFilesKey ||
						!isLoaded(
							context,
							data.files.map(({ name }) => name),
						)
					) {
						const files = await Promise.all(
							data.files.map(async ({ url, name }) => {
								const response = await fetch(url, { signal: request.signal })
								if (!response.ok) throw new Error(`${name}: HTTP ${response.status}`)
								return { name, blob: await response.blob() }
							}),
						)
						if (request.signal.aborted) return
						loadedFilesKey = undefined
						await context.workspace.clear()
						for (const { name, blob } of files) {
							if (request.signal.aborted) return
							await context.workspace.add(name, blob)
						}
						if (request.signal.aborted) return
						loadedFilesKey = filesKey
					}
					if (data.entry) {
						const entry = findEntry(context, data.entry)
						if (entry) {
							await context.editor.add(tabType(entry.name), entry)
						} else {
							context.notification.warning('Could not find file in workspace', {
								description: data.entry,
							})
						}
					}
					if (!request.signal.aborted) post({ type: 'loaded', id: data.id })
				} catch (error) {
					if (request.signal.aborted) return
					request.abort()
					post({ type: 'error', id: data.id })
					context.notification.error('Could not load files from Modrinth', {
						description: String(error?.message ?? error),
					})
				}
			})
			return pending
		}

		window.addEventListener('message', onMessage)
		instance = {
			version: VERSION,
			get pending() {
				return pending
			},
			dispose: () => {
				controller?.abort()
				window.removeEventListener('message', onMessage)
			},
		}
		window[INSTANCE_KEY] = instance
		post({ type: 'ready', version: VERSION })
	},
	unload() {
		if (!instance || window[INSTANCE_KEY] !== instance) return
		instance.dispose()
		delete window[INSTANCE_KEY]
		instance = undefined
	},
}
