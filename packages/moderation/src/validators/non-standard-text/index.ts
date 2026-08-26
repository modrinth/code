// the following are non-standard text that are detected
export type NonStandardTextIssueKind =
	| 'fancy' // styled characters such as `𝐀`, `Ⓐ`, or `Ａ`.
	| 'zalgo' // detached or excessive combining marks such as `a̴̵̶`.
	| 'invisible' // hidden formatting such as a word joiner or bidi override.
	| 'control' // disallowed control characters such as a null byte.
	| 'private-use' // characters from Unicode private-use areas.
	| 'unassigned' // code points with no assigned Unicode character.
	| 'surrogate' // malformed standalone UTF-16 surrogate code units.

export interface NonStandardTextIssue {
	kind: NonStandardTextIssueKind
	character: string
	codePoint: string
	index: number
}

export interface NonStandardTextResult {
	valid: boolean
	issues: NonStandardTextIssue[]
	counts: Record<NonStandardTextIssueKind, number>
}

export interface NonStandardTextOptions {
	allowNewlines?: boolean
	allowTabs?: boolean
	maxCombiningMarksPerCharacter?: number
}

export const DEFAULT_MAX_COMBINING_MARKS_PER_CHARACTER = 2

export function getNonStandardTextRatio(text: string, result: NonStandardTextResult): number {
	const characterCount = Array.from(text).length
	if (characterCount === 0) return 0

	const nonStandardCharacterCount = new Set(result.issues.map(({ index }) => index)).size
	return nonStandardCharacterCount / characterCount
}

const FANCY_RANGES: ReadonlyArray<readonly [number, number]> = [
	[0x02b0, 0x02ff],
	[0x1d400, 0x1d7ff],
	[0x2460, 0x24ff],
	[0x2070, 0x209f],
	[0x2100, 0x214f],
	[0xfb00, 0xfb06],
	[0xff10, 0xff19],
	[0xff21, 0xff3a],
	[0xff41, 0xff5a],
	[0x1f100, 0x1f1ad],
]

const MARK_PATTERN = /\p{M}/u
const CONTROL_PATTERN = /\p{Cc}/u
const FORMAT_PATTERN = /\p{Cf}/u
const PRIVATE_USE_PATTERN = /\p{Co}/u
const UNASSIGNED_PATTERN = /\p{Cn}/u
const LETTER_PATTERN = /\p{L}/u
const EXTENDED_PICTOGRAPHIC_PATTERN = /\p{Extended_Pictographic}/u
const EMOJI_PRESENTATION_PATTERN = /\p{Emoji_Presentation}/u
const UNIFIED_IDEOGRAPH_PATTERN = /\p{Unified_Ideograph}/u

function createCounts(): Record<NonStandardTextIssueKind, number> {
	return {
		fancy: 0,
		zalgo: 0,
		invisible: 0,
		control: 0,
		'private-use': 0,
		unassigned: 0,
		surrogate: 0,
	}
}

function isInRanges(codePoint: number, ranges: ReadonlyArray<readonly [number, number]>) {
	return ranges.some(([start, end]) => codePoint >= start && codePoint <= end)
}

function isVariationSelector(codePoint: number) {
	return (
		(codePoint >= 0xfe00 && codePoint <= 0xfe0f) || (codePoint >= 0xe0100 && codePoint <= 0xe01ef)
	)
}

function isEmojiModifier(codePoint: number) {
	return codePoint >= 0x1f3fb && codePoint <= 0x1f3ff
}

function isEmojiTag(codePoint: number) {
	return codePoint >= 0xe0020 && codePoint <= 0xe007f
}

function isAscii(character: string) {
	return character.codePointAt(0)! <= 0x7f
}

function isAllowedZeroWidthNonJoiner(
	characters: readonly string[],
	characterIndex: number,
): boolean {
	const previous = characters[characterIndex - 1]
	const next = characters[characterIndex + 1]
	if (!previous || !next || !LETTER_PATTERN.test(previous) || !LETTER_PATTERN.test(next)) {
		return false
	}
	return !isAscii(previous) || !isAscii(next)
}

function findAdjacentEmojiCharacter(
	characters: readonly string[],
	start: number,
	direction: -1 | 1,
): string | undefined {
	for (let index = start; index >= 0 && index < characters.length; index += direction) {
		const character = characters[index]
		const codePoint = character.codePointAt(0)!
		if (isVariationSelector(codePoint) || isEmojiModifier(codePoint)) continue
		return character
	}
	return undefined
}

function isAllowedZeroWidthJoiner(characters: readonly string[], characterIndex: number): boolean {
	const previous = findAdjacentEmojiCharacter(characters, characterIndex - 1, -1)
	const next = findAdjacentEmojiCharacter(characters, characterIndex + 1, 1)
	return (
		previous !== undefined &&
		next !== undefined &&
		EXTENDED_PICTOGRAPHIC_PATTERN.test(previous) &&
		EXTENDED_PICTOGRAPHIC_PATTERN.test(next)
	)
}

