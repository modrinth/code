export function joinDisplayPath(basePath: string | undefined, itemPath: string) {
	if (!basePath) return itemPath

	const separator = basePath.includes('\\') ? '\\' : '/'
	const path = itemPath.replace(/^[\\/]+/, '').replace(/[\\/]+/g, separator)
	const base = basePath.replace(/[\\/]+$/, '')

	return path ? `${base}${separator}${path}` : basePath
}
