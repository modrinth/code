import { Reader } from '@zip.js/zip.js'

/**
 * A zip.js Reader for remote archives that:
 *  1. Fetches the tail (EOCD + central directory) first, so getEntries() resolves after 1–2 requests,
 *  2. Serves on-demand reads (entry.getData) with priority,
 *  3. Downloads the rest of the file in the background.
 *
 * Overall the given zip is cached in fixed-size blocks, any tiny reads
 * (30-byte local headers, names, extra fields) become a few block-sized requests, and only unique
 * data is saved to the cache.
 */

// ---------------------------------------------------------------------------------------------
// Block cache
// ---------------------------------------------------------------------------------------------

/**
 * Fixed-size block store. Block i covers [i*blockSize, min((i+1)*blockSize, fileSize)).
 * Blocks are views into response buffers: inserting never copies, reading copies only when a read
 * spans more than one block.
 */
export class BlockCache {
	readonly blockCount: number
	private readonly blocks: (Uint8Array | undefined)[]
	private cached = 0

	constructor(
		readonly fileSize: number,
		readonly blockSize: number,
	) {
		this.blockCount = Math.ceil(fileSize / blockSize)
		this.blocks = new Array(this.blockCount)
	}

	/** Total number of bytes cached. */
	get size(): number {
		return this.cached
	}

	blockOf(offset: number): number {
		return Math.floor(offset / this.blockSize)
	}

	blockStart(i: number): number {
		return i * this.blockSize
	}

	blockEnd(i: number): number {
		return Math.min((i + 1) * this.blockSize, this.fileSize)
	}

	has(i: number): boolean {
		return this.blocks[i] !== undefined
	}

	/**
	 * Stores `bytes`, which start at the beginning of block `first` and cover whole blocks (the last
	 * one may end at EOF). Already-cached blocks are left as they are.
	 */
	insertBlocks(first: number, bytes: Uint8Array): void {
		let pos = 0
		for (let i = first; pos < bytes.length; i++) {
			const len = this.blockEnd(i) - this.blockStart(i)
			if (pos + len > bytes.length)
				throw new RangeError(`Partial block ${i}: ${bytes.length - pos} of ${len} bytes`)
			if (!this.blocks[i]) {
				this.blocks[i] = bytes.subarray(pos, pos + len)
				this.cached += len
			}
			pos += len
		}
	}

	/** Indices of blocks in [first, last] that are not cached. */
	missing(first: number, last: number): number[] {
		const out: number[] = []
		for (let i = first; i <= last; i++) if (!this.blocks[i]) out.push(i)
		return out
	}

	/** Bytes [start, end) if fully cached, else undefined. Zero-copy within a single block. */
	read(start: number, end: number): Uint8Array | undefined {
		if (end <= start) return new Uint8Array(0)
		const first = this.blockOf(start)
		const last = this.blockOf(end - 1)
		if (first === last) {
			const b = this.blocks[first]
			if (!b) return undefined
			const base = this.blockStart(first)
			return b.subarray(start - base, end - base)
		}
		for (let i = first; i <= last; i++) if (!this.blocks[i]) return undefined
		const out = new Uint8Array(end - start)
		for (let i = first; i <= last; i++) {
			const b = this.blocks[i]!
			const base = this.blockStart(i)
			const from = Math.max(start, base) - base
			const to = Math.min(end, base + b.length) - base
			out.set(b.subarray(from, to), base + from - start)
		}
		return out
	}

	/** All blocks in order, for building a Blob without concatenating. Undefined if incomplete. */
	allBlocks(): Uint8Array[] | undefined {
		return this.cached === this.fileSize ? (this.blocks as Uint8Array[]) : undefined
	}
}

// ---------------------------------------------------------------------------------------------
// Request limiter with priority (foreground reads before background fill)
// ---------------------------------------------------------------------------------------------

class Limiter {
	private active = 0
	private readonly high: (() => void)[] = []
	private readonly low: (() => void)[] = []

