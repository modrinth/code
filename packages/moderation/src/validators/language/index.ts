import { francAll } from 'franc-min'

export interface LanguageDetection {
	language: string
	accuracy: number
}

export interface EnglishTextResult {
	valid: boolean
	detections: LanguageDetection[]
	reasons: EnglishTextFailureReason[]
}

export type EnglishTextFailureReason = 'insufficient-english-chunk-coverage'

export interface LanguageChunkAnalysis {
	totalChunks: number
	englishChunks: number
	nonEnglishChunks: number
	ambiguousChunks: number
	englishChunkPercentage: number | null
}

export const MIN_LANGUAGE_DETECTION_WORDS = 8
export const MIN_LANGUAGE_DETECTION_CHARACTERS = 35
export const MIN_ENGLISH_SCORE = 0.8
export const MIN_ENGLISH_SUMMARY_SCORE = 0.5
export const LANGUAGE_CHUNK_WORDS = 24
export const LANGUAGE_CHUNK_STRIDE_WORDS = 12
export const MIN_ENGLISH_CHUNK_PERCENTAGE = 0.3

const wordSegmenter = new Intl.Segmenter(undefined, { granularity: 'word' })
const characterSegmenter = new Intl.Segmenter(undefined, { granularity: 'grapheme' })

function hasEnoughCharacters(text: string): boolean {
	let characterCount = 0

	for (const _ of characterSegmenter.segment(text.trim())) {
		if (++characterCount >= MIN_LANGUAGE_DETECTION_CHARACTERS) return true
	}

	return false
}

function getWords(text: string): string[] {
	return [...wordSegmenter.segment(text)]
		.filter(({ isWordLike }) => isWordLike)
		.map(({ segment }) => segment)
}

function getWordWindows(block: string): string[] {
	const words = getWords(block)
	if (words.length < MIN_LANGUAGE_DETECTION_WORDS) return []
	if (words.length <= LANGUAGE_CHUNK_WORDS) {
		const window = words.join(' ')
		return hasEnoughCharacters(window) ? [window] : []
	}

	const starts = new Set<number>()
	for (
		let start = 0;
		start + MIN_LANGUAGE_DETECTION_WORDS <= words.length;
		start += LANGUAGE_CHUNK_STRIDE_WORDS
	) {
		starts.add(start)
	}
	starts.add(words.length - LANGUAGE_CHUNK_WORDS)

	return [...starts]
		.sort((left, right) => left - right)
		.map((start) => words.slice(start, start + LANGUAGE_CHUNK_WORDS).join(' '))
		.filter(hasEnoughCharacters)
}

export function analyzeLanguageChunks(blocks: string[]): LanguageChunkAnalysis {
	let englishChunks = 0
	let nonEnglishChunks = 0
	let ambiguousChunks = 0

	for (const chunk of blocks.flatMap(getWordWindows)) {
		const results = francAll(chunk)
		const primaryLanguage = results[0]?.[0]
		const englishScore = results.find(([language]) => language === 'eng')?.[1] ?? 0

		if (primaryLanguage === 'eng') englishChunks++
		else if (primaryLanguage && englishScore < MIN_ENGLISH_SCORE) nonEnglishChunks++
		else ambiguousChunks++
	}

	const totalChunks = englishChunks + nonEnglishChunks + ambiguousChunks
	const classifiedChunks = englishChunks + nonEnglishChunks
	return {
		totalChunks,
		englishChunks,
		nonEnglishChunks,
		ambiguousChunks,
		englishChunkPercentage: classifiedChunks === 0 ? null : englishChunks / classifiedChunks,
	}
}

export function validateEnglishTextBlocks(blocks: string[]): EnglishTextResult {
	const normalizedBlocks = blocks.map((block) => block.trim()).filter(Boolean)
	const text = normalizedBlocks.join('\n')
	const chunkAnalysis = analyzeLanguageChunks(normalizedBlocks)
	const valid =
		chunkAnalysis.englishChunkPercentage === null ||
		chunkAnalysis.englishChunkPercentage >= MIN_ENGLISH_CHUNK_PERCENTAGE
	const detections =
		getWords(text).length >= MIN_LANGUAGE_DETECTION_WORDS && hasEnoughCharacters(text)
			? francAll(text).map(([language, accuracy]) => ({ language, accuracy }))
			: []

	return {
		valid,
		detections,
		reasons: valid ? [] : ['insufficient-english-chunk-coverage'],
	}
}

export function validateEnglishText(text: string): EnglishTextResult {
	return validateEnglishTextBlocks(text.split(/\n+/))
}

export function validateEnglishSummaryText(text: string): EnglishTextResult {
	const normalizedText = text.trim()
	if (
		getWords(normalizedText).length < MIN_LANGUAGE_DETECTION_WORDS ||
		!hasEnoughCharacters(normalizedText)
	) {
		return { valid: true, detections: [], reasons: [] }
	}

	const detections = francAll(normalizedText).map(([language, accuracy]) => ({
		language,
		accuracy,
	}))
	const englishScore = detections.find(({ language }) => language === 'eng')?.accuracy ?? 0
	const valid = englishScore >= MIN_ENGLISH_SUMMARY_SCORE

	return {
		valid,
		detections,
		reasons: valid ? [] : ['insufficient-english-chunk-coverage'],
	}
}
