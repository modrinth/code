import {
	collapseDuplicatesTransformer,
	parseRawPattern,
	RegExpMatcher,
	resolveConfusablesTransformer,
	resolveLeetSpeakTransformer,
	skipNonAlphabeticTransformer,
	toAsciiLowerCaseTransformer,
} from 'obscenity'

export type ProfanityKind = 'profanity' | 'slur'

export interface ProfanityPattern {
	kind: ProfanityKind
}

export interface ProfanityConfig {
	patterns: Readonly<Record<string, ProfanityPattern>>
	allowlist?: readonly string[]
}

export interface ProfanityMatch {
	kind: ProfanityKind
	term: string
	rawText: string
	start: number
	end: number
}

export interface ProfanityResult {
	valid: boolean
	profanityCount: number
	slurCount: number
	firstMatch?: ProfanityMatch
	matches: ProfanityMatch[]
}

export interface ProfanityValidator {
	findFirst(text: string): ProfanityMatch | undefined
	findAll(text: string): ProfanityMatch[]
	validate(text: string): ProfanityResult
}

const TERMS = [
	'anuslick',
	'arsehol',
	'arselick',
	'asslick',
	'arsch',
	'asshol',
	'auschwitz',
	'beaner',
	'bestiality',
	'baise',
	'bakachon',
	'bakatyon',
	'bastard',
	'bitch',
	'btch',
	'biatch',
	'bussy',
	'blowjob',
	'blowme',
	'bukakke',
	'buttplug',
	'buttchug',
	'butagorosi',
	'cagada',
	'caralho',
	'cameljockey',
	'castrate',
	'cazzo',
	'ceemen',
	'chankoro',
	'chink',
	'chingchong',
	'choad',
	'chode',
	'chlamydia',
	'clit',
	'clitoris',
	'cock',
	'coon',
	'cocain',
	'coitus',
	'cottonpic',
	'cottonpik',
	'cum',
	'cunt',
	'cvnt',
	'cunny',
	'cunnie',
	'csam',
	'cyka',
	'darkie',
	'dick',
	'dildo',
	'douchebag',
	'dyke',
	'downie',
	'dumbass',
	'ejaculate',
	'fag',
	'feck',
	'fellate',
	'fellatio',
	'felch',
	'fuck',
	'fvck',
	'fxck',
	'fack',
	'fzck',
	'fck',
	'fudgepacker',
	'flange',
	'gestapo',
	'gook',
	'horny',
	'hooker',
	'hitler',
	'incest',
	'jap',
	'jizz',
	'jigabo',
	'junglebunny',
	'kkk',
	'kike',
	'klux',
	'kluklux',
	'klukluxklan',
	'koon',
	'lickmy',
	'masturbat',
	'molest',
	'muff',
	'nazi',
	'nigger',
	'nigga',
	'niqa',
	'niqqa',
	'niggu',
	'niqqu',
	'niggr',
	'niglet',
	'nignog',
	'paki',
	'penis',
	'porn',
	'prostitut',
	'pube',
	'pussie',
	'pussy',
	'raghead',
	'rape',
	'rapist',
	'retard',
	'rimjob',
	'shit',
	'slut',
	'spunk',
	'suckmy',
	'sodom',
	'semen',
	'teensex',
	'tittie',
	'titty',
	'trannie',
	'tranny',
	'vagina',
	'wank',
	'wetback',
	'whore',
	'whitepower',
	'fondle',
	'minestorm',
	'kissmy',
	'blowmy',
	'jelqing',
	'dafuq',
] as const

const SLUR_TERMS = new Set([
	'beaner',
	'cameljockey',
	'chankoro',
	'chink',
	'chingchong',
	'coon',
	'cottonpic',
	'cottonpik',
	'darkie',
	'downie',
	'dyke',
	'fag',
	'gook',
	'jap',
	'jigabo',
	'junglebunny',
	'kike',
	'koon',
	'nigg',
	'niqa',
	'nigga',
	'niqqa',
	'niggu',
	'niqqu',
	'niggr',
	'nigger',
	'niglet',
	'nignog',
	'paki',
	'raghead',
	'retard',
	'trannie',
	'tranny',
	'wetback',
])

export const DEFAULT_PROFANITY_PATTERNS: Readonly<Record<string, ProfanityPattern>> =
	Object.fromEntries(
		TERMS.map((term) => {
			const kind: ProfanityKind = SLUR_TERMS.has(term) ? 'slur' : 'profanity'
			return [term, { kind }] as const
		}),
	)

export const DEFAULT_PROFANITY_ALLOWLIST = ['Кооп'] as const

export const DEFAULT_PROFANITY_CONFIG: ProfanityConfig = {
	patterns: DEFAULT_PROFANITY_PATTERNS,
	allowlist: DEFAULT_PROFANITY_ALLOWLIST,
}

function getDuplicateThresholds(terms: readonly string[]): Map<string, number> {
	const thresholds = new Map<string, number>()

	for (const term of terms) {
		let runLength = 0
		let previousCharacter = ''

		for (const character of term) {
			runLength = character === previousCharacter ? runLength + 1 : 1
			previousCharacter = character
			thresholds.set(character, Math.max(thresholds.get(character) ?? 1, runLength))
		}
	}

	return thresholds
}