	constructor(private readonly max: number) {}

	async run<T>(priority: 'high' | 'low', task: () => Promise<T>): Promise<T> {
		if (this.active >= this.max) {
			await new Promise<void>((resolve) =>
				(priority === 'high' ? this.high : this.low).push(resolve),
			)
		} else {
			this.active++
		}
		try {
			return await task()
		} finally {
			const next = this.high.shift() ?? this.low.shift()
			if (next)
				next() // hand the slot over directly
			else this.active--
		}
	}
}

// ---------------------------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------------------------

export class RangeReaderError extends Error {
	override name = 'RangeReaderError'
}
/** The remote file changed between requests (If-Range mismatch, validator change, size change). */
export class FileChangedError extends RangeReaderError {
	override name = 'FileChangedError'
}

// ---------------------------------------------------------------------------------------------
// Central directory location (EOCD / Zip64)
// ---------------------------------------------------------------------------------------------

export interface CentralDirectoryLocation {
	offset: number
	size: number
	entryCount: number
	zip64: boolean
}

const SIG_EOCD = 0x06054b50
const SIG_ZIP64_LOCATOR = 0x07064b50
const SIG_ZIP64_EOCD = 0x06064b50
const EOCD_MIN = 22
const MAX_COMMENT = 0xffff
const ZIP64_LOCATOR_SIZE = 20
const ZIP64_EOCD_MIN = 56

function u64(view: DataView, at: number): number {
	const v = view.getBigUint64(at, true)
	if (v > BigInt(Number.MAX_SAFE_INTEGER)) throw new RangeReaderError('Zip64 value exceeds 2^53')
	return Number(v)
}

/** Finds the EOCD inside `tail` (the last bytes of the file, starting at absolute `tailStart`). */
function findEocd(tail: Uint8Array, tailStart: number, fileSize: number) {
	const view = new DataView(tail.buffer, tail.byteOffset, tail.byteLength)
	for (let i = tail.length - EOCD_MIN; i >= 0; i--) {
		if (view.getUint32(i, true) !== SIG_EOCD) continue
		const commentLength = view.getUint16(i + 20, true)
		// The comment must run exactly to EOF; guards against the signature appearing inside the comment.
		if (tailStart + i + EOCD_MIN + commentLength !== fileSize) continue
		return {
			absOffset: tailStart + i,
			entryCount: view.getUint16(i + 10, true),
			cdSize: view.getUint32(i + 12, true),
			cdOffset: view.getUint32(i + 16, true),
		}
	}
	return undefined
}

// ---------------------------------------------------------------------------------------------
// Reader
// ---------------------------------------------------------------------------------------------

export interface RangeCacheReaderOptions {
	/**
	 * Known total size in bytes. Skips the initial HEAD, so the tree needs a single request.
	 */
	size?: number
	/**
	 * Cache/request granularity. Every request covers whole blocks, so this is the minimum request
	 * size. Default 128 KiB.
	 */
	blockSize?: number
	/**
	 * Bytes requested from the end of the file first. Default 256 KiB (covers EOCD + typical zip CD).
	 */
	tailSize?: number
	/**
	 *  Max size of one background-fill request. Default 4 MiB.
	 */
	backgroundRequestSize?: number
	/**
	 * Max size of one on-demand request (long missing runs are split). Default 8 MiB.
	 */
	maxRequestSize?: number
	/**
	 * Max concurrent requests. On-demand reads are queued ahead of background fill. Default 4.
	 */
	maxConcurrentRequests?: number
	/**
	 * Send If-Range so a changed file is detected server-side. Default true. If-Range is not a
	 * CORS-safelisted header, so it triggers a preflight. The Last-Modified/ETag comparison on
	 * every response still catches a replaced file.
	 */
	validate?: boolean
	/**
	 * Extra fetch options (credentials, mode, ...). Headers given here are merged.
	 */
	fetchInit?: RequestInit
	/**
	 * Called after each response is cached. Throttle on your side if it drives UI.
	 */
	onProgress?: (cached: number, total: number) => void
}

