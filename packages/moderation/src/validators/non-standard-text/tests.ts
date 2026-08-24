import assert from 'node:assert/strict'
import test from 'node:test'

import { validateNonStandardText } from './index.ts'

test('accepts ordinary multilingual text and punctuation', () => {
	const result = validateNonStandardText(
		'Hello, “world”! Français — Русский — العربية — 中文 — 日本語',
	)

	assert.equal(result.valid, true)
	assert.deepEqual(result.issues, [])
})

test('accepts composed and normally decomposed accents', () => {
	assert.equal(validateNonStandardText('café').valid, true)
	assert.equal(validateNonStandardText('cafe\u0301').valid, true)
	assert.equal(validateNonStandardText('a\u0301\u0327').valid, true)
})

test('detects excessive and leading combining marks as zalgo text', () => {
	const excessive = validateNonStandardText('a\u0301\u0327\u0308')
	const leading = validateNonStandardText('\u0301text')

	assert.equal(excessive.valid, false)
	assert.equal(excessive.counts.zalgo, 1)
	assert.equal(excessive.issues[0].index, 3)
	assert.equal(leading.counts.zalgo, 1)
})

test('supports a custom combining-mark threshold', () => {
	assert.equal(
		validateNonStandardText('a\u0301\u0327', {
			maxCombiningMarksPerCharacter: 1,
		}).counts.zalgo,
		1,
	)
	assert.throws(
		() => validateNonStandardText('text', { maxCombiningMarksPerCharacter: -1 }),
		/non-negative integer/,
	)
})

test('detects common fancy alphabets and presentation forms', () => {
	const result = validateNonStandardText('𝐇 Ⓗ ʰ ℌ ｈ ﬀ')

	assert.equal(result.valid, false)
	assert.equal(result.counts.fancy, 6)
	assert.deepEqual(
		result.issues.map(({ codePoint }) => codePoint),
		['U+1D407', 'U+24BD', 'U+02B0', 'U+210C', 'U+FF48', 'U+FB00'],
	)
})

test('allows ordinary emoji and valid emoji joiner sequences', () => {
	assert.equal(validateNonStandardText('Hello 👋🏽').valid, true)
	assert.equal(validateNonStandardText('Family: 👨‍👩‍👧‍👦').valid, true)
	assert.equal(validateNonStandardText('Developer: 🧑🏽‍💻').valid, true)
	assert.equal(validateNonStandardText('Heart: ❤️').valid, true)
	assert.equal(validateNonStandardText('Information: ℹ️').valid, true)
	assert.equal(validateNonStandardText('A button: 🅰️').valid, true)
	assert.equal(validateNonStandardText('Scotland: 🏴󠁧󠁢󠁳󠁣󠁴󠁿').valid, true)
})

test('detects suspicious invisible and directional characters', () => {
	const result = validateNonStandardText('ab\u200Bcd\u202Eef\u2060gh f\uFE0F')

	assert.equal(result.counts.invisible, 4)
	assert.deepEqual(
		result.issues.map(({ codePoint }) => codePoint),
		['U+200B', 'U+202E', 'U+2060', 'U+FE0F'],
	)
})

test('allows contextual non-joiners but catches ASCII separator evasion', () => {
	assert.equal(validateNonStandardText('می‌خواهم').valid, true)
	assert.equal(validateNonStandardText('f‌uck').counts.invisible, 1)
	assert.equal(validateNonStandardText('a‍b').counts.invisible, 1)
})

test('allows newlines and tabs by default and can reject them', () => {
	assert.equal(validateNonStandardText('line one\n\tline two').valid, true)

	const result = validateNonStandardText('line one\n\tline two', {
		allowNewlines: false,
		allowTabs: false,
	})
	assert.equal(result.counts.control, 2)
})

test('detects other disallowed control characters', () => {
	const result = validateNonStandardText(`hello\u0000world`)

	assert.equal(result.counts.control, 1)
	assert.equal(result.issues[0].codePoint, 'U+0000')
})

test('detects private-use, unassigned, and lone surrogate code points', () => {
	const privateUse = validateNonStandardText('\uE000')
	const unassigned = validateNonStandardText('\uFDD0')
	const surrogate = validateNonStandardText('\uD800')

	assert.equal(privateUse.counts['private-use'], 1)
	assert.equal(unassigned.counts.unassigned, 1)
	assert.equal(surrogate.counts.surrogate, 1)
})

test('reports UTF-16 indexes consistently around astral characters', () => {
	const result = validateNonStandardText('🙂\u200Btext')

	assert.equal(result.issues[0].index, 2)
})

test('reports multiple issue categories in source order', () => {
	const result = validateNonStandardText('𝐀\u200B\u0000')

	assert.deepEqual(
		result.issues.map(({ kind }) => kind),
		['fancy', 'invisible', 'control'],
	)
})
