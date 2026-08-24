import assert from 'node:assert/strict'
import test from 'node:test'

import { createProfanityValidator, sanitizeProfanityText, validateProfanity } from './index.ts'

test('sanitizes text with the configured single-character replacements', () => {
	assert.equal(sanitizeProfanityText('4@3105789+$([{!|£€¥¢<'), 'aaeiostbgtsccciieeycc')
})

test('sanitizes paired characters before their single-character replacements', () => {
	assert.equal(sanitizeProfanityText('()[]{}<>'), 'oooo')
})

test('strips accents, separators, emoji, and ASCII casing', () => {
	assert.equal(sanitizeProfanityText('F Ü.C—K🙂'), 'fuck')
})

test('matches profanity across separators and common substitutions', () => {
	assert.equal(validateProfanity('f.u c-k').firstMatch?.term, 'fuck')
	assert.equal(validateProfanity('$h!t').firstMatch?.term, 'shit')
	assert.equal(validateProfanity('p()rn').firstMatch?.term, 'porn')
})

test('honors exact negative prefix and suffix matches', () => {
	const validator = createProfanityValidator({
		patterns: {
			bad: { kind: 'profanity', exceptions: ['notbadword'] },
		},
	})

	assert.equal(validator.findFirst('not bad word'), undefined)
	assert.equal(validator.findFirst('very bad word')?.term, 'bad')
	assert.equal(validator.findFirst('not bad phrase')?.term, 'bad')
})

test('matches the first profanity while ignoring a later negative match', () => {
	const validator = createProfanityValidator({
		patterns: {
			shit: { kind: 'profanity', exceptions: ['horseshit', 'bullshit'] },
			fuck: { kind: 'profanity', exceptions: [] },
		},
	})

	assert.equal(validator.findFirst('this horseshit'), undefined)
	assert.equal(validator.findFirst('fuck this bullshit')?.term, 'fuck')
})

test('uses the first terminal when one configured term prefixes another', () => {
	const validator = createProfanityValidator({
		patterns: {
			bad: { kind: 'profanity', exceptions: [] },
			badword: { kind: 'profanity', exceptions: [] },
		},
	})

	assert.equal(validator.findFirst('badword')?.term, 'bad')
})

test('does not match configured false-positive substrings', () => {
	assert.equal(validateProfanity('Scunthorpe and peacock').valid, true)
	assert.equal(validateProfanity('cock and cunt').profanityCount, 2)
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

test('rejects slurs', () => {
	const validator = createProfanityValidator({
		patterns: {
			forbidden: { kind: 'slur', exceptions: [] },
		},
	})

	assert.equal(validator.validate('forbidden').valid, false)
	assert.equal(validator.validate('forbidden').slurCount, 1)
})

test('counts matches from left to right without overlaps', () => {
	const validator = createProfanityValidator({
		patterns: {
			bad: { kind: 'profanity', exceptions: [] },
		},
	})

	assert.deepEqual(
		validator
			.findAll('bad-bad')
			.map(({ sanitizedStart, sanitizedEnd }) => [sanitizedStart, sanitizedEnd]),
		[
			[0, 3],
			[3, 6],
		],
	)
})

test('rejects invalid configuration', () => {
	assert.throws(
		() =>
			createProfanityValidator({
				patterns: {
					'not sanitized': { kind: 'profanity', exceptions: [] },
				},
			}),
		/term must already be sanitized/,
	)
	assert.throws(
		() =>
			createProfanityValidator({
				patterns: {
					bad: { kind: 'profanity', exceptions: ['innocent'] },
				},
			}),
		/exception must contain bad/,
	)
})