function isAllowedVariationSelector(
	characters: readonly string[],
	characterIndex: number,
	codePoint: number,
): boolean {
	const previous = characters[characterIndex - 1]
	if (!previous) return false
	if (codePoint >= 0xfe00 && codePoint <= 0xfe0f) {
		return EXTENDED_PICTOGRAPHIC_PATTERN.test(previous) || /^[0-9#*]$/u.test(previous)
	}
	return UNIFIED_IDEOGRAPH_PATTERN.test(previous)
}

function isAllowedEmojiTagSequence(characters: readonly string[], characterIndex: number) {
	let start = characterIndex - 1
	while (start >= 0 && isEmojiTag(characters[start].codePointAt(0)!)) start--
	if (characters[start]?.codePointAt(0) !== 0x1f3f4) return false

	let end = characterIndex
	while (end < characters.length && isEmojiTag(characters[end].codePointAt(0)!)) end++
	return characters[end - 1]?.codePointAt(0) === 0xe007f
}

function isPresentedAsEmoji(characters: readonly string[], characterIndex: number) {
	const character = characters[characterIndex]
	return (
		EMOJI_PRESENTATION_PATTERN.test(character) ||
		characters[characterIndex + 1]?.codePointAt(0) === 0xfe0f
	)
}

function codePointLabel(codePoint: number) {
	return `U+${codePoint.toString(16).toUpperCase().padStart(4, '0')}`
}

export function validateNonStandardText(
	text: string,
	options: NonStandardTextOptions = {},
): NonStandardTextResult {
	const allowNewlines = options.allowNewlines ?? true
	const allowTabs = options.allowTabs ?? true
	const maxCombiningMarks =
		options.maxCombiningMarksPerCharacter ?? DEFAULT_MAX_COMBINING_MARKS_PER_CHARACTER
	if (!Number.isInteger(maxCombiningMarks) || maxCombiningMarks < 0) {
		throw new Error('Maximum combining marks must be a non-negative integer')
	}

	const issues: NonStandardTextIssue[] = []
	const counts = createCounts()
	const characters = Array.from(text)
	let utf16Index = 0
	let hasBaseCharacter = false
	let combiningMarkCount = 0

	function addIssue(
		kind: NonStandardTextIssueKind,
		character: string,
		codePoint: number,
		index: number,
	) {
		issues.push({
			kind,
			character,
			codePoint: codePointLabel(codePoint),
			index,
		})
		counts[kind]++
	}

	for (let characterIndex = 0; characterIndex < characters.length; characterIndex++) {
		const character = characters[characterIndex]
		const codePoint = character.codePointAt(0)!
		const currentIndex = utf16Index
		utf16Index += character.length

		if (codePoint >= 0xd800 && codePoint <= 0xdfff) {
			addIssue('surrogate', character, codePoint, currentIndex)
			hasBaseCharacter = false
			combiningMarkCount = 0
			continue
		}

		if (PRIVATE_USE_PATTERN.test(character)) {
			addIssue('private-use', character, codePoint, currentIndex)
		} else if (UNASSIGNED_PATTERN.test(character)) {
			addIssue('unassigned', character, codePoint, currentIndex)
		}

		if (CONTROL_PATTERN.test(character)) {
			const allowedNewline = allowNewlines && (character === '\n' || character === '\r')
			const allowedTab = allowTabs && character === '\t'
			if (!allowedNewline && !allowedTab) {
				addIssue('control', character, codePoint, currentIndex)
			}
			hasBaseCharacter = false
			combiningMarkCount = 0
			continue
		}

		if (FORMAT_PATTERN.test(character)) {
			const allowed =
				codePoint === 0x200b ||
				(codePoint === 0x200c && isAllowedZeroWidthNonJoiner(characters, characterIndex)) ||
				(codePoint === 0x200d && isAllowedZeroWidthJoiner(characters, characterIndex)) ||
				(isEmojiTag(codePoint) && isAllowedEmojiTagSequence(characters, characterIndex))
			if (!allowed) addIssue('invisible', character, codePoint, currentIndex)
			hasBaseCharacter = false
			combiningMarkCount = 0
			continue
		}

		if (MARK_PATTERN.test(character)) {
			if (isVariationSelector(codePoint)) {
				if (!isAllowedVariationSelector(characters, characterIndex, codePoint)) {
					addIssue('invisible', character, codePoint, currentIndex)
				}
				continue
			}

			combiningMarkCount++
			if (!hasBaseCharacter || combiningMarkCount > maxCombiningMarks) {
				addIssue('zalgo', character, codePoint, currentIndex)
			}
			continue
		}

		combiningMarkCount = 0
		hasBaseCharacter = !/^\s$/u.test(character)

		if (
			codePoint !== 0x2122 &&
			isInRanges(codePoint, FANCY_RANGES) &&
			!isPresentedAsEmoji(characters, characterIndex)
		) {
			addIssue('fancy', character, codePoint, currentIndex)
		}
	}

	return {
		valid: issues.length === 0,
		issues,
		counts,
	}
}
