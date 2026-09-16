/**
 * Mirror of sanitize helpers for Node unit tests (no Vite/TS loader).
 * Keep in sync with apps/app-frontend/src/helpers/owyx-api.ts
 */
const { describe, it } = require('node:test')
const assert = require('node:assert/strict')

const DEFAULT = 'https://api.owyx.site'

function sanitizeOwyxApiBase(url) {
	const raw = (url ?? '').trim()
	if (!raw) return DEFAULT
	try {
		const parsed = new URL(raw)
		if (parsed.protocol === 'https:') {
			return parsed.origin + (parsed.pathname === '/' ? '' : parsed.pathname.replace(/\/$/, ''))
		}
		if (
			parsed.protocol === 'http:' &&
			(parsed.hostname === '127.0.0.1' || parsed.hostname === 'localhost')
		) {
			return parsed.origin + (parsed.pathname === '/' ? '' : parsed.pathname.replace(/\/$/, ''))
		}
		return DEFAULT
	} catch {
		return DEFAULT
	}
}

function isSafeExternalHttpsUrl(url) {
	if (!url) return false
	const trimmed = url.trim()
	if (trimmed.startsWith('//')) return false
	if (trimmed.startsWith('/')) return true
	try {
		const parsed = new URL(trimmed)
		return parsed.protocol === 'https:'
	} catch {
		return false
	}
}

function resolvePackUrl(url, apiBase) {
	if (!url) return null
	const trimmed = url.trim()
	if (!isSafeExternalHttpsUrl(trimmed)) return null
	if (trimmed.startsWith('/')) {
		return `${sanitizeOwyxApiBase(apiBase).replace(/\/$/, '')}${trimmed}`
	}
	return trimmed
}

describe('sanitizeOwyxApiBase', () => {
	it('defaults and rejects evil http', () => {
		assert.equal(sanitizeOwyxApiBase(''), DEFAULT)
		assert.equal(sanitizeOwyxApiBase('http://evil.com'), DEFAULT)
		assert.equal(sanitizeOwyxApiBase('http://127.0.0.1:3001'), 'http://127.0.0.1:3001')
	})
})

describe('resolvePackUrl', () => {
	it('absolutizes relative pack paths against API base', () => {
		assert.equal(
			resolvePackUrl('/fixtures/packs/demo.zip', 'https://api.owyx.site'),
			'https://api.owyx.site/fixtures/packs/demo.zip',
		)
		assert.equal(resolvePackUrl('//evil.com/x', 'https://api.owyx.site'), null)
	})
})
