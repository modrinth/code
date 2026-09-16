const { describe, it, beforeEach, afterEach } = require('node:test')
const assert = require('node:assert/strict')
const { verifyTurnstile } = require('./turnstile')

describe('verifyTurnstile production fail-close', () => {
	const prev = {}
	beforeEach(() => {
		for (const k of ['NODE_ENV', 'TURNSTILE_SKIP', 'TURNSTILE_SECRET_KEY', 'TURNSTILE_SECRET']) {
			prev[k] = process.env[k]
		}
	})
	afterEach(() => {
		for (const [k, v] of Object.entries(prev)) {
			if (v === undefined) delete process.env[k]
			else process.env[k] = v
		}
	})

	it('rejects in production without usable secret', async () => {
		process.env.NODE_ENV = 'production'
		delete process.env.TURNSTILE_SKIP
		process.env.TURNSTILE_SECRET_KEY = 'obt-pending-placeholder'
		const result = await verifyTurnstile('any-token')
		assert.equal(result.success, false)
		assert.equal(result.code, 'turnstile_misconfigured')
	})

	it('skips in development without secret', async () => {
		process.env.NODE_ENV = 'development'
		delete process.env.TURNSTILE_SECRET_KEY
		delete process.env.TURNSTILE_SECRET
		const result = await verifyTurnstile(null)
		assert.equal(result.success, true)
		assert.equal(result.skipped, true)
	})
})
