import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export async function init_ads_window(overrideShown = false) {
	return await invoke('plugin:ads|init_ads_window', {
		overrideShown,
		dpr: window.devicePixelRatio,
	})
}

let adsWindowHoldUpdate = Promise.resolve()

async function update_ads_window_hold(acquire) {
	adsWindowHoldUpdate = adsWindowHoldUpdate
		.catch(() => {})
		.then(() =>
			invoke('plugin:ads|update_ads_window_hold', {
				acquire,
				dpr: window.devicePixelRatio,
			}),
		)

	return await adsWindowHoldUpdate
}

export async function take_ads_window_hold() {
	return await update_ads_window_hold(true)
}

export async function release_ads_window_hold() {
	return await update_ads_window_hold(false)
}

export async function hide_ads_window(reset) {
	return await invoke('plugin:ads|hide_ads_window', { reset })
}

export async function should_show_ads_consent_popup() {
	return await invoke('plugin:ads|should_show_ads_consent_popup')
}

export async function perform_ads_consent_action(action) {
	return await invoke('plugin:ads|perform_ads_consent_action', { action })
}

export async function open_ads_consent_preferences() {
	return await invoke('plugin:ads|open_ads_consent_preferences')
}

export async function ads_consent_listener(callback) {
	return await listen('ads-consent-required', (event) => callback(event.payload))
}

export async function record_ads_click() {
	return await invoke('plugin:ads|record_ads_click')
}

export async function open_ads_link(path, origin) {
	return await invoke('plugin:ads|open_link', { path, origin })
}