interface Validator {
	header: 'etag' | 'last-modified'
	value: string
}

export class RangeCacheReader extends Reader<string> {
	readonly url: string
	private readonly opts: Required<
		Omit<RangeCacheReaderOptions, 'onProgress' | 'fetchInit' | 'size'>
	> &
		Pick<RangeCacheReaderOptions, 'onProgress' | 'fetchInit' | 'size'>

	private cache!: BlockCache
	/** Pending block downloads, by block index, for de-duplication. */
	private readonly pending = new Map<number, Promise<void>>()
	private readonly limiter: Limiter
	private readonly controllers = new Set<AbortController>()
	private aborted: { reason: unknown } | undefined
	private validator: Validator | undefined
	private initPromise: Promise<void> | undefined
	private backgroundPromise: Promise<void> | undefined

	// Foreground reads pause the background fill between requests.
	private foregroundCount = 0
	private foregroundIdle: Promise<void> = Promise.resolve()
	private releaseIdle: () => void = () => {}

	/** False if the server ignored Range and the whole file was downloaded in one response. */
	rangesSupported = true
	/** Location of the central directory, available after init(). */
	centralDirectory!: CentralDirectoryLocation

	constructor(url: string, options: RangeCacheReaderOptions = {}) {
		super(url)
		this.url = url
		this.opts = {
			size: options.size,
			blockSize: options.blockSize ?? 128 * 1024,
			// Never smaller than the EOCD search window + Zip64 locator, or locating the CD costs extra requests.
			tailSize: Math.max(
				options.tailSize ?? 256 * 1024,
				EOCD_MIN + MAX_COMMENT + ZIP64_LOCATOR_SIZE,
			),
			backgroundRequestSize: options.backgroundRequestSize ?? 4 * 1024 * 1024,
			maxRequestSize: options.maxRequestSize ?? 8 * 1024 * 1024,
			maxConcurrentRequests: options.maxConcurrentRequests ?? 4,
			validate: options.validate ?? true,
			fetchInit: options.fetchInit,
			onProgress: options.onProgress,
		}
		this.limiter = new Limiter(this.opts.maxConcurrentRequests)
	}

	/** Idempotent: zip.js calls init() itself, and callers may call it earlier. */
	override init(): Promise<void> {
		this.initPromise ??= this.doInit()
		return this.initPromise
	}

	private async doInit(): Promise<void> {
		// 1. Total size, via HEAD unless given. Content-Length/Last-Modified are CORS-safelisted.
		let size = this.opts.size
		if (size === undefined) {
			const head = await this.exchange({}, 'HEAD')
			const len = Number(head.headers.get('content-length') ?? NaN)
			if (head.ok && Number.isSafeInteger(len) && len >= 0) {
				size = len
				this.validator = pickValidator(head.headers)
			}
		}
		if (size === undefined) return this.initFromFullDownload() // HEAD unusable: just download

		this.size = size
		this.cache = new BlockCache(size, this.opts.blockSize)

		// 2. Tail, aligned down to a block boundary, via an explicit range (never `bytes=-N`).
		const firstBlock = this.cache.blockOf(Math.max(0, size - this.opts.tailSize))
		const tailStart = this.cache.blockStart(firstBlock)
		const headers: Record<string, string> = { Range: `bytes=${tailStart}-${size - 1}` }
		if (this.opts.validate && this.validator) headers['If-Range'] = this.validator.value
		const res = await this.exchange(headers)

		// 200 = server ignored Range, or the file changed since HEAD. Either way the body is the
		// current, complete file, so use it.
		if (res.status === 200) return this.initFromBytes(res.body!)
		if (res.status !== 206)
			throw new RangeReaderError(`Unexpected status ${res.status} for ${this.url}`)

		this.validator ??= pickValidator(res.headers)
		this.assertSameFile(res.headers)
		this.assertRange(res.headers, tailStart, size, res.body!.length)
		this.cache.insertBlocks(firstBlock, res.body!)
		this.report()

		this.centralDirectory = await this.locateCentralDirectory()
		// Make sure the whole CD is cached so getEntries() never waits on the network again.
		const cd = this.centralDirectory
		await this.ensure(cd.offset, cd.offset + cd.size, 'high')
	}

