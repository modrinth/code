import assert from 'node:assert/strict'
import test from 'node:test'

import {
	containsProjectLinkOrIp,
	extractProjectLinks,
	findProjectTitleMetadata,
	projectSummaryMatchesTitle,
	validateProjectDescription,
	validateProjectSummary,
	validateProjectText,
	validateProjectTitle,
} from './index.ts'

const metadata = {
	gameVersions: ['1.21.1'],
	loaders: ['fabric'],
}

test('finds game versions and loaders in project titles', () => {
	assert.deepEqual(findProjectTitleMetadata('Tools for 1.21.1', metadata), {
		kind: 'game-version',
		value: '1.21.1',
	})
	assert.deepEqual(findProjectTitleMetadata('FABRIC Tools', metadata), {
		kind: 'loader',
		value: 'fabric',
	})
	assert.equal(findProjectTitleMetadata('Magical Tools', metadata), null)
	assert.equal(findProjectTitleMetadata('Ordinary Tools', metadata), null)
})

test('compares summaries and titles after trimming and Unicode normalization', () => {
	assert.equal(projectSummaryMatchesTitle('  Caf\u00e9  ', 'Cafe\u0301'), true)
	assert.equal(projectSummaryMatchesTitle('Project summary', 'Project title'), false)
	assert.equal(projectSummaryMatchesTitle('', ''), false)
})

test('detects links and IP addresses but not email addresses or game versions', () => {
	assert.equal(containsProjectLinkOrIp('Visit https://modrinth.com'), true)
	assert.equal(containsProjectLinkOrIp('Visit modrinth.com'), true)
	assert.equal(containsProjectLinkOrIp('Join 127.0.0.1:25565'), true)
	assert.equal(containsProjectLinkOrIp('Supports Minecraft 1.21.1'), false)
	assert.equal(containsProjectLinkOrIp('Contact hello@example.com'), false)
})

test('extracts and deduplicates normalized links', () => {
	assert.deepEqual(
		extractProjectLinks(
			'Visit [Modrinth](https://modrinth.com) and example.net twice: example.net',
		),
		['https://modrinth.com', 'http://example.net'],
	)
})

test('validates shared project text', () => {
	assert.equal(validateProjectText('An ordinary project'), null)
	assert.equal(
		validateProjectText('This project is shit')?.message.id,
		'project.text-validation.profanity',
	)
	assert.equal(
		validateProjectText('𝐅ancy project')?.message.id,
		'project.text-validation.non-standard-text',
	)
})

test('validates project title metadata', () => {
	assert.deepEqual(validateProjectTitle('Fabric Tools', metadata), {
		severity: 'error',
		message: {
			id: 'project.text-validation.title-loader',
			defaultMessage: 'Project titles cannot include the loader “{value}”.',
		},
		values: { value: 'fabric' },
	})
	assert.equal(validateProjectTitle('Ordinary Tools', metadata), null)
})

test('validates project summaries', () => {
	assert.equal(
		validateProjectSummary('Visit modrinth.com', 'Project title')?.message.id,
		'project.text-validation.summary-link',
	)
	assert.equal(
		validateProjectSummary('  Caf\u00e9  ', 'Cafe\u0301')?.message.id,
		'project.text-validation.summary-matches-title',
	)
	assert.equal(validateProjectSummary('Project summary', 'Project title'), null)
})

test('allows sparse non-standard text in descriptions but rejects it at the threshold', () => {
	const belowFivePercent = '𝐀'.concat('a'.repeat(20))
	const exactlyFivePercent = '𝐀'.concat('a'.repeat(19))

	assert.equal(validateProjectDescription(belowFivePercent), null)
	assert.equal(
		validateProjectDescription(exactlyFivePercent)?.message.id,
		'project.text-validation.non-standard-text',
	)
})
