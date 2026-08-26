import assert from 'node:assert/strict'
import test from 'node:test'

import { createProfanityValidator, validateProfanity } from './index.ts'

const blockedForms = [
	['normal form', 'fuck'],
	['capitalization', 'FUCK'],
	['Unicode variants', 'ｆｕｃｋ'],
	['zero-width characters', 'f\u200Buck'],
	['leetspeak', '$h!t'],
	['period separators', 'f.u.c.k'],
	['space separators', 'f u c k'],
	['repeated letters', 'fuuuuuck'],
] as const

for (const [form, input] of blockedForms) {
	test(`blocks ${form}`, () => {
		const result = validateProfanity(input)

		assert.equal(result.valid, false)
		assert.equal(result.firstMatch?.rawText, input)
	})
}

test('does not use term exceptions', () => {
	const validator = createProfanityValidator({
		patterns: {
			bad: { kind: 'profanity' },
		},
	})

	assert.equal(validator.findFirst('not bad word')?.term, 'bad')
})

test('uses the first match when one configured term prefixes another', () => {
	const validator = createProfanityValidator({
		patterns: {
			bad: { kind: 'profanity' },
			badword: { kind: 'profanity' },
		},
	})

	assert.equal(validator.findFirst('badword')?.term, 'bad')
})

test('rejects any uncensored configured profanity', () => {
	assert.equal(validateProfanity('A clean project').valid, true)
	assert.equal(validateProfanity('This is shit').valid, false)
})

test('allows redacted profanity when the removed letters cannot reconstruct a term', () => {
	assert.equal(validateProfanity('f**k').valid, true)
	assert.equal(validateProfanity('f**k works in titles, summaries, and descriptions').valid, true)
	assert.equal(validateProfanity('f.u.c.k').valid, false)
})

test('classifies slurs separately from other profanity', () => {
	const validator = createProfanityValidator({
		patterns: {
			forbidden: { kind: 'slur' },
		},
	})
	const result = validator.validate('FORBIDDEN')

	assert.equal(result.valid, false)
	assert.equal(result.profanityCount, 0)
	assert.equal(result.slurCount, 1)
})

test('returns non-overlapping matches and original input offsets', () => {
	const validator = createProfanityValidator({
		patterns: {
			bad: { kind: 'profanity' },
		},
	})

	assert.deepEqual(
		validator.findAll('b.a.d bad').map(({ start, end }) => [start, end]),
		[
			[0, 5],
			[6, 9],
		],
	)
})

test('rejects invalid configuration', () => {
	assert.throws(
		() =>
			createProfanityValidator({
				patterns: {
					'not sanitized': { kind: 'profanity' },
				},
			}),
		/term must contain only ASCII letters/,
	)
})
