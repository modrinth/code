import assert from 'node:assert/strict'
import test from 'node:test'

import {
	containsExplicitHttpProjectLink,
	containsProjectLinkOrIp,
	countText,
	extractProjectLinks,
	MIN_CHARS_PER_IMAGE,
	MIN_DESCRIPTION_CHARS,
	projectSummaryMatchesTitle,
	validateProjectDescription,
	validateProjectSummary,
	validateProjectText,
	validateProjectTitle,
} from './index.ts'

test('compares summaries and titles without whitespace and after Unicode normalization', () => {
	assert.equal(projectSummaryMatchesTitle('  Caf\u00e9  ', 'Cafe\u0301'), true)
	assert.equal(projectSummaryMatchesTitle('Project summary', 'Projectsummary'), true)
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

test('distinguishes explicit HTTP links from bare domains', () => {
	assert.equal(containsExplicitHttpProjectLink('Visit https://myserver.com'), true)
	assert.equal(containsExplicitHttpProjectLink('Visit HTTP://myserver.com'), true)
	assert.equal(containsExplicitHttpProjectLink('Visit myserver.com'), false)
	assert.equal(containsExplicitHttpProjectLink('The protocol is https://'), false)
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
	const summaryContentMessage =
		'Your summary should not contain formatting, line breaks, special characters, or links. The summary only displays plain text.'
	assert.equal(
		validateProjectSummary('Visit https://example.dev', 'Project title')[0]?.message.id,
		'nags.summary-special-formatting.description',
	)
	assert.equal(
		validateProjectSummary('Visit https://example.dev', 'Project title')[0]?.message.defaultMessage,
		summaryContentMessage,
	)
	assert.equal(
		validateProjectSummary('Visit https://example.dev', 'Project title')[0]?.severity,
		'error',
	)
	assert.equal(
		validateProjectSummary('Visit http://example.dev', 'Project title')[0]?.code,
		'summary-link',
	)
	assert.deepEqual(
		validateProjectSummary(
			'Connect at myserver.com to join our friendly community',
			'Project title',
		),
		[],
	)
	assert.equal(
		validateProjectSummary('  Caf\u00e9  ', 'Cafe\u0301')[0]?.message.id,
		'project.text-validation.summary-matches-title',
	)
	assert.equal(validateProjectSummary('  Caf\u00e9  ', 'Cafe\u0301')[0]?.severity, 'error')
	assert.equal(
		validateProjectSummary('  Caf\u00e9  ', 'Cafe\u0301')[0]?.message.defaultMessage,
		"A project summary cannot be the same as it's title.",
	)
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
	assert.equal(validateProjectSummary('# Short summary', 'Project title')[1]?.severity, 'error')
	assert.equal(
		validateProjectSummary('# Short summary', 'Project title')[1]?.message.defaultMessage,
		summaryContentMessage,
	)
})

test('rejects blocklisted links and IP addresses in summaries and descriptions', () => {
	const blockedSummary = validateProjectSummary(
		'Visit https://social.modrinth.com/project',
		'Title',
	)
	assert.deepEqual(blockedSummary[0], {
		code: 'text-banned-link',
		severity: 'error',
		message: {
			id: 'project.text-validation.banned-link',
			defaultMessage: '“{url}” is not allowed in project summaries or descriptions.',
		},
		values: {
			label: 'Modrinth',
			url: 'https://social.modrinth.com/project',
		},
	})

	const blockedDescription = validateProjectDescription(
		`A detailed project description with https://bit.ly/project. ${'More details. '.repeat(20)}`,
	)
	assert.equal(blockedDescription[0]?.code, 'text-banned-link')
	assert.equal(blockedDescription[0]?.values?.label, 'URL shortener')

	const blockedIp = validateProjectSummary('Join 127.0.0.1:25565 to play', 'Title')
	assert.equal(blockedIp[0]?.code, 'text-banned-link')
	assert.equal(blockedIp[0]?.values?.label, 'IP address')

	const allowedDescription = validateProjectDescription(
		`Read more at https://example.dev/project. ${'More details. '.repeat(20)}`,
	)
	assert.equal(
		allowedDescription.some(({ code }) => code === 'text-banned-link'),
		false,
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

test('counts blockquote content as readable description text', () => {
	assert.equal(countText('> Quoted text'), 'Quoted text'.length)
	assert.equal(
		countText('> First line\n> > Nested line\n>\n> - Quoted list item'),
		'First line Nested line Quoted list item'.length,
	)

	const quotedDescription = `> ${'A'.repeat(MIN_DESCRIPTION_CHARS)}`
	assert.equal(
		validateProjectDescription(quotedDescription).some(
			({ code }) => code === 'description-too-short',
		),
		false,
	)

	const images = ['![One](one.png)', '![Two](two.png)', '![Three](three.png)', '![Four](four.png)']
	const quotedImageDescription = [
		`> ${'A'.repeat(MIN_CHARS_PER_IMAGE * images.length)}`,
		...images,
	].join('\n')
	assert.equal(
		validateProjectDescription(quotedImageDescription).some(
			({ code }) => code === 'description-image-heavy',
		),
		false,
	)
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
