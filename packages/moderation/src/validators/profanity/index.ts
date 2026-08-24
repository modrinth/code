export type ProfanityKind = 'profanity' | 'slur'

export interface ProfanityPattern {
	kind: ProfanityKind
	exceptions: readonly string[]
}

export interface ProfanityConfig {
	patterns: Readonly<Record<string, ProfanityPattern>>
}

export interface ProfanityMatch {
	kind: ProfanityKind
	term: string
	sanitizedStart: number
	sanitizedEnd: number
}

export interface ProfanityResult {
	valid: boolean
	profanityCount: number
	slurCount: number
	firstMatch?: ProfanityMatch
	matches: ProfanityMatch[]
}

export interface ProfanityValidator {
	sanitize(text: string): string
	findFirst(text: string): ProfanityMatch | undefined
	findAll(text: string): ProfanityMatch[]
	validate(text: string): ProfanityResult
}

interface NegativeMatch {
	prefix: string
	suffix: string
}

interface TrieNode {
	children: Map<string, TrieNode>
	negatives: NegativeMatch[]
	terminal?: {
		kind: ProfanityKind
		term: string
	}
}

const CHARACTER_REPLACEMENTS: Readonly<Record<string, string>> = {
	'4': 'a',
	'@': 'a',
	'3': 'e',
	'1': 'i',
	'0': 'o',
	'5': 's',
	'7': 't',
	'8': 'b',
	'9': 'g',
	'+': 't',
	$: 's',
	'(': 'c',
	'{': 'c',
	'[': 'c',
	'!': 'i',
	'|': 'i',
	'£': 'e',
	'€': 'e',
	'¥': 'y',
	'¢': 'c',
	'<': 'c',
}

const MULTI_CHARACTER_REPLACEMENTS: Readonly<Record<string, Readonly<Record<string, string>>>> = {
	'(': { ')': 'o' },
	'[': { ']': 'o' },
	'{': { '}': 'o' },
	'<': { '>': 'o' },
}

