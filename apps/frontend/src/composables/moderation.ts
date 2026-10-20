import { type KeybindDefinition, Keybinds, Settings } from '@modrinth/moderation'
import { computed } from 'vue'

import type { CookieOptions } from '#app'
import {
	defaultAppearance,
	parseAppearance,
	type AppearanceValues,
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