function isWordCharacter(character: string | undefined): boolean {
	return character !== undefined && /^[\p{L}\p{M}\p{N}_]$/u.test(character)
}

const LEET_SPEAK_CHARACTERS = new Set(['@', '(', '|', '!', '/', '$'])

function isObfuscatedWordCharacter(character: string): boolean {
	return isWordCharacter(character) || LEET_SPEAK_CHARACTERS.has(character)
}

function isCharacterByCharacterObfuscation(text: string): boolean {
	const chunkLengths: number[] = []
	let currentChunkLength = 0
	let hasInvisibleSeparator = false
	let hasVisibleSeparator = false

	for (const character of text) {
		if (isObfuscatedWordCharacter(character)) {
			currentChunkLength++
		} else {
			if (/^\p{Cf}$/u.test(character)) {
				hasInvisibleSeparator = true
			} else {
				hasVisibleSeparator = true
			}

			if (currentChunkLength > 0) {
				chunkLengths.push(currentChunkLength)
				currentChunkLength = 0
			}
		}
	}

	if (currentChunkLength > 0) chunkLengths.push(currentChunkLength)

	return (
		chunkLengths.length > 1 &&
		((hasInvisibleSeparator && !hasVisibleSeparator) ||
			chunkLengths.every((length) => length === 1))
	)
}

function getCharacterBefore(text: string, index: number): string | undefined {
	if (index <= 0) return undefined

	const codePoint = text.codePointAt(index - 1)
	if (codePoint === undefined) return undefined
	if (codePoint >= 0xdc00 && codePoint <= 0xdfff && index > 1) {
		return text.slice(index - 2, index)
	}

	return text[index - 1]
}

function getCharacterAt(text: string, index: number): string | undefined {
	const codePoint = text.codePointAt(index)
	return codePoint === undefined ? undefined : String.fromCodePoint(codePoint)
}

function isWholeWordMatch(text: string, start: number, end: number): boolean {
	return (
		!isWordCharacter(getCharacterBefore(text, start)) && !isWordCharacter(getCharacterAt(text, end))
	)
}

export function createProfanityValidator(
	config: ProfanityConfig = DEFAULT_PROFANITY_CONFIG,
): ProfanityValidator {
	const allowlist = new Set(config.allowlist?.map((term) => term.normalize('NFC').toLowerCase()))
	const entries = Object.entries(config.patterns).map(([rawTerm, pattern]) => {
		const term = rawTerm.toLowerCase()
		if (!term || !/^[a-z]+$/.test(term)) {
			throw new Error(`Profanity term must contain only ASCII letters: ${rawTerm}`)
		}

		return { kind: pattern.kind, term }
	})
	const blacklistedTerms = entries.map(({ term }, id) => ({ id, pattern: parseRawPattern(term) }))
	const baseTransformers = [
		resolveConfusablesTransformer(),
		resolveLeetSpeakTransformer(),
		toAsciiLowerCaseTransformer(),
	]
	const duplicateTransformer = () =>
		collapseDuplicatesTransformer({
			customThresholds: getDuplicateThresholds(entries.map(({ term }) => term)),
		})
	const strictMatcher = new RegExpMatcher({
		blacklistedTerms,
		blacklistMatcherTransformers: [...baseTransformers, duplicateTransformer()],
	})
	const separatorMatcher = new RegExpMatcher({
		blacklistedTerms,
		blacklistMatcherTransformers: [
			...baseTransformers,
			skipNonAlphabeticTransformer(),
			duplicateTransformer(),
		],
	})

	function findAll(text: string): ProfanityMatch[] {
		const matches: ProfanityMatch[] = []
		const strictMatches = new Set(
			[...strictMatcher.getAllMatches(text, true)].map(
				(match) => `${match.termId}:${match.startIndex}:${match.endIndex}`,
			),
		)

		for (const match of separatorMatcher.getAllMatches(text, true)) {
			const profanityPattern = entries[match.termId]
			const end = match.endIndex + 1
			const rawText = text.slice(match.startIndex, end)
			const matchKey = `${match.termId}:${match.startIndex}:${match.endIndex}`
			if (
				!profanityPattern ||
				allowlist.has(rawText.normalize('NFC').toLowerCase()) ||
				(!strictMatches.has(matchKey) && !isCharacterByCharacterObfuscation(rawText)) ||
				!isWholeWordMatch(text, match.startIndex, end) ||
				match.startIndex < (matches.at(-1)?.end ?? 0)
			) {
				continue
			}

			matches.push({
				...profanityPattern,
				rawText,
				start: match.startIndex,
				end,
			})
		}

		return matches
	}

	function findFirst(text: string): ProfanityMatch | undefined {
		return findAll(text)[0]
	}

	function validate(text: string): ProfanityResult {
		const matches = findAll(text)
		const profanityCount = matches.filter((match) => match.kind === 'profanity').length
		const slurCount = matches.length - profanityCount

		return {
			valid: matches.length === 0,
			profanityCount,
			slurCount,
			firstMatch: matches[0],
			matches,
		}
	}

	return {
		findFirst,
		findAll,
		validate,
	}
}

export const profanityValidator = createProfanityValidator()

export function validateProfanity(text: string): ProfanityResult {
	return profanityValidator.validate(text)
}
