import assert from 'node:assert/strict'
import test from 'node:test'

import { getBlockedProjectContentLink, PROJECT_CONTENT_LINK_BLOCKLIST } from './index.ts'

test('blocks every configured project-content domain and its subdomains', () => {
	for (const { label, domains } of PROJECT_CONTENT_LINK_BLOCKLIST) {
		for (const domain of domains) {
			assert.deepEqual(getBlockedProjectContentLink(`https://${domain}/project`), {
				label,
				url: `https://${domain}/project`,
			})
			assert.equal(
				getBlockedProjectContentLink(`https://subdomain.${domain}/project`)?.label,
				label,
			)
		}
	}
})

test('blocks IP-address URLs without blocking domain lookalikes', () => {
	assert.equal(getBlockedProjectContentLink('http://127.0.0.1:25565')?.label, 'IP address')
	assert.equal(getBlockedProjectContentLink('https://[2001:db8::1]')?.label, 'IP address')
	assert.equal(getBlockedProjectContentLink('https://modrinth.com.example.dev'), null)
	assert.equal(getBlockedProjectContentLink('not a URL'), null)
})
