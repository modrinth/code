const NODE_AUTH_PATH_REGEX = /\/modrinth\/v\d+\/(?:fs|ws)\/?$/
const HTTP_SCHEME_REGEX = /^https?:\/\//i
const WS_SCHEME_REGEX = /^wss?:\/\//i

export function getNodeBaseUrl(url: string): string {
	const baseUrl = url.replace(NODE_AUTH_PATH_REGEX, '')
	return HTTP_SCHEME_REGEX.test(baseUrl) ? baseUrl : `https://${baseUrl}`
}

export function getNodeWebSocketUrl(url: string): string {
	const baseUrl = url.replace(NODE_AUTH_PATH_REGEX, '')
	return WS_SCHEME_REGEX.test(baseUrl) ? baseUrl : `wss://${baseUrl}`
}
