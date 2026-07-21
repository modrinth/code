import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export async function init_ads_window(overrideShown = false) {
	return await invoke('plugin:ads|init_ads_window', {
		overrideShown,
		dpr: window.devicePixelRatio,
	})
}

export async function show_ads_window() {
	return await invoke('plugin:ads|show_ads_window', { dpr: window.devicePixelRatio })
}

export async function hide_ads_window(reset) {
	return await invoke('plugin:ads|hide_ads_window', { reset })
}

export async function get_ads_consent_required() {
	return await invoke('plugin:ads|get_ads_consent_required')
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
