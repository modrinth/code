/**
 * Slicer script injected into the embedded slicer.run iframe via its `script` URL parameter.
 * It listens for `load` requests from the Modrinth moderation page and replaces the slicer
 * workspace with the requested files.
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

let instance

function post(message) {
	window.parent.postMessage({ source: SOURCE, ...message }, PARENT_ORIGIN)
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

		let latestId
		async function onMessage(event) {
			if (event.source !== window.parent || event.origin !== PARENT_ORIGIN) return
			const data = event.data
			if (data?.source !== SOURCE || data.type !== 'load') return
			latestId = data.id
			try {
				const files = await Promise.all(
					data.files.map(async ({ url, name }) => {
						const response = await fetch(url)
						if (!response.ok) throw new Error(`${name}: HTTP ${response.status}`)
						return { name, blob: await response.blob() }
					}),
				)
				if (latestId !== data.id) return
				await context.editor.clear()
				await context.workspace.clear()
				for (const { name, blob } of files) {
					await context.workspace.add(name, blob)
				}
			} catch (error) {
				context.notification.error('Could not load files from Modrinth', {
					description: String(error?.message ?? error),
				})
			}
		}

		window.addEventListener('message', onMessage)
		instance = {
			version: VERSION,
			dispose: () => window.removeEventListener('message', onMessage),
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
