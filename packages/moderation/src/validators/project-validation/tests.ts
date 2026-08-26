import assert from 'node:assert/strict'
import test from 'node:test'

import type { Labrinth } from '@modrinth/api-client'

import type { ProjectTitleMetadata } from '../project-fields/index.ts'
import { hasProjectFieldValidationFailures, validateProjectFields } from './index.ts'

const metadata: ProjectTitleMetadata = {
	gameVersions: ['1.21.1'],
	loaders: ['fabric'],
}

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
		name: 'Fabric Tools',
		summary: 'Fabric Tools',
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

	const result = validateProjectFields(project, metadata)

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
				field: 'name',
				galleryIndex: undefined,
				galleryUrl: undefined,
				message: 'project.text-validation.title-loader',
			},
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

	assert.deepEqual(validateProjectFields(validProject, metadata), {
		valid: true,
		failures: [],
	})
	assert.equal(hasProjectFieldValidationFailures(validProject, metadata), false)
	assert.equal(hasProjectFieldValidationFailures(invalidProject, metadata), true)
})

test('treats title metadata and summary content recommendations as warnings', () => {
	const project = createProject({
		name: 'Fabric Tools',
		summary: 'Visit modrinth.com for more information',
	})
	const result = validateProjectFields(project, metadata)

	assert.equal(result.valid, true)
	assert.deepEqual(
		result.failures.map(({ code, severity }) => ({ code, severity })),
		[
			{ code: 'title-loader', severity: 'warn' },
			{ code: 'summary-link', severity: 'warn' },
		],
	)
	assert.equal(hasProjectFieldValidationFailures(project, metadata), false)
})

test('reports summary recommendations without invalidating the project', () => {
	const project = createProject({ summary: 'Short summary' })

	assert.deepEqual(validateProjectFields(project, metadata), {
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
	assert.equal(hasProjectFieldValidationFailures(project, metadata), false)
})