	private async initFromFullDownload(): Promise<void> {
		const res = await this.exchange({})
		if (!res.ok) throw new RangeReaderError(`Unexpected status ${res.status} for ${this.url}`)
		return this.initFromBytes(res.body!)
	}

	/** Uses a complete-file body as the whole cache; no further requests are made. */
	private async initFromBytes(bytes: Uint8Array): Promise<void> {
		this.rangesSupported = false
		this.size = bytes.length
		this.cache = new BlockCache(bytes.length, this.opts.blockSize)
		this.cache.insertBlocks(0, bytes)
		this.report()
		this.centralDirectory = await this.locateCentralDirectory()
	}

	private async locateCentralDirectory(): Promise<CentralDirectoryLocation> {
		const searchLen = Math.min(this.size, EOCD_MIN + MAX_COMMENT)
		const tailStart = this.size - searchLen
		const tail = await this.bytes(tailStart, this.size)
		const eocd = findEocd(tail, tailStart, this.size)
		if (!eocd) throw new RangeReaderError('End of central directory record not found (not a zip?)')

		const needsZip64 =
			eocd.cdOffset === 0xffffffff || eocd.cdSize === 0xffffffff || eocd.entryCount === 0xffff
		const locatorAt = eocd.absOffset - ZIP64_LOCATOR_SIZE
		if (locatorAt >= 0) {
			const loc = await this.bytes(locatorAt, eocd.absOffset)
			const lv = new DataView(loc.buffer, loc.byteOffset, loc.byteLength)
			if (lv.getUint32(0, true) === SIG_ZIP64_LOCATOR) {
				const z64At = u64(lv, 8)
				const rec = await this.bytes(z64At, z64At + ZIP64_EOCD_MIN)
				const rv = new DataView(rec.buffer, rec.byteOffset, rec.byteLength)
				if (rv.getUint32(0, true) !== SIG_ZIP64_EOCD)
					throw new RangeReaderError('Bad Zip64 EOCD record')
				return { entryCount: u64(rv, 32), size: u64(rv, 40), offset: u64(rv, 48), zip64: true }
			}
		}
		if (needsZip64) throw new RangeReaderError('Zip64 markers present but no Zip64 EOCD locator')
		return { offset: eocd.cdOffset, size: eocd.cdSize, entryCount: eocd.entryCount, zip64: false }
	}

	override async readUint8Array(index: number, length: number): Promise<Uint8Array> {
		await this.init()
		const end = Math.min(index + length, this.size) // zip.js contract: truncate at EOF
		const hit = this.cache.read(index, end)
		if (hit) return hit

		this.beginForeground()
		try {
			return await this.bytes(index, end)
		} finally {
			this.endForeground()
		}
	}

	/**
	 * Starts (or returns) the background download of everything not yet cached.
	 * Resolves when the whole file is cached. Safe to call multiple times.
	 */
	fillBackground(): Promise<void> {
		this.backgroundPromise ??= (async () => {
			await this.init()
			const step = Math.max(1, Math.floor(this.opts.backgroundRequestSize / this.opts.blockSize))
			for (let b = 0; b < this.cache.blockCount; b += step) {
				await this.foregroundIdle // yield to on-demand reads
				this.throwIfAborted()
				const last = Math.min(b + step, this.cache.blockCount) - 1
				await this.ensureBlocks(b, last, 'low', step)
			}
		})()
		return this.backgroundPromise
	}

