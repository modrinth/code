const PROJECT_SLUG_UNSAFE_CHARS = /[^a-zA-Z0-9._-]/g
const PROJECT_SLUG_REGEX = /^[a-zA-Z0-9._-]{3,64}$/

export function generateUrlSlug(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replaceAll(' ', '-')
		.replaceAll(PROJECT_SLUG_UNSAFE_CHARS, '')
		.replaceAll(/--+/gm, '-')
}

export function isValidProjectSlug(value: string) {
	return PROJECT_SLUG_REGEX.test(value)
}

export function generateProjectSlugSuggestions(title: string, username?: string | null) {
	const titleSlug = generateUrlSlug(title)
	const titleWords = title
		.trim()
		.split(/\s+/)
		.map((word) => generateUrlSlug(word))
		.filter(Boolean)
	const acronym = titleWords.length > 1 ? titleWords.map((word) => word[0]).join('') : ''
	const withoutDashes = titleSlug.replaceAll('-', '')
	const usernameSlug = username ? generateUrlSlug(username) : ''
	let withUsername = ''

	if (titleSlug && usernameSlug) {
		const availableTitleLength = 64 - usernameSlug.length - 1
		const truncatedTitle = titleSlug.slice(0, availableTitleLength).replace(/-+$/, '')
		if (truncatedTitle) withUsername = `${truncatedTitle}-${usernameSlug}`
	}

	return [...new Set([titleSlug, acronym, withoutDashes, withUsername])].filter(isValidProjectSlug)
}
