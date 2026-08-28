import assert from 'node:assert/strict'
import test from 'node:test'

import {
	EXTERNAL_LINKS_BLOCK_LIST,
	getBlockedProjectExternalLink,
	getLinkHostname,
	isCommonProjectLink,
	isDiscordLink,
	isInappropriateLicenseLink,
	URL_SHORTENERS,
	validateLink,
	validateLinkSyntax,
} from './index.ts'

test('rejects invalid and insecure URLs', async () => {
	const insecure = await validateLink({
		field: 'source',
		url: 'http://github.com/modrinth/code',
	})
	const reserved = await validateLink({
		field: 'source',
		url: 'https://example.com/project',
	})

	assert.equal(insecure?.severity, 'error')
	assert.equal(reserved?.severity, 'error')
})

test('uses a description-specific message for invalid content links', async () => {
	const result = await validateLink({
		field: 'description',
		url: 'http://example.dev/project',
		generalContent: true,
	})

	assert.equal(result?.message?.id, 'nags.link.description.invalid-url')
	assert.equal(result?.message?.defaultMessage, 'The description has an invalid link: “{fullUrl}”.')
	assert.deepEqual(result?.values, { fullUrl: 'http://example.dev/project' })
})

test('matches recognized hosts case-insensitively', async () => {
	const googleForm = await validateLink({
		field: 'issues',
		url: 'https://DOCS.GOOGLE.COM/forms/d/e/example',
	})
	const shortener = await validateLink({ field: 'source', url: 'https://BIT.LY/example' })

	assert.equal(googleForm?.severity, 'valid')
	assert.equal(shortener?.severity, 'error')
})

test('rejects a recognized link used in the wrong field', async () => {
	const result = await validateLink({
		field: 'wiki',
		url: 'https://docs.google.com/forms/d/e/example',
	})

	assert.equal(result?.severity, 'error')
	assert.equal(result?.message?.id, 'nags.link.wrong-field')
})

test('allows structured link types in general content', async () => {
	const result = await validateLink({
		field: 'description',
		url: 'https://github.com/modrinth/code',
		generalContent: true,
	})

	assert.equal(result?.severity, 'valid')
})

test('allows unrecognized valid links in general content', async () => {
	const allowed = await validateLink({
		field: 'description',
		url: 'https://docs.example.dev/project',
		generalContent: true,
	})
	const shortener = await validateLink({
		field: 'description',
		url: 'https://bit.ly/project',
		generalContent: true,
	})

	assert.equal(allowed?.severity, 'valid')
	assert.equal(shortener?.severity, 'valid')
})

test('applies the external-link blocklist only outside general content', async () => {
	const blockedExternalLink = await validateLink({
		field: 'site',
		url: 'https://social.modrinth.com/project',
	})
	const allowedContentLink = await validateLink({
		field: 'description',
		url: 'https://social.modrinth.com/project',
		generalContent: true,
	})
	const allowed = await validateLink({
		field: 'description',
		url: 'https://modrinth.com.example.dev/project',
		generalContent: true,
	})

	assert.equal(blockedExternalLink?.severity, 'error')
	assert.equal(allowedContentLink?.severity, 'valid')
	assert.equal(allowed?.severity, 'valid')
})

test('compares recognized license URLs with the selected license', async () => {
	const matching = await validateLink({
		field: 'license',
		url: 'https://spdx.org/licenses/MIT.html',
		expectedLicense: 'MIT',
		isCustom: false,
	})
	const mismatching = await validateLink({
		field: 'license',
		url: 'https://spdx.org/licenses/MIT.html',
		expectedLicense: 'Apache-2.0',
		isCustom: false,
	})

	assert.equal(matching?.severity, 'valid')
	assert.equal(mismatching?.severity, 'warn')
})

test('blocks every configured URL shortener and its subdomains', () => {
	for (const domain of URL_SHORTENERS) {
		assert.equal(
			getBlockedProjectExternalLink(`https://subdomain.${domain}/project`)?.label,
			'URL shortener',
		)
	}
})

test('blocks every configured external domain and its subdomains', () => {
	for (const { label, domains } of EXTERNAL_LINKS_BLOCK_LIST) {
		for (const domain of domains) {
			assert.deepEqual(getBlockedProjectExternalLink(`https://${domain}/project`), {
				label,
				url: `https://${domain}/project`,
			})
			assert.equal(
				getBlockedProjectExternalLink(`https://subdomain.${domain}/project`)?.label,
				label,
			)
		}
	}
})

test('blocks configured external links', () => {
	assert.equal(
		getBlockedProjectExternalLink('https://social.modrinth.com/project')?.label,
		'Modrinth',
	)
})

test('blocks IP-address URLs without blocking domain lookalikes', () => {
	assert.equal(getBlockedProjectExternalLink('http://127.0.0.1:25565')?.label, 'IP address')
	assert.equal(getBlockedProjectExternalLink('https://[2001:db8::1]')?.label, 'IP address')
	assert.equal(getBlockedProjectExternalLink('https://modrinth.com.example.dev'), null)
	assert.equal(getBlockedProjectExternalLink('not a URL'), null)
})

test('matches classified domains exactly or by subdomain', () => {
	assert.equal(isCommonProjectLink('https://github.com/modrinth/code', 'source'), true)
	assert.equal(isCommonProjectLink('https://subdomain.github.com/modrinth/code', 'source'), true)
	assert.equal(isCommonProjectLink('https://fakegithub.com/modrinth/code', 'source'), false)
	assert.equal(isCommonProjectLink('https://github.com.example.com/modrinth/code', 'source'), false)
	assert.equal(isDiscordLink('https://discord.gg/modrinth'), true)
	assert.equal(isDiscordLink('https://discord.gg.example.com/modrinth'), false)
	assert.equal(isInappropriateLicenseLink('https://youtube.com/watch?v=example'), true)
	assert.equal(isInappropriateLicenseLink('https://youtube.com.evil.dev/license'), false)
})

test('extracts normalized hostnames from valid web URLs', () => {
	assert.equal(getLinkHostname('https://GITHUB.COM./modrinth/code'), 'github.com')
	assert.equal(getLinkHostname('not a URL'), null)
	assert.equal(getLinkHostname('mailto:example@example.com'), null)
})

test('validates recognized link syntax without performing remote checks', () => {
	const source = validateLinkSyntax({
		field: 'source',
		url: 'https://github.com/modrinth/code',
	})
	const wrongField = validateLinkSyntax({
		field: 'wiki',
		url: 'https://docs.google.com/forms/d/e/example',
	})

	assert.equal(source?.severity, 'valid')
	assert.equal(wrongField?.severity, 'error')
	assert.equal(wrongField?.message?.id, 'nags.link.wrong-field')
})
