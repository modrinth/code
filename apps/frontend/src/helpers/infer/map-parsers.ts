export function hasWorldRoot(entries: string[]) {
	return entries.includes('level.dat')
}

export function hasWorldData(entries: string[]) {
	return (
		entries.some((entry) => entry.startsWith('region/')) ||
		entries.some((entry) => entry.startsWith('data/'))
	)
}

export function isLikelyWorldSave(entries: string[]) {
	return hasWorldRoot(entries) && hasWorldData(entries)
}
