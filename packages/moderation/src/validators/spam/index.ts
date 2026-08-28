export type SpamMatchKind = 'character' | 'word' | 'phrase'

export interface SpamMatch {
	kind: SpamMatchKind
	value: string
}

export interface SpamResult {
	valid: boolean
	firstMatch?: SpamMatch
}

export const MIN_REPEATED_CHARACTER_COUNT = 8
export const MIN_REPEATED_WORD_COUNT = 4
export const MIN_REPEATED_PHRASE_COUNT = 3
export const MAX_REPEATED_PHRASE_WORDS = 8

const REPEATABLE_CHARACTER_PATTERN = /\S/u
const WORD_PATTERN = /[\p{L}\p{M}\p{N}]+/gu

function findRepeatedCharacter(text: string): SpamMatch | undefined {
	let previousCharacter: string | undefined
	let repeatedCount = 0

	for (const character of text.normalize('NFC')) {
		const normalizedCharacter = character.toLowerCase()
		if (
			REPEATABLE_CHARACTER_PATTERN.test(normalizedCharacter) &&
			normalizedCharacter === previousCharacter
		) {
			repeatedCount++
		} else {
			previousCharacter = normalizedCharacter
			repeatedCount = REPEATABLE_CHARACTER_PATTERN.test(normalizedCharacter) ? 1 : 0
		}

		if (repeatedCount >= MIN_REPEATED_CHARACTER_COUNT) {
			return { kind: 'character', value: character.repeat(repeatedCount) }
		}
	}

	return undefined
}

function getWords(text: string): string[] {
	return [...text.normalize('NFC').toLowerCase().matchAll(WORD_PATTERN)].map((match) => match[0])
}

function findRepeatedWord(words: readonly string[]): SpamMatch | undefined {
	let repeatedCount = 1

	for (let index = 1; index < words.length; index++) {
		repeatedCount = words[index] === words[index - 1] ? repeatedCount + 1 : 1
		if (repeatedCount >= MIN_REPEATED_WORD_COUNT) {
			return { kind: 'word', value: words[index] }
		}
	}

	return undefined
}

function phrasesMatch(
	words: readonly string[],
	firstStart: number,
	secondStart: number,
	size: number,
) {
	for (let offset = 0; offset < size; offset++) {
		if (words[firstStart + offset] !== words[secondStart + offset]) return false
	}

	return true
}

function findRepeatedPhrase(words: readonly string[]): SpamMatch | undefined {
	const maxPhraseWords = Math.min(
		MAX_REPEATED_PHRASE_WORDS,
		Math.floor(words.length / MIN_REPEATED_PHRASE_COUNT),
	)

	for (let phraseWords = maxPhraseWords; phraseWords >= 2; phraseWords--) {
		const repeatedWords = phraseWords * MIN_REPEATED_PHRASE_COUNT
		for (let start = 0; start + repeatedWords <= words.length; start++) {
			let matches = true
			for (let repetition = 1; repetition < MIN_REPEATED_PHRASE_COUNT; repetition++) {
				if (!phrasesMatch(words, start, start + repetition * phraseWords, phraseWords)) {
					matches = false
					break
				}
			}

			if (matches) {
				return { kind: 'phrase', value: words.slice(start, start + phraseWords).join(' ') }
			}
		}
	}

	return undefined
}

/**
  The spam validator checks normalized, readable description text in this order:
		Characters: the same non-whitespace character repeated 8 times consecutively, e.g. aaaaaaaa.
		Words: the same word repeated 4 times consecutively, case-insensitively, e.g. Great great GREAT great.
		Phrases: a 2–8 word phrase repeated 3 times consecutively. Punctuation and capitalization are ignored, so best project, best project! BEST PROJECT is rejected.
 */
export function validateSpam(text: string): SpamResult {
	const words = getWords(text)
	const firstMatch =
		findRepeatedCharacter(text) ?? findRepeatedWord(words) ?? findRepeatedPhrase(words)

	return firstMatch ? { valid: false, firstMatch } : { valid: true }
}
