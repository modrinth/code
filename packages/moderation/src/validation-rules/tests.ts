import assert from 'node:assert/strict'
import test from 'node:test'

import { evaluateRules } from './evaluate-rules.ts'
import {
	analyzeImageContent,
	BANNED_DESCRIPTION_LINK_DOMAINS,
	countText,
	extractDescriptionLinks,
	findBannedDescriptionLink,
	MIN_DESCRIPTION_CHARS,
	validateProjectDescription,
} from './rules/description.ts'
import { validateProjectGalleryDescription, validateProjectGalleryName } from './rules/gallery.ts'
import { projectNameValidationRules, validateProjectNameField } from './rules/name.ts'
import { projectSummaryMatchesName, validateProjectSummary } from './rules/summary.ts'
import { toFieldMessages } from './to-field-messages.ts'
import { toNags } from './to-nags.ts'
import type { ValidationRuleSet } from './types.ts'

test('evaluates matching rules in definition order and converts them to field messages', () => {
	const rules = {
		'without-values': {
			severity: 'error',
			evaluate: () => ({ valid: false }),
			presentation: {
				message: { id: 'without-values' },
				nag: { title: { id: 'without-values-title' }, destination: 'general' },
			},
		},
		'with-values': {
			severity: 'warning',
			evaluate: (value) => ({ valid: false, values: { value } }),
			presentation: {
				message: { id: 'with-values' },
				nag: { title: { id: 'with-values-title' }, destination: 'general' },
			},
		},
		valid: {
			severity: 'error',
			evaluate: () => ({ valid: true }),
			presentation: {
				message: { id: 'valid' },
				nag: { title: { id: 'valid-title' }, destination: 'general' },
			},
		},
	} satisfies ValidationRuleSet<string>

	const matches = evaluateRules('detected', rules)
	assert.deepEqual(
		matches.map(({ code }) => code),
		['without-values', 'with-values'],
	)
	assert.deepEqual(toFieldMessages(matches), [
		{
			code: 'without-values',
			severity: 'error',
			message: { id: 'without-values' },
			values: undefined,
		},
		{
			code: 'with-values',
			severity: 'warning',
			message: { id: 'with-values' },
			values: { value: 'detected' },
		},
	])
})

test('allows clean project names and versioned ports', () => {
	assert.deepEqual(evaluateRules('Sodium Extras', projectNameValidationRules), [])
	assert.deepEqual(evaluateRules('Sodium 1.20 Port', projectNameValidationRules), [])
})

test('collects every matching project name rule', () => {
	const matches = evaluateRules('Minecraft 1.20 fuck', projectNameValidationRules)

	assert.deepEqual(
		matches.map(({ code }) => code),
		['project-name-profanity', 'project-name-version', 'minecraft-title-clause'],
	)
	assert.deepEqual(matches[0]?.values, { value: 'fuck' })
	assert.equal(matches[1]?.rule.severity, 'error')
	assert.equal(matches[2]?.rule.severity, 'error')
})

test('derives project name field presentation from the matching rule', () => {
	assert.deepEqual(validateProjectNameField('Minecraft'), [
		{
			code: 'minecraft-title-clause',
			severity: 'error',
			message: {
				id: 'nags.minecraft-title-clause.description',
				defaultMessage:
					'Projects must not use Minecraft\'s branding or include "Minecraft" as a significant part of the name.',
			},
			values: undefined,
		},
	])
})

test('derives project name nags from the same matching rule', () => {
	const [nag] = toNags(evaluateRules('Minecraft', projectNameValidationRules))

	assert.equal(nag?.id, 'minecraft-title-clause')
	assert.equal(nag?.status, 'required')
	assert.equal(nag?.title.id, 'nags.minecraft-title-clause.title')
	assert.equal(nag?.link?.path, 'settings')
	assert.equal(nag?.link?.title.id, 'nags.edit-title.title')
})

test('validates summary content from one rule set', () => {
	assert.equal(projectSummaryMatchesName('  Caf\u00e9  ', 'Cafe\u0301'), true)
	assert.deepEqual(
		validateProjectSummary({ summary: '# Short summary', name: 'Project title' }).map(
			({ code, severity }) => ({ code, severity }),
		),
		[
			{ code: 'summary-too-short', severity: 'error' },
			{ code: 'summary-special-formatting', severity: 'error' },
		],
	)
	assert.deepEqual(
		validateProjectSummary({
			summary: 'A detailed summary of this excellent project',
			name: 'Project title',
		}),
		[],
	)
})

