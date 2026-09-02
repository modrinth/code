export type LanguageProduct = 'app' | 'website'

export interface LanguageCoverageStats {
	percentage: number
	interfaceCoverage: number
	translationCoverage: number
	translatedMessages: number
	totalMessages: number
	unlocalizedStrings: number
}

export type LanguageCoverageByProduct = Record<
	LanguageProduct,
	Record<string, LanguageCoverageStats>
>
