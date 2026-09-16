/**
 * Security gate unit tests (Node, no Express).
 * Run: node --test owyxsite/backend/src/middleware/clientKey.test.js owyxsite/backend/src/utils/turnstile.test.js
 */

const { describe, it } = require('node:test')
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

describe('requestHost', () => {
	it('ignores spoofed X-Forwarded-Host', () => {
		const host = requestHost(
			mockReq({ host: 'api.owyx.site', hostname: 'api.owyx.site', xfHost: 'owyx.site' }),
		)
		assert.equal(host, 'api.owyx.site')
	})
})

describe('clientKeyGate', () => {
	it('requires key on api host even with XFH spoof', async () => {
		process.env.API_HOSTS = 'api.owyx.site'
		process.env.LAUNCHER_CLIENT_KEY = 'test-key-value'
		process.env.NODE_ENV = 'production'
		const req = mockReq({
			host: 'api.owyx.site',
			hostname: 'api.owyx.site',
			xfHost: 'owyx.site',
		})
		let status = 0
		let body = null
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
		let nextCalled = false
		await new Promise((resolve) => {
			clientKeyGate(req, res, () => {
				nextCalled = true
				resolve()
			})
			// if gate responded, resolve soon
			setTimeout(resolve, 20)
		})
		assert.equal(nextCalled, false)
		assert.equal(status, 401)
		assert.equal(body.error, 'unauthorized_client')
	})
})