const TERM_EXCEPTIONS: Readonly<Record<string, readonly string[]>> = {
	anuslick: [],
	arsehol: [],
	arselick: [],
	asslick: [],
	arsch: [],
	asshol: [],
	auschwitz: [],
	beaner: [],
	bestiality: [],
	baise: [],
	bakachon: [],
	bakatyon: [],
	bastard: ['bastardized'],
	bitch: [],
	btch: [],
	biatch: [],
	bussy: [],
	blowjob: [],
	blowme: [],
	bukakke: [],
	buttplug: [],
	buttchug: [],
	butagorosi: [],
	cagada: [],
	caralho: [],
	cameljockey: [],
	castrate: [],
	cazzo: [],
	ceemen: [],
	chankoro: [],
	chink: [],
	chingchong: [],
	choad: [],
	chode: [],
	chlamydia: [],
	clit: ['clitheroe'],
	clitoris: [],
	cock: [
		'cockade',
		'cockatiel',
		'cockatiels',
		'cockatoo',
		'cockatoos',
		'cockatrice',
		'cockayne',
		'cockburn',
		'cockcroft',
		'cocked',
		'cocker',
		'cockerel',
		'cockers',
		'cockeyed',
		'cockiness',
		'cocking',
		'cocklebur',
		'cockney',
		'cockpit',
		'cockpits',
		'cockroach',
		'cockroaches',
		'cockscomb',
		'cockspur',
		'cocktail',
		'gamecock',
		'hancock',
		'haycock',
		'hitchcock',
		'leacock',
		'peacock',
		'shuttlecock',
		'stopcock',
		'woodcock',
	],
	coon: ['cocoon', 'laocoon', 'raccoon', 'racoon', 'tycoon'],
	cocain: [],
	coitus: [],
	cottonpic: [],
	cottonpik: [],
	cum: [
		'acumen',
		'acuminate',
		'altocumulus',
		'cumber',
		'cumbing',
		'cumbria',
		'cumbrian',
		'cumbrous',
		'cummerbund',
		'cumming',
		'cumulat',
		'cumuli',
		'cumulonimbus',
		'cumulus',
		'encumber',
		'encumbrance',
		'scumbag',
		'locum',
		'modicum',
		'magnacumlaude',
		'macumba',
		'practicum',
		'recumbent',
		'slocum',
		'stratocumulus',
		'succumb',
		'talcum',
		'taraxacum',
		'tecumseh',
		'tucuman',
		'capsicum',
		'cecum',
		'circum',
		'colchicum',
		'document',
		'ecumeni',
		'illyricum',
		'incumben',
	],
	cunt: ['scunthorpe'],
	cvnt: [],
	cunny: [],
	cunnie: [],
	csam: [],
	cyka: [],
	darkie: [],
	dick: [
		'chappaquiddick',
		'dickens',
		'dickensian',
		'dickerson',
		'dickey',
		'dickies',
		'dickinson',
		'dickson',
		'dickvandyke',
		'dicky',
	],
	dildo: [],
	douchebag: [],
	dyke: ['vandyke'],
	downie: [],
	dumbass: [],
	ejaculate: [],
	fag: ['antofagasta', 'serfage', 'wharfage', 'fagin', 'leafage'],
	feck: [],
	fellate: [],
	fellatio: [],
	felch: [],
	fuck: [],
	fvck: [],
	fxck: [],
	fack: [],
	fzck: [],
	fck: [],
	fudgepacker: [],
	flange: ['flanged', 'flanges'],
	gestapo: [],
	gook: [],
	horny: ['thorny'],
	hooker: [],
	hitler: [],
	incest: [],
	jap: ['japan'],
	jizz: [],
	jigabo: [],
	junglebunny: [],
	kkk: [],
	kike: [],
	klux: [],
	kluklux: [],
	klukluxklan: [],
	koon: [],
	lickmy: [],
	masturbat: [],
	molest: [],
	muff: [
		'muffed',
		'muffin',
		'muffins',
		'muffle',
		'muffled',
		'muffler',
		'mufflers',
		'muffles',
		'muffling',
		'muffs',
		'ragamuffin',
		'earmuff',
		'earmuffs',
	],
	nazi: ['ashkenazi', 'ashkenazic', 'ashkenazim', 'monazite'],
	nigg: [],
	niqa: [],
	nigga: [],
	niqqa: [],
	niggu: [],
	niqqu: [],
	niggr: [],
	nigger: [],
	niglet: [],
	nignog: [],
	paki: ['pakistan'],
	penis: ['openis', 'penistone'],
	porn: [],
	prostitut: [],
	pube: [],
	pussie: [],
	pussy: ['pussycat', 'pussyfoot'],
	raghead: [],
	rape: [
		'grape',
		'forape',
		'trapeze',
		'trapezium',
		'trapezius',
		'trapezoid',
		'therapeutic',
		'drape',
		'parapet',
		'rapeseed',
		'scrape',
		'serape',
	],
	rapist: ['therapist'],
	retard: ['retardant', 'retarder', 'retarding'],
	rimjob: [],
	shit: [
		'cushitic',
		'shitake',
		'pushit',
		'peshitta',
		'libshitz',
		'shitzu',
		'wishit',
		'yamashita',
		'finishit',
		'shitbox',
		'shitmg',
		'publishit',
		'englishit',
	],
	slut: [],
	spunk: ['spunky'],
	suckmy: [],
	sodom: [],
	semen: ['sement'],
	teensex: [],
	tittie: [],
	titty: [],
	trannie: [],
	tranny: [],
	vagina: [],
	wank: ['swank', 'wankel'],
	wetback: [],
	whore: ['whores', 'whorev', 'whoreturned'],
	whitepower: [],
	fondle: [],
	minestorm: [],
	kissmy: [],
	blowmy: [],
	jelqing: [],
	dafuq: [],
}

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
		Object.entries(TERM_EXCEPTIONS).map(([term, exceptions]) => {
			const kind: ProfanityKind = SLUR_TERMS.has(term) ? 'slur' : 'profanity'
			return [term, { kind, exceptions }] as const
		}),
	)

export const DEFAULT_PROFANITY_CONFIG: ProfanityConfig = {
	patterns: DEFAULT_PROFANITY_PATTERNS,
}

function newTrieNode(): TrieNode {
	return {
		children: new Map(),
		negatives: [],
	}
}