	/** Whole file as a Blob once fully cached (starts the background fill if needed). No big copy. */
	async toBlob(type = 'application/java-archive'): Promise<Blob> {
		await this.fillBackground()
		return new Blob(this.cache.allBlocks()! as Uint8Array<ArrayBuffer>[], { type })
	}

	/** Cancels all in-flight requests; pending and future reads reject. */
	abort(reason: unknown = new DOMException('Aborted', 'AbortError')): void {
		this.aborted = { reason }
		for (const c of this.controllers) c.abort(reason)
	}

	get cachedBytes(): number {
		return this.cache?.size ?? 0
	}

	// -------------------------------------------------------------------------------------------
	// Internals
	// -------------------------------------------------------------------------------------------

	private async bytes(start: number, end: number): Promise<Uint8Array> {
		await this.ensure(start, end, 'high')
		return this.cache.read(start, end)!
	}

	private ensure(start: number, end: number, priority: 'high' | 'low'): Promise<void> {
		if (end <= start) return Promise.resolve()
		const maxBlocks = Math.max(1, Math.floor(this.opts.maxRequestSize / this.opts.blockSize))
		return this.ensureBlocks(
			this.cache.blockOf(start),
			this.cache.blockOf(end - 1),
			priority,
			maxBlocks,
		)
	}

	/**
	 * Ensures blocks [first, last] are cached. Blocks already downloading are awaited; the rest are
	 * grouped into contiguous runs of at most `maxBlocks` and fetched, one request per run.
	 */
	private async ensureBlocks(
		first: number,
		last: number,
		priority: 'high' | 'low',
		maxBlocks: number,
	): Promise<void> {
		this.throwIfAborted()
		const waits: Promise<void>[] = []
		let runStart = -1
		let runEnd = -1 // inclusive
		const flush = () => {
			for (let s = runStart; s <= runEnd; s += maxBlocks) {
				waits.push(this.fetchBlocks(s, Math.min(s + maxBlocks - 1, runEnd), priority))
			}
			runStart = runEnd = -1
		}
		for (const i of this.cache.missing(first, last)) {
			const p = this.pending.get(i)
			if (p) {
				if (runStart !== -1) flush()
				if (!waits.includes(p)) waits.push(p)
				continue
			}
			if (runStart !== -1 && i !== runEnd + 1) flush()
			if (runStart === -1) runStart = i
			runEnd = i
		}
		if (runStart !== -1) flush()
		await Promise.all(waits)
	}

	/** Fetches blocks [first, last] in one request and caches them. */
	private fetchBlocks(first: number, last: number, priority: 'high' | 'low'): Promise<void> {
		if (!this.rangesSupported) {
			return Promise.reject(new RangeReaderError('Range requests unsupported and bytes not cached'))
		}
		const start = this.cache.blockStart(first)
		const end = this.cache.blockEnd(last)
		const promise = this.limiter
			.run(priority, async () => {
				this.throwIfAborted()
				const headers: Record<string, string> = { Range: `bytes=${start}-${end - 1}` }
				if (this.opts.validate && this.validator) headers['If-Range'] = this.validator.value
				// Don't download a full body we'd throw away.
				const res = await this.exchange(headers, 'GET', (status) => status !== 206)

				if (res.status === 200) {
					throw new FileChangedError(
						`Server returned the full file for a range request; ${this.url} changed?`,
					)
				}
				if (res.status !== 206) throw new RangeReaderError(`Unexpected status ${res.status}`)
				// Backstop for servers/CDNs that ignore If-Range: compare the validator ourselves.
				this.assertSameFile(res.headers)
				this.assertRange(res.headers, start, end, res.body!.length)
				this.cache.insertBlocks(first, res.body!)
				this.report()
			})
			.finally(() => {
				for (let i = first; i <= last; i++)
					if (this.pending.get(i) === promise) this.pending.delete(i)
			})
		for (let i = first; i <= last; i++) this.pending.set(i, promise)
		return promise
	}

