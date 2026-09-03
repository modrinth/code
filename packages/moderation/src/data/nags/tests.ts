import assert from 'node:assert/strict'
import test from 'node:test'

import {
	getProjectNagValues,
	normalizeProjectNagKind,
	toProjectFieldMessage,
	toProjectNag,
} from './index.ts'

test('normalizes backend snake-case nag kinds', () => {
	assert.equal(normalizeProjectNagKind('multiple_resolution_tags'), 'multiple-resolution-tags')
	assert.equal(normalizeProjectNagKind('unknown_nag'), null)
})

test('formats backend resolution tags for ICU messages', () => {
	assert.deepEqual(
		getProjectNagValues({
			kind: 'multiple_resolution_tags',
			severity: 'warning',
			details: { count: 3, tags: '8x-|32x|512x+' },
		}),
		{
			count: 3,
			tags: '8x or lower, 32x, 512x or higher',
		},
	)
})

test('maps snake-case backend details to ICU variable names', () => {
	assert.deepEqual(
		getProjectNagValues({
			kind: 'too_many_tags',
			severity: 'warning',
			details: {
				full_url: 'https://example.com',
				language_count: 11,
				max_tag_count: 8,
				tag_count: 9,
				total_available_tags: 20,
			},
		}),
		{
			fullUrl: 'https://example.com',
			languageCount: 11,
			maxTagCount: 8,
			tagCount: 9,
			totalAvailableTags: 20,
		},
	)
})

test('maps required backend nags to field errors', () => {
	const nag = {
		kind: 'project_name_profanity',
		severity: 'required',
		details: { value: 'example' },
	} as const
	const message = toProjectFieldMessage(nag)

	assert.equal(message.code, 'project-name-profanity')
	assert.equal(message.severity, 'error')
	assert.deepEqual(message.values, { value: 'example' })
	assert.equal(toProjectNag(nag).status, 'required')
})

test('uses project-specific gallery copy', () => {
	const message = toProjectFieldMessage(
		{
			kind: 'upload_gallery_image',
			severity: 'required',
			details: {},
		},
		'shader',
	)

	assert.equal(message.message.id, 'nags.upload-gallery-image.description-shader')
})

test('uses detailed license URL copy when the backend supplies a domain', () => {
	const message = toProjectFieldMessage({
		kind: 'invalid_license_url',
		severity: 'required',
		details: { domain: 'example.com' },
	})

	assert.equal(message.message.id, 'nags.invalid-license-url.description.domain')
})
