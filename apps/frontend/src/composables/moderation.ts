import { type KeybindDefinition, Keybinds, Settings } from '@modrinth/moderation'
import { computed } from 'vue'

import type {CookieOptions, CookieRef} from '#app'
import { FrontendNotificationManager } from '~/providers/frontend-notifications'

const moderationKeybindsId = 'moderation-keybinds'
const moderationSettingsId = 'moderation-settings'

type StoredKeybinds = { [id: string]: KeybindDefinition[] }
type StoredSettings = { [id: string]: any }
type PartialStoredOptions = {
	keybinds: Partial<StoredKeybinds>
	settings: Partial<StoredSettings>
}

type StoredOptions = { keybinds: StoredKeybinds; settings: StoredSettings };

const getCookieOptions = <T>() =>
	({
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: useRuntimeConfig().public.cookieSecure,
		httpOnly: false,
		path: '/',
		watch: true
	}) satisfies CookieOptions<T>;

let keybindCookie: CookieRef<Partial<StoredKeybinds> | null> | null = null;
let optionsCookie: CookieRef<Partial<PartialStoredOptions> | null> | null = null

const keybindCookieGetter = () => {
	if (keybindCookie == null) {
		keybindCookie = useCookie<Partial<StoredKeybinds> | null>(
			moderationKeybindsId,
			getCookieOptions(),
		);
	}
	return keybindCookie!;
}

const optionsCookieGetter = () => {
	if (optionsCookie == null) {
		optionsCookie = useCookie<Partial<PartialStoredOptions> | null>(
			moderationSettingsId,
			getCookieOptions(),
		)
	}
	return optionsCookie!;
}

const useModerationCookies = computed(
	() => {
		const keybindCookieRef = keybindCookieGetter();
		const optionsCookieRef = optionsCookieGetter();

		const options: Partial<PartialStoredOptions> = optionsCookieRef.value ?? { keybinds: {}, settings: {} };

		if (keybindCookieRef.value) {
			options.keybinds = keybindCookieRef.value;
			keybindCookieRef.value = null
		}

		return options;
	}
)

let moderationOptions: ComputedRef<StoredOptions> | null = null;

const useModerationOptions = () => {
	if (moderationOptions == null) {
		moderationOptions = computed<{ keybinds: StoredKeybinds; settings: StoredSettings }>( () => {
			const stored = useModerationCookies.value;

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
	}

	return moderationOptions!;
}

export const useModerationKeybinds = () =>
	computed(() => new Keybinds(useModerationOptions().value.keybinds))

export const useModerationSettings = () =>
	computed(() => new Settings(useModerationOptions().value.settings, saveModerationOptions))

export const saveModerationOptions = () => {
	const options = useModerationOptions();

	optionsCookieGetter().value = {
		keybinds: options.value.keybinds,
		settings: options.value.settings,
	}

	refreshCookie(moderationSettingsId);
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
