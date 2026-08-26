import assert from 'node:assert/strict'
import test from 'node:test'

import {
	containsProjectLinkOrIp,
	extractProjectLinks,
	projectSummaryMatchesTitle,
	validateProjectDescription,
	validateProjectSummary,
	validateProjectText,
	validateProjectTitle,
} from './index.ts'

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
	assert.deepEqual(validateProjectText('An ordinary project'), [])
	assert.deepEqual(validateProjectText('This project is SHIT')[0], {
		code: 'text-profanity',
		severity: 'error',
		message: {
			id: 'project.text-validation.profanity',
			defaultMessage: 'The detected profanity “{value}” is not allowed.',
		},
		values: { value: 'SHIT' },
	})
	assert.deepEqual(validateProjectText('F.A.G')[0]?.values, { value: 'F.A.G' })
	assert.equal(
		validateProjectText('𝐅ancy project')[0]?.message.id,
		'project.text-validation.non-standard-text',
	)
})

test('validates project titles', () => {
	assert.deepEqual(validateProjectTitle('Tools 1.2.3'), [
		{
			code: 'title-version-number',
			severity: 'error',
			message: {
				id: 'project.text-validation.title-version-number',
				defaultMessage: 'Names are not allowed to include version numbers.',
			},
		},
	])
	assert.deepEqual(validateProjectTitle('Tools 1.2'), [
		{
			code: 'title-version-number',
			severity: 'error',
			message: {
				id: 'project.text-validation.title-version-number',
				defaultMessage: 'Names are not allowed to include version numbers.',
			},
		},
	])
	assert.deepEqual(validateProjectTitle('My Mod 1.2 Fabric Port'), [])
	assert.deepEqual(validateProjectTitle('My Mod 1.2 FORK'), [])
	assert.equal(validateProjectTitle('My Port of Mod 1.2')[0]?.code, 'title-version-number')
	assert.equal(validateProjectTitle('My Mod 1.2 Supported')[0]?.code, 'title-version-number')
	assert.deepEqual(validateProjectTitle('Fabric Tools'), [])
	assert.equal(validateProjectTitle('Minecraft Tools')[0]?.code, 'title-minecraft-branding')
	assert.deepEqual(
		validateProjectTitle('Minecraft Tools 1.2').map(({ code }) => code),
		['title-version-number', 'title-minecraft-branding'],
	)
	assert.deepEqual(validateProjectTitle('Ordinary Tools'), [])
})

test('validates project summaries', () => {
	assert.equal(
		validateProjectSummary('Visit modrinth.com', 'Project title')[0]?.message.id,
		'project.text-validation.summary-link',
	)
	assert.equal(validateProjectSummary('Visit modrinth.com', 'Project title')[0]?.severity, 'warn')
	assert.equal(
		validateProjectSummary('  Caf\u00e9  ', 'Cafe\u0301')[0]?.message.id,
		'project.text-validation.summary-matches-title',
	)
	assert.equal(validateProjectSummary('  Caf\u00e9  ', 'Cafe\u0301')[0]?.severity, 'warn')
	assert.deepEqual(validateProjectSummary('Short summary', 'Project title'), [
		{
			code: 'summary-too-short',
			severity: 'warn',
			message: {
				id: 'project.text-validation.summary-too-short',
				defaultMessage:
					'Your summary is {length, plural, one {# character} other {# characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create an informative and enticing summary.',
			},
			values: { length: 13, minChars: 30 },
		},
	])
	assert.deepEqual(
		validateProjectSummary('A detailed summary of this excellent project', 'Project title'),
		[],
	)
	assert.deepEqual(
		validateProjectSummary('# Short summary', 'Project title').map(({ code }) => code),
		['summary-too-short', 'summary-special-formatting'],
	)
})

test('allows sparse non-standard text in descriptions but rejects it at the threshold', () => {
	const belowFivePercent = '𝐀'.concat('a'.repeat(20))
	const exactlyFivePercent = '𝐀'.concat('a'.repeat(19))

	assert.equal(
		validateProjectDescription(belowFivePercent).some(({ code }) => code === 'text-non-standard'),
		false,
	)
	assert.equal(
		validateProjectDescription(exactlyFivePercent)[0]?.message.id,
		'project.text-validation.non-standard-text',
	)
})

test('allows one profanity match in descriptions but rejects a second match or any slur', () => {
	const description = 'A detailed project description '.repeat(10)

	assert.equal(
		validateProjectDescription(`${description} shit`).some(({ code }) => code === 'text-profanity'),
		false,
	)
	assert.equal(
		validateProjectDescription(`${description} f.u.c.k`).some(
			({ code }) => code === 'text-profanity',
		),
		false,
	)
	assert.deepEqual(validateProjectDescription(`${description} shit FUCK`)[0], {
		code: 'text-profanity',
		severity: 'error',
		message: {
			id: 'project.text-validation.description-profanity',
			defaultMessage: 'Excessive profanity is not allowed. Detected: {values}',
		},
		values: { values: '"shit", "FUCK"' },
	})
	assert.deepEqual(validateProjectDescription(`${description} shit FUCK bastard`)[0]?.values, {
		values: '"shit", "FUCK", "bastard"',
	})
	assert.equal(validateProjectDescription(`${description} nigga`)[0]?.code, 'text-slur')
	assert.equal(validateProjectDescription(`${description} nigger`)[0]?.code, 'text-slur')
})

test('validates required description content and returns simultaneous recommendations', () => {
	assert.equal(validateProjectDescription('  ')[0]?.code, 'description-required')

	const description = `${'# '.concat('A'.repeat(81))}\n![](one.png)\n![](two.png)\n![](three.png)\n![](four.png)`
	assert.deepEqual(
		validateProjectDescription(description).map(({ code }) => code),
		[
			'description-too-short',
			'description-long-headers',
			'description-image-heavy',
			'description-missing-alt-text',
		],
	)
})