export function sanitizeProfanityText(text: string): string {
	const transformed = text
		.normalize('NFD')
		.replaceAll(/\p{Mn}/gu, '')
		.normalize('NFC')
	const characters = Array.from(transformed)
	let sanitized = ''

	for (let index = 0; index < characters.length; index++) {
		const character = characters[index]
		const multiReplacement = MULTI_CHARACTER_REPLACEMENTS[character]
		const nextCharacter = characters[index + 1]

		if (multiReplacement && nextCharacter && multiReplacement[nextCharacter]) {
			sanitized += multiReplacement[nextCharacter]
			index++
			continue
		}

		const replacement = CHARACTER_REPLACEMENTS[character]
		if (replacement) {
			sanitized += replacement
		} else if (character >= 'A' && character <= 'Z') {
			sanitized += character.toLowerCase()
		} else if ((character >= 'a' && character <= 'z') || (character >= '0' && character <= '9')) {
			sanitized += character
		}
	}

	return sanitized
}

function createTrie(patterns: Readonly<Record<string, ProfanityPattern>>): TrieNode {
	const root = newTrieNode()

	for (const [rawTerm, pattern] of Object.entries(patterns)) {
		const term = rawTerm.toLowerCase()
		if (!term || sanitizeProfanityText(term) !== term) {
			throw new Error(`Profanity term must already be sanitized: ${rawTerm}`)
		}

		let current = root
		for (const character of term) {
			let child = current.children.get(character)
			if (!child) {
				child = newTrieNode()
				current.children.set(character, child)
			}
			current = child
		}

		if (current.terminal) {
			throw new Error(`Duplicate sanitized profanity term: ${term}`)
		}
		current.terminal = { kind: pattern.kind, term }

		for (const rawException of pattern.exceptions) {
			const exception = rawException.toLowerCase()
			if (sanitizeProfanityText(exception) !== exception) {
				throw new Error(`Profanity exception must already be sanitized: ${rawException}`)
			}

			const termIndex = exception.indexOf(term)
			if (termIndex < 0) {
				throw new Error(`Profanity exception must contain ${term}: ${rawException}`)
			}

			current.negatives.push({
				prefix: exception.slice(0, termIndex),
				suffix: exception.slice(termIndex + term.length),
			})
		}
	}

	return root
}

function negativeMatches(negative: NegativeMatch, text: string, start: number, end: number) {
	const prefixIndex = start - negative.prefix.length
	const suffixIndex = end + negative.suffix.length

	if (prefixIndex < 0 || suffixIndex > text.length) return false
	return (
		text.slice(prefixIndex, start) === negative.prefix &&
		text.slice(end, suffixIndex) === negative.suffix
	)
}

function findAt(root: TrieNode, text: string, start: number): ProfanityMatch | undefined {
	let current = root

	for (let index = start; index < text.length; index++) {
		const child = current.children.get(text[index])
		if (!child) return undefined
		current = child

		if (!current.terminal) continue

		const end = index + 1
		const matchesNegative = current.negatives.some((negative) =>
			negativeMatches(negative, text, start, end),
		)
		if (matchesNegative) continue

		return {
			kind: current.terminal.kind,
			term: text.slice(start, end),
			sanitizedStart: start,
			sanitizedEnd: end,
		}
	}

	return undefined
}

export function createProfanityValidator(
	config: ProfanityConfig = DEFAULT_PROFANITY_CONFIG,
): ProfanityValidator {
	const root = createTrie(config.patterns)

	function findFirst(text: string): ProfanityMatch | undefined {
		const sanitized = sanitizeProfanityText(text)
		for (let index = 0; index < sanitized.length; index++) {
			const match = findAt(root, sanitized, index)
			if (match) return match
		}
		return undefined
	}

	function findAll(text: string): ProfanityMatch[] {
		const sanitized = sanitizeProfanityText(text)
		const matches: ProfanityMatch[] = []

		for (let index = 0; index < sanitized.length; ) {
			const match = findAt(root, sanitized, index)
			if (match) {
				matches.push(match)
				index = match.sanitizedEnd
			} else {
				index++
			}
		}

		return matches
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
		sanitize: sanitizeProfanityText,
		findFirst,
		findAll,
		validate,
	}
}

export const profanityValidator = createProfanityValidator()

export function validateProfanity(text: string): ProfanityResult {
	return profanityValidator.validate(text)
}
