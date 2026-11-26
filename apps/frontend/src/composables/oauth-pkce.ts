import { onMounted, ref } from 'vue'

const VERIFIER_KEY = 'modrinth-oauth-pkce-verifier'
const LOGIN_CODE_PREFIX = 'mlc_'

const claimedLoginCodes = new Set<string>()

export function isLoginCode(code: string) {
	return code.startsWith(LOGIN_CODE_PREFIX) && isLoginCodeBody(code.slice(LOGIN_CODE_PREFIX.length))
}

export function claimLoginCode(code: string) {
	if (claimedLoginCodes.has(code)) {
		return false
	}
	claimedLoginCodes.add(code)
	return true
}

export function readOauthCodeVerifier() {
	if (!import.meta.client) {
		return null
	}

	const verifier = window.sessionStorage.getItem(VERIFIER_KEY)
	if (!verifier || !isPkceVerifier(verifier)) {
		return null
	}
	return verifier
}

export function clearOauthCodeVerifier() {
	if (!import.meta.client) {
		return
	}
	window.sessionStorage.removeItem(VERIFIER_KEY)
}

export async function ensureOauthCodeChallenge() {
	let verifier = readOauthCodeVerifier()
	if (!verifier) {
		verifier = createVerifier()
		window.sessionStorage.setItem(VERIFIER_KEY, verifier)
	}
	return codeChallengeForVerifier(verifier)
}

export function useOAuthCodeChallenge() {
	const challenge = ref<string | null>(null)

	onMounted(() => {
		void ensureOauthCodeChallenge()
			.then((value) => {
				challenge.value = value
			})
			.catch((error: unknown) => {
				console.error(error)
			})
	})

	return challenge
}

function createVerifier() {
	return bytesToBase64Url(crypto.getRandomValues(new Uint8Array(32)))
}

async function codeChallengeForVerifier(verifier: string) {
	const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier))
	return bytesToBase64Url(new Uint8Array(digest))
}

function isPkceVerifier(value: string) {
	return /^[A-Za-z0-9._~-]{43,128}$/.test(value)
}

function isLoginCodeBody(value: string) {
	return /^[A-Za-z0-9]{43}$/.test(value)
}

function bytesToBase64Url(bytes: Uint8Array) {
	let binary = ''
	for (const byte of bytes) {
		binary += String.fromCharCode(byte)
	}

	return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '')
}