test('rejects repeated summary padding', () => {
	assert.deepEqual(
		validateProjectSummary({
			summary: 'Useful project! '.repeat(3),
			name: 'Project title',
		}).map(({ code }) => code),
		['project-summary-spam'],
	)
})

test('rejects every link and IP address in project summaries', () => {
	for (const summary of [
		'Visit https://example.dev for more information about this project',
		'Visit example.dev for more information about this project',
		'Join 127.0.0.1:25565 for more information about this project',
	]) {
		assert.deepEqual(
			validateProjectSummary({ summary, name: 'Project title' }).map(({ code }) => code),
			['summary-special-formatting'],
		)
	}

	assert.deepEqual(
		validateProjectSummary({
			summary: 'Contact hello@example.com for more information about this project',
			name: 'Project title',
		}),
		[],
	)
})

test('rejects configured description links and allows other links', () => {
	for (const domain of BANNED_DESCRIPTION_LINK_DOMAINS) {
		assert.equal(
			findBannedDescriptionLink(`Visit https://${domain}/project`),
			`https://${domain}/project`,
		)
		assert.equal(
			findBannedDescriptionLink(`Visit subdomain.${domain}/project`),
			`http://subdomain.${domain}/project`,
		)
	}

	assert.equal(findBannedDescriptionLink('Visit https://example.dev/project'), null)
	assert.equal(findBannedDescriptionLink('Join 127.0.0.1:25565'), null)
	assert.deepEqual(extractDescriptionLinks('Join 127.0.0.1:25565 or visit example.dev'), [
		'http://example.dev',
	])
})

test('validates description requirements and simultaneous recommendations', () => {
	assert.deepEqual(
		validateProjectDescription('  ').map(({ code }) => code),
		['add-description'],
	)

	const description = `${'# '.concat('A'.repeat(81))}\n![](one.png)\n![](two.png)\n![](three.png)\n![](four.png)`
	assert.deepEqual(
		validateProjectDescription(description).map(({ code }) => code),
		['description-too-short', 'project-description-spam', 'long-headers', 'missing-alt-text'],
	)
})

test('requires 125 readable description characters', () => {
	const description =
		'This project adds useful tools, flexible behavior, accessible documentation, polished gameplay, and support for every player.'

	assert.equal(countText(description), MIN_DESCRIPTION_CHARS)
	assert.deepEqual(
		validateProjectDescription(description.slice(0, -1)).map(({ code }) => code),
		['description-too-short'],
	)
	assert.deepEqual(validateProjectDescription(description), [])
})

test('rejects repeated description padding', () => {
	assert.deepEqual(
		validateProjectDescription('Useful project! '.repeat(10)).map(({ code }) => code),
		['project-description-spam'],
	)
})

test('requires alt text for description images', () => {
	assert.deepEqual(analyzeImageContent('![Screenshot](screenshot.png)'), {
		hasEmptyAltText: false,
	})
	assert.deepEqual(analyzeImageContent('![](screenshot.png)'), { hasEmptyAltText: true })
	assert.deepEqual(analyzeImageContent('<img src="screenshot.png">'), { hasEmptyAltText: true })
})

test('counts image alt text as readable description text', () => {
	assert.equal(countText('![Project screenshot](screenshot.png)'), 'Project screenshot'.length)
	assert.equal(
		countText('<img src="screenshot.png" alt="Project screenshot">'),
		'Project screenshot'.length,
	)
})

test('counts blockquote content as readable description text', () => {
	assert.equal(countText('> Quoted text'), 'Quoted text'.length)

	const quotedDescription =
		'> This project adds useful tools, flexible behavior, accessible documentation, polished gameplay, and support for every player.'
	assert.equal(countText(quotedDescription), MIN_DESCRIPTION_CHARS)
	assert.equal(
		validateProjectDescription(quotedDescription).some(
			({ code }) => code === 'description-too-short',
		),
		false,
	)
})

test('uses the gallery text rules for names and descriptions', () => {
	assert.deepEqual(
		validateProjectGalleryName('This is $h!t').map(({ code }) => code),
		['gallery-text-profanity'],
	)
	assert.deepEqual(
		validateProjectGalleryDescription('𝐁').map(({ code }) => code),
		['gallery-text-non-standard'],
	)
})
