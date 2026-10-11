import { requireUser, tokenFrom } from '../common/auth'
import { digest, randomId } from '../common/crypto'
import { ApiError, json, readJson, routePath, stringField, unauthorized } from '../common/http'
import type { Env } from '../common/types'

// Core pairing. A Core registers the code it is showing, an account claims that code in the App,
// and the Core then confirms the claim to receive its own key. After that the backend only keeps
// the account's list of Cores and their addresses.

const CODE_TTL_MS = 15 * 60_000
// Expired codes are kept this long so a late guess is told "expired" instead of "wrong".
const EXPIRED_GRACE_MS = 60 * 60_000
const MAX_WRONG_GUESSES = 5
const GUESS_WINDOW_MS = 10 * 60_000
// The Core's alphabet: no i, o, 0 or 1.
const CODE_PATTERN = /^[a-hj-np-z2-9]{8}$/

interface CodeRow {
	code: string
	core_id: string
	address: string
	expires: string
	claimed_by: string | null
}

interface CoreRow {
	id: string
	owner_id: string
	name: string
	address: string
	created: string
	last_seen: string
}

function isPrivateHost(hostname: string): boolean {
	const host = hostname.replace(/^\[|\]$/g, '').toLowerCase()
	if (host === 'localhost' || host === '::1') return true
	const ipv4 = host.match(/^(\d{1,3})\.(\d{1,3})\.\d{1,3}\.\d{1,3}$/)
	if (ipv4) {
		const [first, second] = [Number(ipv4[1]), Number(ipv4[2])]
		return (
			first === 127 ||
			first === 10 ||
			(first === 172 && second >= 16 && second <= 31) ||
			(first === 192 && second === 168) ||
			(first === 169 && second === 254)
		)
	}
	// IPv6 unique local (fc00::/7) and link local (fe80::/10).
	return host.includes(':') && /^(f[cd]|fe[89ab])/.test(host)
}

/** A Core address must be HTTPS unless it is loopback or on a private network. */
function coreAddress(body: Record<string, unknown>): string {
	const value = stringField(body, 'address', 512)
	let url: URL
	try {
		url = new URL(value)
	} catch {
		throw new ApiError(400, 'invalid_address', 'The Core address is not a valid URL')
	}
	const allowed =
		url.protocol === 'https:' || (url.protocol === 'http:' && isPrivateHost(url.hostname))
	if (!allowed || url.username || url.password)
		throw new ApiError(
			400,
			'invalid_address',
			'A Core address must use HTTPS unless it is on this computer or a private network',
		)
	return url.origin + url.pathname.replace(/\/$/, '')
}

function pairingCode(body: Record<string, unknown>): string {
	return stringField(body, 'code', 32)
		.replace(/[^a-z0-9]/gi, '')
		.toLowerCase()
}

function publicCore(row: CoreRow) {
	return {
		id: row.id,
		name: row.name,
		address: row.address,
		created: row.created,
		last_seen: row.last_seen,
	}
}

/** Core, unowned: publish the code it is showing. */
async function registerCode(request: Request, env: Env) {
	const body = await readJson(request)
	const code = pairingCode(body)
	if (!CODE_PATTERN.test(code)) throw new ApiError(400, 'invalid_input', 'Invalid code')
	const coreId = stringField(body, 'core_id', 64)
	const address = coreAddress(body)
	const now = Date.now()
	const nowIso = new Date(now).toISOString()

	await env.DB.prepare('DELETE FROM core_pairing_codes WHERE expires < ?')
		.bind(new Date(now - EXPIRED_GRACE_MS).toISOString())
		.run()
	if (await env.DB.prepare('SELECT 1 FROM cores WHERE id = ?').bind(coreId).first())
		throw new ApiError(409, 'already_paired', 'This Core is already paired to an account')
	// A Core may replace its own code. Another Core's live code is never overwritten.
	const taken = await env.DB.prepare(
		'SELECT 1 FROM core_pairing_codes WHERE code = ? AND core_id != ? AND expires > ?',
	)
		.bind(code, coreId, nowIso)
		.first()
	if (taken) throw new ApiError(409, 'code_taken', 'This code is in use')
	const expires = new Date(now + CODE_TTL_MS).toISOString()
	await env.DB.batch([
		env.DB.prepare('DELETE FROM core_pairing_codes WHERE code = ? OR core_id = ?').bind(
			code,
			coreId,
		),
		env.DB.prepare(
			'INSERT INTO core_pairing_codes (code,core_id,address,expires) VALUES (?,?,?,?)',
		).bind(code, coreId, address, expires),
	])
	return json({ expires })
}

