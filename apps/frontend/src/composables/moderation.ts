import type {CookieOptions} from '#app'
import {
	type KeybindDefinition,
	type KeybindListener,
	keybinds,
	normalizeKeybind
} from "@modrinth/moderation";

const moderationKeybindsId = 'moderation-keybinds'

type StoredKeybinds = { [id: string]: KeybindDefinition[] }
type PartialStoredKeybinds = Partial<StoredKeybinds>

const getCookieOptions = () =>
	({
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: useRuntimeConfig().public.cookieSecure,
		httpOnly: false,
		path: '/',
	}) satisfies CookieOptions<PartialStoredKeybinds>

export const useModerationKeybinds = () =>
	useState<{ [id: string]: KeybindListener }>(moderationKeybindsId, () => {
		const storedKeybinds = useCookie<PartialStoredKeybinds>(moderationKeybindsId, getCookieOptions())

		if (!storedKeybinds.value) {
			storedKeybinds.value = {}
		}

		const output: { [id: string]: KeybindListener } = {}

		for (const [id, keybind] of Object.entries(keybinds)) {
			const definitions = storedKeybinds.value[id]
			output[id] = {
				keybind: definitions !== undefined ? definitions : keybind.keybind,
				description: keybind.description,
				enabled: keybind.enabled,
				action: keybind.action,
			}
		}

		return output
	})

export const saveModerationKeybinds = () => {
	const keybinds = useModerationKeybinds()
	const cookie = useCookie<PartialStoredKeybinds>(moderationKeybindsId, getCookieOptions())

	const storedKeybinds: PartialStoredKeybinds = {}
	for (const [id, keybind] of Object.entries(keybinds.value)) {
		storedKeybinds[id] = (Array.isArray(keybind.keybind) ? keybind.keybind : [keybind.keybind]).map(normalizeKeybind)
	}
	cookie.value = storedKeybinds
}