	/**
	 * Performs one request and reads its body. Each request gets its own AbortController (tracked so
	 * abort() reaches it) instead of sharing one signal, so listeners never pile up.
	 */
	private async exchange(
		extraHeaders: Record<string, string>,
		method = 'GET',
		skipBody: (status: number) => boolean = () => false,
	): Promise<{ status: number; ok: boolean; headers: Headers; body?: Uint8Array }> {
		this.throwIfAborted()
		const init = this.opts.fetchInit ?? {}
		const headers = new Headers(init.headers)
		for (const [k, v] of Object.entries(extraHeaders)) headers.set(k, v)
		const controller = new AbortController()
		this.controllers.add(controller)
		try {
			const res = await fetch(this.url, { ...init, method, headers, signal: controller.signal })
			let body: Uint8Array | undefined
			if (method === 'HEAD' || skipBody(res.status)) await res.body?.cancel()
			else body = new Uint8Array(await res.arrayBuffer())
			return { status: res.status, ok: res.ok, headers: res.headers, body }
		} finally {
			this.controllers.delete(controller)
		}
	}

	/** Throws FileChangedError if the response's validator differs from the one we started with. */
	private assertSameFile(headers: Headers): void {
		if (!this.validator) return
		const now = headers.get(this.validator.header)
		if (now !== null && now !== this.validator.value) {
			throw new FileChangedError(
				`${this.validator.header} changed (${this.validator.value} → ${now}); ${this.url} was replaced`,
			)
		}
	}

	/**
	 * Verifies a 206 body covers [start, end). Content-Range is checked when readable (cross-origin it
	 * is only readable with Access-Control-Expose-Headers); otherwise the exact body length is required.
	 */
	private assertRange(headers: Headers, start: number, end: number, bodyLength: number): void {
		const range = parseContentRange(headers.get('content-range'))
		if (range) {
			if (range.total !== this.size)
				throw new FileChangedError(`File size changed (${this.size} → ${range.total})`)
			if (range.start !== start || range.end !== end) {
				throw new RangeReaderError(
					`Server returned bytes ${range.start}-${range.end - 1}, wanted ${start}-${end - 1}`,
				)
			}
		}
		if (bodyLength !== end - start)
			throw new RangeReaderError(`Got ${bodyLength} bytes, wanted ${end - start}`)
	}

	private beginForeground(): void {
		if (this.foregroundCount++ === 0) {
			this.foregroundIdle = new Promise((resolve) => (this.releaseIdle = resolve))
		}
	}

	private endForeground(): void {
		if (--this.foregroundCount === 0) this.releaseIdle()
	}

	private throwIfAborted(): void {
		if (this.aborted) throw this.aborted.reason
	}

	private report(): void {
		this.opts.onProgress?.(this.cache.size, this.size)
	}
}

// ---------------------------------------------------------------------------------------------
// Header helpers
// ---------------------------------------------------------------------------------------------

/** Parses `bytes start-end/total` into a half-open range. */
function parseContentRange(
	value: string | null,
): { start: number; end: number; total: number } | undefined {
	const m = value?.match(/^bytes (\d+)-(\d+)\/(\d+)$/)
	if (!m) return undefined
	return { start: Number(m[1]), end: Number(m[2]) + 1, total: Number(m[3]) }
}

/**
 * If-Range requires a strong ETag; weak ones are ignored by servers, so fall back to Last-Modified.
 * Cross-origin, ETag is only readable with Access-Control-Expose-Headers; Last-Modified always is.
 */
function pickValidator(headers: Headers): Validator | undefined {
	const etag = headers.get('etag')
	if (etag && !etag.startsWith('W/')) return { header: 'etag', value: etag }
	const lm = headers.get('last-modified')
	return lm ? { header: 'last-modified', value: lm } : undefined
}
