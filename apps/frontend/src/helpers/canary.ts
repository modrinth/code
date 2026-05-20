export const LABRINTH_CANARY_HEADER = 'Labrinth-Canary'
export const LABRINTH_CANARY_VALUE = 'always'

export function withLabrinthCanaryHeader(headers: Record<string, string> = {}) {
	const flags = useFeatureFlags()

	if (!flags.value.labrinthApiCanary) {
		return headers
	}

	return {
		...headers,
		[LABRINTH_CANARY_HEADER]: LABRINTH_CANARY_VALUE,
	}
}
