/**
 * CSL public skin helper unit tests (no Express / DB).
 * Run: node --test owyxsite/backend/src/routes/csl.test.js
 */

const { describe, it, beforeEach, afterEach } = require('node:test')
const assert = require('node:assert/strict')
const path = require('path')
const fs = require('fs')

const {
	absoluteWebsiteAsset,
	resolveLocalUpload,
} = require('./csl-helpers')

describe('absoluteWebsiteAsset', () => {
	const prev = {}

	beforeEach(() => {
		for (const k of ['SITE_PUBLIC_URL', 'PUBLIC_SITE_URL', 'API_PUBLIC_URL']) {
			prev[k] = process.env[k]
			delete process.env[k]
		}
	})

	afterEach(() => {
		for (const k of Object.keys(prev)) {
			if (prev[k] === undefined) delete process.env[k]
			else process.env[k] = prev[k]
		}
	})

	it('returns null for empty', () => {
		assert.equal(absoluteWebsiteAsset(null), null)
		assert.equal(absoluteWebsiteAsset(''), null)
	})

	it('allows relative /uploads/skins paths', () => {
		assert.equal(
			absoluteWebsiteAsset('/uploads/skins/skin-1.png'),
			'https://owyx.site/uploads/skins/skin-1.png',
		)
	})

	it('rejects non-uploads relative paths', () => {
		assert.equal(absoluteWebsiteAsset('/evil/x.png'), null)
	})

	it('rejects relative path traversal', () => {
		assert.equal(absoluteWebsiteAsset('/uploads/skins/../../evil.png'), null)
	})

	it('allows avatars under /uploads/avatars/', () => {
		assert.equal(
			absoluteWebsiteAsset('/uploads/avatars/a.png'),
			'https://owyx.site/uploads/avatars/a.png',
		)
	})

	it('rejects percent-encoded path traversal', () => {
		assert.equal(
			absoluteWebsiteAsset('/uploads/skins/%2e%2e/%2e%2e/etc/passwd'),
			null,
		)
		assert.equal(
			absoluteWebsiteAsset('/uploads/skins/%2E%2E/secret.png'),
			null,
		)
	})

	it('rejects third-party absolute URLs (open redirect)', () => {
		assert.equal(absoluteWebsiteAsset('https://evil.example/x.png'), null)
		assert.equal(absoluteWebsiteAsset('http://evil.example/uploads/skins/x.png'), null)
	})

	it('rewrites api.owyx.site uploads to website origin', () => {
		assert.equal(
			absoluteWebsiteAsset('https://api.owyx.site/uploads/skins/skin-1.png'),
			'https://owyx.site/uploads/skins/skin-1.png',
		)
	})

	it('keeps owyx.site upload URLs', () => {
		assert.equal(
			absoluteWebsiteAsset('https://owyx.site/uploads/skins/skin-1.png'),
			'https://owyx.site/uploads/skins/skin-1.png',
		)
	})

	it('rejects allowlisted host without /uploads/ path', () => {
		assert.equal(absoluteWebsiteAsset('https://owyx.site/login'), null)
	})
})

describe('resolveLocalUpload', () => {
	it('rejects path traversal', () => {
		assert.equal(resolveLocalUpload('/uploads/skins/../../etc/passwd'), null)
		assert.equal(resolveLocalUpload('/uploads/skins/../secrets.png'), null)
		assert.equal(resolveLocalUpload('/uploads/skins/%2e%2e/%2e%2e/etc/passwd'), null)
	})

	it('rejects non-upload dirs', () => {
		assert.equal(resolveLocalUpload('/uploads/other/a.png'), null)
		assert.equal(resolveLocalUpload('/uploads/secrets.png'), null)
	})

	it('returns null when file missing', () => {
		assert.equal(resolveLocalUpload('/uploads/skins/does-not-exist-owyx.png'), null)
	})

	it('resolves an existing skin under uploads/skins', () => {
		const root = path.resolve(path.join(__dirname, '../../uploads/skins'))
		fs.mkdirSync(root, { recursive: true })
		const file = path.join(root, 'csl-test-skin.png')
		fs.writeFileSync(file, Buffer.alloc(128, 1))
		try {
			const resolved = resolveLocalUpload('/uploads/skins/csl-test-skin.png')
			assert.equal(resolved, file)
		} finally {
			fs.unlinkSync(file)
		}
	})
})
