declare global {
	interface LocaleResources {
		'languages.json'?: Partial<Record<string, string>>
	}

	interface LocaleMeta {
		displayName?: string
		category?: string
		searchTerms?: string
	}
}

export {}
