import assert from 'node:assert/strict'
import test from 'node:test'

import { effectScope, ref } from 'vue'

import { checkLink, getLinkCheckState, isLinkCheckPending, useLinkCheck } from './index.ts'

test('rejects invalid and insecure URLs', async () => {
	const insecure = { field: 'source', url: 'http://github.com/modrinth/code' }
	const reserved = { field: 'source', url: 'https://example.com/project' }

	await checkLink(insecure)
	await checkLink(reserved)

	assert.equal(getLinkCheckState(insecure)?.severity, 'error')
	assert.equal(getLinkCheckState(reserved)?.severity, 'error')
})

test('uses a description-specific message for invalid content links', async () => {
	const context = {
		field: 'description',
		url: 'http://example.dev/project',
		generalContent: true,
	}

	await checkLink(context)

	assert.equal(getLinkCheckState(context)?.message?.id, 'nags.link.description.invalid-url')
	assert.equal(
		getLinkCheckState(context)?.message?.defaultMessage,
		'The description has an invalid link',
	)
})

test('matches recognized hosts case-insensitively', async () => {
	const googleForm = { field: 'issues', url: 'https://DOCS.GOOGLE.COM/forms/d/e/example' }
	const shortener = { field: 'source', url: 'https://BIT.LY/example' }

	await checkLink(googleForm)
	await checkLink(shortener)

	assert.equal(getLinkCheckState(googleForm)?.severity, 'valid')
	assert.equal(getLinkCheckState(shortener)?.severity, 'error')
})

test('rejects a recognized link used in the wrong field', async () => {
	const context = { field: 'wiki', url: 'https://docs.google.com/forms/d/e/example' }

	await checkLink(context)

	assert.equal(getLinkCheckState(context)?.severity, 'error')
	assert.equal(getLinkCheckState(context)?.message?.id, 'nags.link.wrong-field')
})

test('allows structured link types in general content', async () => {
	const context = {
		field: 'description',
		url: 'https://github.com/modrinth/code',
		generalContent: true,
	}

	await checkLink(context)

	assert.equal(getLinkCheckState(context)?.severity, 'valid')
})

test('allows unrecognized valid links but keeps global restrictions in general content', async () => {
	const allowed = {
		field: 'description',
		url: 'https://docs.example.dev/project',
		generalContent: true,
	}
	const blocked = {
		field: 'description',
		url: 'https://bit.ly/project',
		generalContent: true,
	}

	await checkLink(allowed)
	await checkLink(blocked)

	assert.equal(getLinkCheckState(allowed)?.severity, 'valid')
	assert.equal(getLinkCheckState(blocked)?.severity, 'error')
})

test('applies the external-link blocklist only outside general content', async () => {
	const blockedExternalLink = {
		field: 'site',
		url: 'https://social.modrinth.com/project',
	}
	const allowedContentLink = {
		field: 'description',
		url: 'https://social.modrinth.com/project',
		generalContent: true,
	}
	const allowed = {
		field: 'description',
		url: 'https://modrinth.com.example.dev/project',
		generalContent: true,
	}

	await checkLink(blockedExternalLink)
	await checkLink(allowedContentLink)
	await checkLink(allowed)

	assert.equal(getLinkCheckState(blockedExternalLink)?.severity, 'error')
	assert.equal(getLinkCheckState(allowedContentLink)?.severity, 'valid')
	assert.equal(getLinkCheckState(allowed)?.severity, 'valid')
})

test('compares recognized license URLs with the selected license', async () => {
	const matching = {
		field: 'license',
		url: 'https://spdx.org/licenses/MIT.html',
		expectedLicense: 'MIT',
		isCustom: false,
	}
	const mismatching = {
		field: 'license',
		url: 'https://spdx.org/licenses/MIT.html',
		expectedLicense: 'Apache-2.0',
		isCustom: false,
	}

	await checkLink(matching)
	await checkLink(mismatching)

	assert.equal(getLinkCheckState(matching)?.severity, 'valid')
	assert.equal(getLinkCheckState(mismatching)?.severity, 'warn')
})

test('marks debounced checks as pending immediately', () => {
	const context = ref({ field: 'source', url: 'https://bit.ly/example' })
	const scope = effectScope()

	scope.run(() => useLinkCheck(context))

	assert.equal(isLinkCheckPending(context.value), true)
	scope.stop()
	assert.equal(isLinkCheckPending(context.value), false)
})