/** App: claim the code a Core is showing. Returns where that Core is so the App can finish there. */
async function claimCode(request: Request, env: Env) {
	const user = await requireUser(request, env)
	const code = pairingCode(await readJson(request))
	const now = Date.now()
	const nowIso = new Date(now).toISOString()

	const attempts = await env.DB.prepare(
		'SELECT failures,window_start FROM core_pairing_attempts WHERE user_id = ?',
	)
		.bind(user.id)
		.first<{ failures: number; window_start: string }>()
	const windowEnds = attempts ? Date.parse(attempts.window_start) + GUESS_WINDOW_MS : 0
	const failures = attempts && windowEnds > now ? attempts.failures : 0
	if (failures >= MAX_WRONG_GUESSES)
		throw new ApiError(
			429,
			'too_many_attempts',
			`Too many wrong codes. Try again in ${Math.ceil((windowEnds - now) / 60_000)} minutes.`,
		)

	const row = await env.DB.prepare('SELECT * FROM core_pairing_codes WHERE code = ?')
		.bind(code)
		.first<CodeRow>()
	if (!row) {
		await env.DB.prepare(
			'INSERT INTO core_pairing_attempts (user_id,failures,window_start) VALUES (?,1,?) ON CONFLICT(user_id) DO UPDATE SET failures = ?, window_start = ?',
		)
			.bind(user.id, nowIso, failures + 1, failures ? (attempts?.window_start ?? nowIso) : nowIso)
			.run()
		throw new ApiError(404, 'wrong_code', 'That code does not match any Core')
	}
	if (row.expires <= nowIso)
		throw new ApiError(410, 'code_expired', 'That code has expired. Use the new one on your Core.')
	const claimed = await env.DB.prepare(
		'UPDATE core_pairing_codes SET claimed_by = ? WHERE code = ? AND (claimed_by IS NULL OR claimed_by = ?) RETURNING code',
	)
		.bind(user.id, code, user.id)
		.first()
	if (!claimed)
		throw new ApiError(409, 'already_paired', 'That Core is being paired to another account')
	return json({ core_id: row.core_id, address: row.address })
}

/**
 * Core: check that `user_id` claimed this Core's code. On success the Core joins that account's
 * list and gets the key it uses from now on. Only the key's hash is stored.
 */
async function confirmClaim(request: Request, env: Env) {
	const body = await readJson(request)
	const code = pairingCode(body)
	const coreId = stringField(body, 'core_id', 64)
	const userId = stringField(body, 'user_id', 64)
	const name = stringField(body, 'name', 64)
	const address = coreAddress(body)
	const now = new Date().toISOString()

	const row = await env.DB.prepare(
		'SELECT * FROM core_pairing_codes WHERE code = ? AND core_id = ? AND expires > ?',
	)
		.bind(code, coreId, now)
		.first<CodeRow>()
	if (!row || row.claimed_by !== userId)
		throw new ApiError(403, 'not_claimed', 'This code was not claimed by that account')

	const key = `amc_${randomId(48)}`
	// Repeating a confirmation whose answer was lost issues a fresh key to the same owner. A Core
	// that belongs to someone else is never taken over.
	const stored = await env.DB.prepare(
		'INSERT INTO cores (id,owner_id,name,address,key_hash,created,last_seen) VALUES (?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET key_hash = excluded.key_hash, address = excluded.address, last_seen = excluded.last_seen WHERE owner_id = excluded.owner_id RETURNING id',
	)
		.bind(coreId, userId, name, address, await digest(key), now, now)
		.first()
	if (!stored)
		throw new ApiError(409, 'already_paired', 'This Core is already paired to another account')
	return json({ key })
}

/** Core, paired: report where it is now. Called every time the Core starts. */
async function reportAddress(request: Request, env: Env) {
	const key = tokenFrom(request)
	if (!key.startsWith('amc_')) unauthorized()
	const address = coreAddress(await readJson(request))
	const row = await env.DB.prepare(
		'UPDATE cores SET address = ?, last_seen = ? WHERE key_hash = ? RETURNING *',
	)
		.bind(address, new Date().toISOString(), await digest(key))
		.first<CoreRow>()
	if (!row) unauthorized()
	return json({ id: row.id, owner_id: row.owner_id })
}

export async function handleCores(request: Request, env: Env): Promise<Response | null> {
	const path = routePath(request),
		method = request.method
	if (path === '/core/pairing' && method === 'POST') return registerCode(request, env)
	if (path === '/core/pairing/confirm' && method === 'POST') return confirmClaim(request, env)
	if (path === '/core/address' && method === 'POST') return reportAddress(request, env)
	if (path === '/cores/pair' && method === 'POST') return claimCode(request, env)
	if (path === '/cores' && method === 'GET') {
		const user = await requireUser(request, env)
		const rows = await env.DB.prepare('SELECT * FROM cores WHERE owner_id = ? ORDER BY created')
			.bind(user.id)
			.all<CoreRow>()
		return json(rows.results.map(publicCore))
	}
	const core = path.match(/^\/cores\/([^/]+)$/)
	if (core && method === 'DELETE') {
		const user = await requireUser(request, env)
		const deleted = await env.DB.prepare('DELETE FROM cores WHERE id = ? AND owner_id = ?')
			.bind(decodeURIComponent(core[1]), user.id)
			.run()
		if (!deleted.meta.changes) throw new ApiError(404, 'not_found', 'Core not found')
		return new Response(null, { status: 204 })
	}
	return null
}
