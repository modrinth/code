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

export const DEFAULT_PROFANITY_CONFIG: ProfanityConfig = {
	patterns: DEFAULT_PROFANITY_PATTERNS,
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

export function createProfanityValidator(
	config: ProfanityConfig = DEFAULT_PROFANITY_CONFIG,
): ProfanityValidator {
	const entries = Object.entries(config.patterns).map(([rawTerm, pattern]) => {
		const term = rawTerm.toLowerCase()
		if (!term || !/^[a-z]+$/.test(term)) {
			throw new Error(`Profanity term must contain only ASCII letters: ${rawTerm}`)
		}

		return { kind: pattern.kind, term }
	})
	const matcher = new RegExpMatcher({
		blacklistedTerms: entries.map(({ term }, id) => ({ id, pattern: parseRawPattern(term) })),
		blacklistMatcherTransformers: [
			resolveConfusablesTransformer(),
			resolveLeetSpeakTransformer(),
			toAsciiLowerCaseTransformer(),
			skipNonAlphabeticTransformer(),
			collapseDuplicatesTransformer({
				customThresholds: getDuplicateThresholds(entries.map(({ term }) => term)),
			}),
		],
	})

	function findAll(text: string): ProfanityMatch[] {
		const matches: ProfanityMatch[] = []

		for (const match of matcher.getAllMatches(text, true)) {
			const profanityPattern = entries[match.termId]
			if (!profanityPattern || match.startIndex < (matches.at(-1)?.end ?? 0)) continue

			matches.push({
				...profanityPattern,
				rawText: text.slice(match.startIndex, match.endIndex + 1),
				start: match.startIndex,
				end: match.endIndex + 1,
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
