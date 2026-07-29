import assert from 'node:assert/strict'
import { test } from 'node:test'

import {
	contractFromMessage,
	contractsEqual,
	sourceContractChanged,
	translationCompatibleWithSource,
} from './i18n-icu-contract'

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

test('translation can move phrase inside ICU branches', () => {
	const source = contractFromMessage(
		'In the last {amount} {unit, select, hours {{amount, plural, one {hour} other {hours}}} days {{amount, plural, one {day} other {days}}} other {days}}',
		'source',
	)
	const translation = contractFromMessage(
		"{unit, select, hours {{amount, plural, one {Nell'ultima ora} other {Nelle ultime # ore}}} days {{amount, plural, one {Nell'ultimo giorno} other {Negli ultimi # giorni}}} other {Negli ultimi {amount} giorni}}",
		'translation',
	)

	assert.equal(translationCompatibleWithSource(source, translation), true)
})

test('translation can use locale-specific plural categories', () => {
	const source = contractFromMessage('{count, plural, one {# file} other {# files}}', 'source')
	const translation = contractFromMessage(
		'{count, plural, one {# файл} few {# файла} many {# файлов} other {# файла}}',
		'translation',
	)

	assert.equal(translationCompatibleWithSource(source, translation), true)
})

test('translation can simplify plural when wording does not vary', () => {
	const source = contractFromMessage('{count, plural, one {# server} other {# servers}}', 'source')
	const translation = contractFromMessage('{count} Server', 'translation')

	assert.equal(translationCompatibleWithSource(source, translation), true)
})

test('translation cannot invent app select values', () => {
	const source = contractFromMessage(
		'{unit, select, hours {hours} days {days} other {days}}',
		'source',
	)
	const translation = contractFromMessage(
		'{unit, select, years {years} other {days}}',
		'translation',
	)

	assert.equal(translationCompatibleWithSource(source, translation), false)
})

test('translation cannot invent variables', () => {
	const source = contractFromMessage('Created by {user}', 'source')
	const translation = contractFromMessage('Created by {username}', 'translation')

	assert.equal(translationCompatibleWithSource(source, translation), false)
})

test('source shape rewrite with same runtime interface is not a contract change', () => {
	assert.equal(
		sourceContractChanged(
			'In the last {amount} {unit, select, hours {{amount, plural, one {hour} other {hours}}} days {{amount, plural, one {day} other {days}}} other {days}}',
			'{unit, select, hours {{amount, plural, one {In the last hour} other {In the last # hours}}} days {{amount, plural, one {In the last day} other {In the last # days}}} other {In the last {amount} days}}',
			'previous',
			'current',
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
