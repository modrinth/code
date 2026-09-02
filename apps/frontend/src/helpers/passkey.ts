function ensurePasskeySupported() {
	const supported =
		typeof window !== 'undefined' &&
		typeof window.PublicKeyCredential !== 'undefined' &&
		typeof navigator !== 'undefined' &&
		!!navigator.credentials
	if (!supported) {
		throw new Error('Passkeys are not supported by this browser.')
	}
}

function base64urlToBuffer(base64url: string) {
	return Uint8Array.from(atob(base64url.replace(/-/g, '+').replace(/_/g, '/')), (char) =>
		char.charCodeAt(0),
	)
}

function bufferToBase64url(buffer: ArrayBuffer) {
	const bytes = new Uint8Array(buffer)
	let str = ''
	for (let i = 0; i < bytes.length; i++) {
		str += String.fromCharCode(bytes[i])
	}
	return btoa(str).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

/**
 * Creates a passkey credential using the browser's WebAuthn API.
 *
 * @param options The public key options for creating the passkey credential, provided by the server.
 */
export async function createPasskeyCredential(options: any) {
	ensurePasskeySupported()

	const publicKey = {
		...options,
		challenge: base64urlToBuffer(options.challenge),
		user: {
			...options.user,
			id: base64urlToBuffer(options.user.id),
		},
		excludeCredentials: options.excludeCredentials?.map((cred: any) => ({
			...cred,
			id: base64urlToBuffer(cred.id),
		})),
	}

	const credential = (await navigator.credentials.create({ publicKey })) as PublicKeyCredential
	const response = credential.response as AuthenticatorAttestationResponse

	return {
		id: credential.id,
		rawId: bufferToBase64url(credential.rawId),
		type: credential.type,
		response: {
			clientDataJSON: bufferToBase64url(response.clientDataJSON),
			attestationObject: bufferToBase64url(response.attestationObject),
		},
		extensions: credential.getClientExtensionResults(),
	}
}

/**
 * Authenticates a user using a passkey credential.
 *
 * @param options The public key options for authenticating the passkey credential, provided by the server.
 */
export async function getPasskeyCredential(options: any) {
	ensurePasskeySupported()

	const publicKey = {
		...options,
		challenge: base64urlToBuffer(options.challenge),
		allowCredentials: options.allowCredentials?.map((cred: any) => ({
			...cred,
			id: base64urlToBuffer(cred.id),
		})),
	}

	const credential = (await navigator.credentials.get({ publicKey })) as PublicKeyCredential
	const response = credential.response as AuthenticatorAssertionResponse

	return {
		id: credential.id,
		rawId: bufferToBase64url(credential.rawId),
		type: credential.type,
		response: {
			clientDataJSON: bufferToBase64url(response.clientDataJSON),
			authenticatorData: bufferToBase64url(response.authenticatorData),
			signature: bufferToBase64url(response.signature),
			userHandle: response.userHandle ? bufferToBase64url(response.userHandle) : null,
		},
		extensions: credential.getClientExtensionResults(),
	}
}
