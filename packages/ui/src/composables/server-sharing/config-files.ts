const SHAREABLE_CONFIG_EXTENSIONS = new Set([
	'json',
	'json5',
	'jsonc',
	'yml',
	'yaml',
	'css',
	'toml',
	'txt',
	'ini',
	'cfg',
	'conf',
	'properties',
	'xml',
	'nbt',
])

/** Whether a file or folder name inside `config/` can be included in a shared instance push. */
export function isShareableConfigEntryName(name: string) {
	return !name.startsWith('.') && !/[/\\:]/.test(name) && !/[. ]$/.test(name)
}

export function hasShareableConfigExtension(name: string) {
	return SHAREABLE_CONFIG_EXTENSIONS.has(name.split('.').pop()?.toLowerCase() ?? '')
}

/**
 * Normalizes a server file path to the `config/...` form accepted by the share endpoint,
 * or returns `undefined` when the file can't be shared as a config file.
 */
export function toShareableConfigPath(path: string): string | undefined {
	const segments = path.split('/').filter(Boolean)
	if (segments.length < 2 || segments[0] !== 'config') return
	const entries = segments.slice(1)
	if (!entries.every(isShareableConfigEntryName)) return
	if (!hasShareableConfigExtension(entries[entries.length - 1])) return
	return segments.join('/')
}
