import assert from 'node:assert/strict'
import test from 'node:test'

import { validateSpam } from './index.ts'

test('allows ordinary prose', () => {
	assert.equal(
		validateSpam('This project adds a configurable compass for exploring large worlds.').valid,
		true,
	)
})

test('detects repeated characters', () => {
	assert.deepEqual(validateSpam('aaaaaaaa').firstMatch, {
		kind: 'character',
		value: 'aaaaaaaa',
	})
})

test('detects repeated words case-insensitively', () => {
	assert.deepEqual(validateSpam('Great great GREAT great').firstMatch, {
		kind: 'word',
		value: 'great',
	})
})

test('detects repeated phrases across punctuation', () => {
	assert.deepEqual(validateSpam('best project, best project! BEST PROJECT').firstMatch, {
		kind: 'phrase',
		value: 'best project',
	})
})

test('allows repetition below the spam thresholds', () => {
	assert.equal(validateSpam('so so so good good phrase here phrase here').valid, true)
})
