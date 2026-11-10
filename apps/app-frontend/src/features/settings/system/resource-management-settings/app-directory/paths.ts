export function normalizePath(path: string, caseInsensitive: boolean): string {
	const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '')
	return caseInsensitive ? normalized.toLowerCase() : normalized
}

export function isNestedPath(first: string, second: string): boolean {
	return first.startsWith(`${second}/`) || second.startsWith(`${first}/`)
}

const DRIVE_ROOTS = [
	/^$/,
	/^[a-z]:$/i,
	/^\/Volumes\/[^/]+$/,
	/^\/mnt\/[^/]+$/,
	/^(\/run)?\/media\/[^/]+\/[^/]+$/,
]

export function isDriveRoot(path: string): boolean {
	const normalized = normalizePath(path, false)
	return DRIVE_ROOTS.some((root) => root.test(normalized))
}
