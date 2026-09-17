/**
 * Security gate unit tests (Node, no Express).
 * Run: node --test owyxsite/backend/src/middleware/clientKey.test.js owyxsite/backend/src/utils/turnstile.test.js
 */

const { describe, it, beforeEach, afterEach } = require('node:test')
const assert = require('node:assert/strict')
const { requestHost, clientKeyGate } = require('./clientKey')

function mockReq({ host, hostname, xfHost, path = '/api/launcher/v1/servers', headers = {} }) {
	return {
		path,
		hostname: hostname || host,
		get(name) {
			const k = name.toLowerCase()
			if (k === 'host') return host
			if (k === 'x-forwarded-host') return xfHost
			if (k === 'x-owyx-client-key') return headers['x-owyx-client-key']
			return undefined
		},
	}
}

function runGate(req) {
	let status = 0
	let body = null
	let nextCalled = false
	const res = {
		status(s) {
			status = s
			return this
		},
		json(j) {
			body = j
			return this
		},
	}
	return new Promise((resolve) => {
		clientKeyGate(req, res, () => {
			nextCalled = true
			resolve({ status, body, nextCalled })
		})
		setTimeout(() => resolve({ status, body, nextCalled }), 20)
	})
}

describe('requestHost', () => {
	it('ignores spoofed X-Forwarded-Host', () => {
		const host = requestHost(
			mockReq({ host: 'api.owyx.site', hostname: 'api.owyx.site', xfHost: 'owyx.site' }),
		)
		assert.equal(host, 'api.owyx.site')
	})
})

describe('clientKeyGate', () => {
	const prev = {}

	beforeEach(() => {
		for (const k of ['API_HOSTS', 'SITE_HOSTS', 'LAUNCHER_CLIENT_KEY', 'NODE_ENV']) {
			prev[k] = process.env[k]
		}
		process.env.LAUNCHER_CLIENT_KEY = 'test-key-value'
		process.env.NODE_ENV = 'production'
		delete process.env.SITE_HOSTS
	})

	afterEach(() => {
		for (const [k, v] of Object.entries(prev)) {
			if (v === undefined) delete process.env[k]
			else process.env[k] = v
		}
	})

	it('requires key on api host even with XFH spoof', async () => {
		const result = await runGate(
			mockReq({
				host: 'api.owyx.site',
				hostname: 'api.owyx.site',
				xfHost: 'owyx.site',
			}),
		)
		assert.equal(result.nextCalled, false)
		assert.equal(result.status, 401)
		assert.equal(result.body.error, 'unauthorized_client')
	})

	it('requires key when Host is spoofed to a non-site hostname', async () => {
		const result = await runGate(
			mockReq({
				host: 'evil.com',
				hostname: 'evil.com',
			}),
		)
		assert.equal(result.nextCalled, false)
		assert.equal(result.status, 401)
		assert.equal(result.body.error, 'unauthorized_client')
	})

	it('allows browser site host without key', async () => {
		const result = await runGate(
			mockReq({
				host: 'owyx.site',
				hostname: 'owyx.site',
			}),
		)
		assert.equal(result.nextCalled, true)
		assert.equal(result.status, 0)
	})

	it('allows public CSL paths on api host without key', async () => {
		const result = await runGate(
			mockReq({
				host: 'api.owyx.site',
				hostname: 'api.owyx.site',
				path: '/api/csl/skins/Steve.png',
			}),
		)
		assert.equal(result.nextCalled, true)
	})
})
