import assert from 'node:assert/strict'
import test from 'node:test'

import type { Labrinth } from '@modrinth/api-client'

import { hasProjectFieldValidationFailures, validateProjectFields } from './index.ts'

function createProject(
	overrides: Partial<Labrinth.Projects.v3.Project> = {},
): Labrinth.Projects.v3.Project {
	return {
		name: 'Ordinary Tools',
		summary: 'A collection of ordinary tools.',
		description: 'This project adds a collection of ordinary tools. '.repeat(5),
		gallery: [],
		...overrides,
	} as Labrinth.Projects.v3.Project
}

test('validates project fields and gallery text', () => {
	const project = createProject({
		name: 'Ordinary Tools',
		summary: 'Ordinary Tools',
		description: '𝐀',
		gallery: [
			{
				url: 'https://cdn.modrinth.com/gallery.png',
				raw_url: 'https://cdn.modrinth.com/gallery.png',
				featured: false,
				name: 'This is $h!t',
				description: '𝐁',
				created: '2026-01-01T00:00:00Z',
				ordering: 0,
			},
		],
	})

	const result = validateProjectFields(project)

	assert.equal(result.valid, false)
	assert.deepEqual(
		result.failures.map(({ field, galleryIndex, galleryUrl, message }) => ({
			field,
			galleryIndex,
			galleryUrl,
			message: message.id,
		})),
		[
			{
				field: 'summary',
				galleryIndex: undefined,
				galleryUrl: undefined,
				message: 'project.text-validation.summary-matches-title',
			},
			{
				field: 'description',
				galleryIndex: undefined,
				galleryUrl: undefined,
				message: 'project.text-validation.non-standard-text',
			},
			{
				field: 'gallery-name',
				galleryIndex: 0,
				galleryUrl: 'https://cdn.modrinth.com/gallery.png',
				message: 'project.text-validation.profanity',
			},
			{
				field: 'gallery-description',
				galleryIndex: 0,
				galleryUrl: 'https://cdn.modrinth.com/gallery.png',
				message: 'project.text-validation.non-standard-text',
			},
		],
	)
	assert.deepEqual(result.failures.find(({ field }) => field === 'gallery-name')?.values, {
		value: '$h!t',
	})
})

test('reports whether a project has field validation failures', () => {
	const validProject = createProject()
	const invalidProject = createProject({ summary: 'This project is shit' })

	assert.deepEqual(validateProjectFields(validProject), {
		valid: true,
		failures: [],
	})
	assert.equal(hasProjectFieldValidationFailures(validProject), false)
	assert.equal(hasProjectFieldValidationFailures(invalidProject), true)
	assert.equal(
		hasProjectFieldValidationFailures(
			createProject({ name: 'Ordinary Tools', summary: 'Ordinary Tools' }),
		),
		true,
	)
})

test('treats version numbers and explicit summary links as errors', () => {
	const project = createProject({
		name: 'Tools 1.2.3',
		summary: 'Visit https://example.dev for more information',
	})
	const result = validateProjectFields(project)

	assert.equal(result.valid, false)
	assert.deepEqual(
		result.failures.map(({ code, severity }) => ({ code, severity })),
		[
			{ code: 'title-version-number', severity: 'error' },
			{ code: 'summary-link', severity: 'error' },
		],
	)
	assert.equal(hasProjectFieldValidationFailures(project), true)
	assert.equal(
		hasProjectFieldValidationFailures(createProject({ name: 'Tools 1.2 Fabric Port' })),
		false,
	)
})

test('rejects blocklisted links in summaries and descriptions', () => {
	const summaryResult = validateProjectFields(
		createProject({ summary: 'Visit modrinth.com for more information' }),
	)
	assert.equal(summaryResult.valid, false)
	assert.equal(summaryResult.failures[0]?.field, 'summary')
	assert.equal(summaryResult.failures[0]?.code, 'text-banned-link')

	const descriptionResult = validateProjectFields(
		createProject({ description: `Visit https://bit.ly/project. ${'More details. '.repeat(20)}` }),
	)
	assert.equal(descriptionResult.valid, false)
	assert.equal(descriptionResult.failures[0]?.field, 'description')
	assert.equal(descriptionResult.failures[0]?.code, 'text-banned-link')
})

test('reports summary recommendations without invalidating the project', () => {
	const project = createProject({ summary: 'Short summary' })

	assert.deepEqual(validateProjectFields(project), {
		valid: true,
		failures: [
			{
				code: 'summary-too-short',
				field: 'summary',
				severity: 'warn',
				message: {
					id: 'project.text-validation.summary-too-short',
					defaultMessage:
						'Your summary is {length, plural, one {# character} other {# characters}}. At least {minChars, plural, one {# character} other {# characters}} is recommended to create an informative and enticing summary.',
				},
				values: { length: 13, minChars: 30 },
			},
		],
	})
	assert.equal(hasProjectFieldValidationFailures(project), false)
})

test('treats summary formatting as a field validation failure', () => {
	const project = createProject({ summary: '# A formatted project summary' })
	const result = validateProjectFields(project)

	assert.equal(result.valid, false)
	assert.deepEqual(
		result.failures.map(({ code, severity }) => ({ code, severity })),
		[
			{ code: 'summary-too-short', severity: 'warn' },
			{ code: 'summary-special-formatting', severity: 'error' },
		],
	)
	assert.equal(hasProjectFieldValidationFailures(project), true)
})
