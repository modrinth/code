const NODE_FS_PATH_REGEX = /\/modrinth\/v\d+\/fs\/?$/
const HTTP_SCHEME_REGEX = /^https?:\/\//i
const WS_SCHEME_REGEX = /^wss?:\/\//i
const HTTP_SECURE_SCHEME_REGEX = /^https:\/\//i
const HTTP_INSECURE_SCHEME_REGEX = /^http:\/\//i

export function getNodeBaseUrl(url: string): string {
	const baseUrl = url.replace(NODE_FS_PATH_REGEX, '')
	return HTTP_SCHEME_REGEX.test(baseUrl) ? baseUrl : `https://${baseUrl}`
}

export function getNodeWebSocketUrl(url: string): string {
	if (WS_SCHEME_REGEX.test(url)) return url
	if (HTTP_SECURE_SCHEME_REGEX.test(url)) return url.replace(HTTP_SECURE_SCHEME_REGEX, 'wss://')
	if (HTTP_INSECURE_SCHEME_REGEX.test(url)) return url.replace(HTTP_INSECURE_SCHEME_REGEX, 'ws://')

	return `wss://${url}`
}
