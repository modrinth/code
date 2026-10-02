import { defineMessages } from '@modrinth/ui'
import type { LocationQuery, LocationQueryValue } from 'vue-router'

export const LAUNCHER_AUTH_PROTOCOL = '2'

export const LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY = 'launcher-reauth-account-id'
export const LAUNCHER_APP_SESSION_QUERY_PARAM = 'app_session'
export const LAUNCHER_APP_CODE_QUERY_PARAM = 'app_code'

export const LAUNCHER_AUTH_QUERY_PARAMS = ['ipver', 'port', 'protocol', 'key', 'nonce'] as const

const SEAL_INFO = new TextEncoder().encode('modrinth-app-auth-v2')
const SEAL_VERSION = 0x02

type QueryValue = LocationQueryValue | LocationQueryValue[] | undefined
type QueryRoute = { query: LocationQuery }

export const launcherAuthMessages = defineMessages({
	handoffFailed: {
		id: 'auth.sign-in.launcher.handoff-failed',
		defaultMessage: "Couldn't finish signing in to the Modrinth App. Try again.",
	},
})

export const getQueryString = (value: QueryValue) => {
	if (Array.isArray(value)) {
		return value[0] ?? null
	}
	return value ?? null
}

export const isLauncherProtocolV2 = (route: QueryRoute) =>
	getQueryString(route.query.launcher) != null &&
	getQueryString(route.query.protocol) === LAUNCHER_AUTH_PROTOCOL

export const getLauncherRedirectUrl = (route: QueryRoute) => {
	const ipver = getQueryString(route.query.ipver)
	const port = Number(getQueryString(route.query.port))
	const usesLocalhostRedirectionScheme = ['4', '6'].includes(ipver ?? '') && port < 65536

	return usesLocalhostRedirectionScheme
		? `http://${ipver === '4' ? '127.0.0.1' : '[::1]'}:${port}`
		: 'https://launcher-files.modrinth.com'
}

export type LauncherHandoff =
	| { type: 'external'; url: string }
	| { type: 'callback'; localhostUrl: string | null; deeplinkUrl: string | null }

export async function createLauncherHandoff(
	route: QueryRoute,
	sessionToken: string,
): Promise<LauncherHandoff> {
	if (!isLauncherProtocolV2(route)) {
		const redirectUrl = `${getLauncherRedirectUrl(route)}/?code=${sessionToken}`
		if (redirectUrl.startsWith('https://launcher-files.modrinth.com/')) {
			return { type: 'external', url: redirectUrl }
		}

		return { type: 'callback', localhostUrl: redirectUrl, deeplinkUrl: null }
	}

	const publicKey = getQueryString(route.query.key)
	const nonce = getQueryString(route.query.nonce)
	if (!publicKey || !nonce) {
		throw new Error('Missing Modrinth App sign-in key')
	}

	const sealed = await sealLauncherSession(sessionToken, publicKey, nonce)
	const params = new URLSearchParams({ code: sealed, nonce })
	const redirectBase = getLauncherRedirectUrl(route)

	return {
		type: 'callback',
		localhostUrl: redirectBase.startsWith('http://')
			? `${redirectBase}/?${params.toString()}`
			: null,
		deeplinkUrl: `modrinth://auth?${params.toString()}`,
	}
}

export function hideLauncherSessionCode() {
	if (!import.meta.client) return

	const url = new URL(window.location.href)
	if (!url.searchParams.has('code')) return

	url.searchParams.delete('code')
	url.searchParams.delete(LAUNCHER_APP_CODE_QUERY_PARAM)
	window.history.replaceState(window.history.state, '', `${url.pathname}${url.search}${url.hash}`)
}

async function sealLauncherSession(token: string, publicKey: string, nonce: string) {
	const recipient = await crypto.subtle.importKey(
		'spki',
		base64UrlToBytes(publicKey),
		{ name: 'ECDH', namedCurve: 'P-256' },
		false,
		[],
	)
	const ephemeral = await crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, [
		'deriveBits',
	])
	const shared = await crypto.subtle.deriveBits(
		{ name: 'ECDH', public: recipient },
		ephemeral.privateKey,
		256,
	)
	const hkdfKey = await crypto.subtle.importKey('raw', shared, 'HKDF', false, ['deriveBits'])
	const aesKeyBytes = await crypto.subtle.deriveBits(
		{
			name: 'HKDF',
			hash: 'SHA-256',
			salt: new Uint8Array(),
			info: SEAL_INFO,
		},
		hkdfKey,
		256,
	)
	const aesKey = await crypto.subtle.importKey('raw', aesKeyBytes, 'AES-GCM', false, ['encrypt'])
	const iv = crypto.getRandomValues(new Uint8Array(12))
	const ciphertext = new Uint8Array(
		await crypto.subtle.encrypt(
			{ name: 'AES-GCM', iv },
			aesKey,
			new TextEncoder().encode(`${nonce}.${token}`),
		),
	)
	const spki = new Uint8Array(await crypto.subtle.exportKey('spki', ephemeral.publicKey))
	const sealed = new Uint8Array(1 + 2 + spki.length + iv.length + ciphertext.length)
	sealed[0] = SEAL_VERSION
	sealed[1] = (spki.length >> 8) & 0xff
	sealed[2] = spki.length & 0xff
	sealed.set(spki, 3)
	sealed.set(iv, 3 + spki.length)
	sealed.set(ciphertext, 3 + spki.length + iv.length)

	return bytesToBase64Url(sealed)
}

function bytesToBase64Url(bytes: Uint8Array) {
	let binary = ''
	for (const byte of bytes) {
		binary += String.fromCharCode(byte)
	}

	return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '')
}

function base64UrlToBytes(value: string) {
	const padded =
		value.replaceAll('-', '+').replaceAll('_', '/') + '='.repeat((4 - (value.length % 4)) % 4)
	const binary = atob(padded)
	const bytes = new Uint8Array(binary.length)
	for (let index = 0; index < binary.length; index += 1) {
		bytes[index] = binary.charCodeAt(index)
	}

	return bytes
}
