const PROJECT_SLUG_UNSAFE_CHARS = /[^a-zA-Z0-9._-]/g

export function generateUrlSlug(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replaceAll(' ', '-')
		.replaceAll(PROJECT_SLUG_UNSAFE_CHARS, '')
		.replaceAll(/--+/gm, '-')
}
