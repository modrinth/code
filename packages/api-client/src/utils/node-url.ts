const NODE_FS_PATH_REGEX = /\/modrinth\/v\d+\/fs\/?$/
const HTTP_SCHEME_REGEX = /^https?:\/\//i

export function getNodeBaseUrl(url: string): string {
	const baseUrl = url.replace(NODE_FS_PATH_REGEX, '')
	return HTTP_SCHEME_REGEX.test(baseUrl) ? baseUrl : `https://${baseUrl}`
}
