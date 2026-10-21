import { type KeybindDefinition, Keybinds, Settings } from '@modrinth/moderation'
import { computed } from 'vue'

import type { CookieOptions } from '#app'
import {
	type AppearanceValues,
	defaultAppearance,
	parseAppearance,
} from '~/helpers/moderation-appearance'
import { FrontendNotificationManager } from '~/providers/frontend-notifications'

const moderationKeybindsId = 'moderation-keybinds'
const moderationSettingsId = 'moderation-settings'

type StoredKeybinds = { [id: string]: KeybindDefinition[] }
type StoredSettings = { [id: string]: any }
type StoredOptions = {
	keybinds: Partial<StoredKeybinds>
	settings: Partial<StoredSettings>
	appearanceTheme?: { id: string; values: AppearanceValues }
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
	useState<StoredOptions & { keybinds: StoredKeybinds; settings: StoredSettings }>(
		moderationKeybindsId,
		() => {
			const cookie = useModerationCookies()
			const stored = cookie.value ?? {}

			const keybindOutput: StoredKeybinds = {}

			for (const [id, definition] of Object.entries(stored.keybinds || {})) {
				if (!definition) continue
				keybindOutput[id] = definition
			}

			if (!('review-cycle-conversation' in keybindOutput)) {
				const reply = keybindOutput['review-tab-reply']
				const note = keybindOutput['review-tab-note']
				if (reply?.length) keybindOutput['review-cycle-conversation'] = reply
				else if (note?.length) keybindOutput['review-cycle-conversation'] = note
				else if (reply || note) keybindOutput['review-cycle-conversation'] = []
			}
			delete keybindOutput['review-tab-reply']
			delete keybindOutput['review-tab-note']

			const settingsOutput: StoredSettings = {}
			for (const [id, setting] of Object.entries(stored.settings || {})) {
				settingsOutput[id] = setting
			}

			let appearanceTheme: StoredOptions['appearanceTheme']
			try {
				if (stored.appearanceTheme) {
					appearanceTheme = {
						id: 'custom',
						values: parseAppearance(
							JSON.stringify({
								format: 'modrinth-review-appearance',
								version: 1,
								values: stored.appearanceTheme.values,
							}),
						),
					}
				}
			} catch {
				appearanceTheme = undefined
			}
			return {
				keybinds: keybindOutput,
				settings: settingsOutput,
				appearanceTheme,
			}
		},
	)

export const useModerationKeybinds = () =>
	computed(() => new Keybinds(useModerationOptions().value.keybinds))

const booleanFields = ['mod', 'ctrl', 'shift', 'alt', 'meta', 'preventDefault'] as const

function compactDefinition(definition: KeybindDefinition): KeybindDefinition {
	const portableMod = definition.mod || (definition.ctrl && definition.meta !== false)
	return {
		key: definition.key.toLowerCase(),
		...(portableMod ? { mod: true } : {}),
		...(!portableMod && definition.ctrl ? { ctrl: true, meta: false } : {}),
		...(!portableMod && definition.meta ? { meta: true } : {}),
		...(definition.shift ? { shift: true } : {}),
		...(definition.alt ? { alt: true } : {}),
		...(definition.preventDefault === false ? { preventDefault: false } : {}),
	}
}

function readOverrides(keybinds: Keybinds): StoredKeybinds {
	return Object.fromEntries(
		[...keybinds].flatMap(([id, binding]) => {
			const definitions = binding.keybind.map(compactDefinition)
			const defaults = binding.defaultKeybind.map(compactDefinition)
			return JSON.stringify(definitions) === JSON.stringify(defaults) ? [] : [[id, definitions]]
		}),
	)
}

export function exportKeybinds(keybinds: Keybinds): string {
	return JSON.stringify(
		{
			format: 'modrinth-review-keybinds',
			version: 1,
			values: readOverrides(keybinds),
		},
		null,
		2,
	)
}

export function parseKeybinds(source: string): StoredKeybinds {
	if (new TextEncoder().encode(source).length > 65536) throw new Error('size')
	const data = JSON.parse(source)
	if (
		!data ||
		data.format !== 'modrinth-review-keybinds' ||
		data.version !== 1 ||
		!data.values ||
		typeof data.values !== 'object' ||
		Array.isArray(data.values)
	)
		throw new Error('format')
	const knownIds = new Set([...new Keybinds({})].map(([id]) => id))
	const values: StoredKeybinds = {}
	for (const [id, definitions] of Object.entries(data.values)) {
		if (!knownIds.has(id) || !Array.isArray(definitions)) throw new Error('action')
		values[id] = definitions.map((definition) => {
			if (
				!definition ||
				typeof definition !== 'object' ||
				Array.isArray(definition) ||
				typeof definition.key !== 'string' ||
				!definition.key.length ||
				definition.key.length > 64 ||
				Object.keys(definition).some(
					(key) => key !== 'key' && !booleanFields.some((field) => field === key),
				) ||
				booleanFields.some((field) => field in definition && typeof definition[field] !== 'boolean')
			)
				throw new Error('definition')
			return compactDefinition(definition)
		})
	}
	return readOverrides(new Keybinds(values))
}

export function replaceModerationKeybinds(keybinds: StoredKeybinds) {
	const options = useModerationOptions()
	const nextOptions = { ...options.value, keybinds }
	if (encodeURIComponent(JSON.stringify(nextOptions)).length > 3800) throw new Error('storage-size')
	options.value.keybinds = keybinds
	saveModerationOptions()
}

export const useModerationSettings = () =>
	computed(() => new Settings(useModerationOptions().value.settings, saveModerationOptions))

export const useModerationAppearanceTheme = () =>
	computed({
		get: () =>
			useModerationOptions().value.appearanceTheme ?? { id: 'default', values: defaultAppearance },
		set: (theme: NonNullable<StoredOptions['appearanceTheme']>) => {
			useModerationOptions().value.appearanceTheme = theme
			saveModerationOptions()
		},
	})

export const saveModerationOptions = () => {
	const options = useModerationOptions()
	const cookie = useModerationCookies()

	cookie.value = {
		keybinds: options.value.keybinds,
		settings: options.value.settings,
		appearanceTheme: options.value.appearanceTheme,
	}
}

let copyNotificationManager: FrontendNotificationManager | undefined

export function notifyCopied(value: string, title: string) {
	copyNotificationManager ??= new FrontendNotificationManager()
	copyNotificationManager.addNotification({
		type: 'success',
		title,
		text: value,
		autoCloseMs: 1000,
	})
}
