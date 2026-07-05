/**
 * Extracts a collection ID from user input, accepting either a raw collection ID
 * or a Modrinth collection URL (e.g. https://modrinth.com/collection/AbCdEf12).
 */
export function parseCollectionId(input: string): string | null {
	const trimmed = input.trim()
	if (!trimmed) return null

	const urlMatch = trimmed.match(/collection\/([a-zA-Z0-9]+)/)
	if (urlMatch) return urlMatch[1]

	if (/^[a-zA-Z0-9]+$/.test(trimmed)) return trimmed

	return null
}

/**
 * Determines when url is pasted
 */
export function isCollectionLink(input: string): boolean {
	return /collection\/[a-zA-Z0-9]+/.test(input.trim())
}
