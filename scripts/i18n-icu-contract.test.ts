import assert from 'node:assert/strict'
import { test } from 'node:test'

import { contractFromMessage, contractsEqual, sourceContractChanged } from './i18n-icu-contract'

test('same plain text contract is equal', () => {
	assert.equal(
		contractsEqual(contractFromMessage('Hello', 'a'), contractFromMessage('Goodbye', 'b')),
		true,
	)
})

test('variable rename changes contract', () => {
	assert.equal(
		contractsEqual(
			contractFromMessage('Hello {name}', 'a'),
			contractFromMessage('Hello {username}', 'b'),
		),
		false,
	)
})

test('variable removal changes contract', () => {
	assert.equal(
		contractsEqual(contractFromMessage('Created by {user}', 'a'), contractFromMessage('Created', 'b')),
		false,
	)
})

test('rich text tag rename changes contract', () => {
	assert.equal(
		contractsEqual(
			contractFromMessage('Read <link>docs</link>', 'a'),
			contractFromMessage('Read <docs-link>docs</docs-link>', 'b'),
		),
		false,
	)
})

test('literal html-like tags are treated as plain text', () => {
	assert.equal(
		contractsEqual(
			contractFromMessage('Line one<br><br>Line two', 'a'),
			contractFromMessage('Zeile eins<br><br>Zeile zwei', 'b'),
		),
		true,
	)
})

test('select branch changes contract', () => {
	assert.equal(
		contractsEqual(
			contractFromMessage('{type, select, mod {mod} other {project}}', 'a'),
			contractFromMessage('{type, select, plugin {plugin} other {project}}', 'b'),
		),
		false,
	)
})

test('invalid previous source message is treated as changed', () => {
	assert.equal(
		sourceContractChanged(
			'Get support at {support-link}',
			'Get support at <support-link></support-link>',
			'previous',
			'current',
		),
		true,
	)
})

test('invalid current source message is rejected', () => {
	assert.throws(() =>
		sourceContractChanged(
			'Get support at <support-link></support-link>',
			'Get support at {support-link}',
			'previous',
			'current',
		),
	)
})
