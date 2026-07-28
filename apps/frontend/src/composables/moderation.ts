import { type KeybindDefinition, Keybinds, Settings } from '@modrinth/moderation'
import { computed } from 'vue'

import type { CookieOptions } from '#app'

const moderationKeybindsId = 'moderation-keybinds'
const moderationSettingsId = 'moderation-settings'

type StoredKeybinds = { [id: string]: KeybindDefinition[] }
type StoredSettings = { [id: string]: any }
type StoredOptions = {
	keybinds: Partial<StoredKeybinds>
	settings: Partial<StoredSettings>
}

const getCookieOptions = <T>() =>
	({
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: useRuntimeConfig().public.cookieSecure,
		httpOnly: false,
		path: '/',
	}) satisfies CookieOptions<T>

const useModerationCookies = () => {
	const keybindCookie = useCookie<Partial<StoredKeybinds> | null>(
		moderationKeybindsId,
		getCookieOptions(),
	)
	const optionsCookie = useCookie<Partial<StoredOptions> | null>(
		moderationSettingsId,
		getCookieOptions(),
	)

	if (keybindCookie.value && !optionsCookie.value) {
		optionsCookie.value = {
			keybinds: keybindCookie.value,
			settings: {},
		}
		keybindCookie.value = null
	} else if (keybindCookie.value && optionsCookie.value) {
		keybindCookie.value = null // options is the new cookie so it will override the existing keybinds.
	}

	return optionsCookie
}

const useModerationOptions = () =>
	useState<{ keybinds: StoredKeybinds; settings: StoredSettings }>(moderationKeybindsId, () => {
		const cookie = useModerationCookies()
		const stored = cookie.value ?? {}

		const keybindOutput: StoredKeybinds = {}

		for (const [id, definition] of Object.entries(stored.keybinds || {})) {
			if (!definition) continue
			keybindOutput[id] = definition
		}

		const settingsOutput: StoredSettings = {}
		for (const [id, setting] of Object.entries(stored.settings || {})) {
			settingsOutput[id] = setting
		}

		return {
			keybinds: keybindOutput,
			settings: settingsOutput,
		}
	})

export const useModerationKeybinds = () =>
	computed(() => new Keybinds(useModerationOptions().value.keybinds))

export const useModerationSettings = () =>
	computed(() => new Settings(useModerationOptions().value.settings, saveModerationOptions))

export const saveModerationOptions = () => {
	const options = useModerationOptions()
	const cookie = useModerationCookies()

	cookie.value = {
		keybinds: options.value.keybinds,
		settings: options.value.settings,
	}
}
