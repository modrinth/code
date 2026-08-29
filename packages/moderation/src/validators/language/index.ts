import { francAll } from 'franc-min'

export interface LanguageDetection {
	language: string
	accuracy: number
}

export interface EnglishTextResult {
	valid: boolean
	detections: LanguageDetection[]
}

export const MIN_LANGUAGE_DETECTION_WORDS = 8
export const MIN_LANGUAGE_DETECTION_CHARACTERS = 35
export const MIN_ENGLISH_SCORE = 0.45

const wordSegmenter = new Intl.Segmenter(undefined, { granularity: 'word' })
const characterSegmenter = new Intl.Segmenter(undefined, { granularity: 'grapheme' })

function hasEnoughCharacters(text: string): boolean {
	let characterCount = 0

	for (const _ of characterSegmenter.segment(text.trim())) {
		if (++characterCount >= MIN_LANGUAGE_DETECTION_CHARACTERS) return true
	}

	return false
}

function hasEnoughWords(text: string): boolean {
	let wordCount = 0

	for (const { isWordLike } of wordSegmenter.segment(text)) {
		if (isWordLike && ++wordCount >= MIN_LANGUAGE_DETECTION_WORDS) return true
	}

	return false
}

export function validateEnglishText(text: string): EnglishTextResult {
	if (!hasEnoughCharacters(text) || !hasEnoughWords(text)) {
		return { valid: true, detections: [] }
	}

	const results = francAll(text)
	const englishScore = results.find(([language]) => language === 'eng')?.[1] ?? 0
	const detections = results.map(([language, accuracy]) => ({ language, accuracy }))

	return {
		valid: englishScore > MIN_ENGLISH_SCORE,
		detections,
	}
}

export const validateEnglishSummaryText = validateEnglishText
