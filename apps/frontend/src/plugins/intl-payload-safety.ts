export default definePayloadPlugin(() => {
	definePayloadReducer('IntlMessageFormat', (value) => {
		if (value?.constructor?.name === 'IntlMessageFormat' || value?._ast !== undefined) {
			if (import.meta.dev) {
				console.warn('[i18n] IntlMessageFormat instance leaked into payload - returning null')
				console.warn('[i18n] This indicates a bug that should be fixed upstream')
				console.warn('[i18n] Leaked value:', value)
			}

			return null
		}

		return false
	})

	definePayloadReviver('IntlMessageFormat', () => null)
})
