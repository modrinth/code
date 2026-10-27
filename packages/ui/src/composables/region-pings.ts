import { type Archon, WebSocketPinger } from '@modrinth/api-client'
import { readonly, ref } from 'vue'

const INITIAL_PING_COUNT = 5
const INITIAL_PING_INTERVAL = 200
const INITIAL_PING_SETTLE_TIME = 1000
const PING_CONNECT_TIMEOUT = 3000
const PING_REFRESH_INTERVAL = 2500
const PING_REFRESH_SETTLE_TIME = 2000
const PING_SAMPLE_WINDOW = 5

/**
 * Latency to each region in ms, averaged over its last pings and keyed by region shortcode. `-1` means pinging the
 * region failed.
 */
const regionPings = ref<Record<string, number>>({})
/** The latency of the last pings to each region over its open connection, keyed by region shortcode. */
const regionPingSamples = new Map<string, number[]>()
/** The open ping connection to each region, keyed by region shortcode. */
const regionPingers = new Map<string, WebSocketPinger>()
/** The ping URL that last connected for each region, keyed by region shortcode. */
const workingPingUrls = new Map<string, string>()

let pingedRegions: Archon.Servers.v1.RegionStock[] = []
let pingRun: Promise<void> | null = null
let initialPingsDone = false

let refreshCount = 0
let refreshController: AbortController | null = null
let refreshLoopsController: AbortController | null = null

function setRegionPing(region: string, ping: number) {
	regionPings.value = { ...regionPings.value, [region]: ping }
}

function closeRegionPinger(region: string, pinger: WebSocketPinger) {
	pinger.close()
	if (regionPingers.get(region) === pinger) regionPingers.delete(region)
}

function closeRegionPingers() {
	for (const [region, pinger] of [...regionPingers]) {
		closeRegionPinger(region, pinger)
	}
}

/**
 * Returns the region's open ping connection, or connects through its ping URLs one at a time, starting with the one
 * that last connected and falling back to the next URL only when one fails to connect.
 */
async function connectRegion(
	region: Archon.Servers.v1.RegionStock,
	signal?: AbortSignal,
): Promise<WebSocketPinger | null> {
	const existing = regionPingers.get(region.region)
	if (existing?.isOpen) return existing
	if (existing) closeRegionPinger(region.region, existing)

	const workingUrl = workingPingUrls.get(region.region)
	const urls = workingUrl
		? [workingUrl, ...region.ping_urls.filter((url) => url !== workingUrl)]
		: region.ping_urls

	for (const url of urls) {
		const pinger = new WebSocketPinger(url, PING_CONNECT_TIMEOUT)
		const connected = await pinger.connected
		if (signal?.aborted) {
			pinger.close()
			return null
		}
		if (connected) {
			workingPingUrls.set(region.region, url)
			regionPingers.set(region.region, pinger)
			return pinger
		}
		pinger.close()
	}
	return null
}

/**
 * Pings a region and returns its latency averaged over its last pings. A new connection is measured with several
 * pings, while an open one is measured with a single ping.
 */
async function pingRegion(
	region: Archon.Servers.v1.RegionStock,
	signal?: AbortSignal,
): Promise<number> {
	const reusingConnection = regionPingers.get(region.region)?.isOpen ?? false
	const pinger = await connectRegion(region, signal)
	if (!pinger) return -1

	const samples = await pinger.measure(
		reusingConnection
			? { count: 1, settleDelayMs: PING_REFRESH_SETTLE_TIME, signal }
			: {
					count: INITIAL_PING_COUNT,
					intervalMs: INITIAL_PING_INTERVAL,
					settleDelayMs: INITIAL_PING_SETTLE_TIME,
					signal,
				},
	)
	if (signal?.aborted) return -1
	if (samples.length === 0) {
		closeRegionPinger(region.region, pinger)
		regionPingSamples.delete(region.region)
		return -1
	}

	const previousSamples = reusingConnection ? (regionPingSamples.get(region.region) ?? []) : []
	const window = [...previousSamples, ...samples].slice(-PING_SAMPLE_WINDOW)
	regionPingSamples.set(region.region, window)
	return Math.round(window.reduce((sum, sample) => sum + sample, 0) / window.length)
}

function sleep(ms: number, signal: AbortSignal): Promise<void> {
	return new Promise((resolve) => {
		const onAbort = () => {
			clearTimeout(timeout)
			resolve()
		}
		const timeout = setTimeout(() => {
			signal.removeEventListener('abort', onAbort)
			resolve()
		}, ms)
		signal.addEventListener('abort', onAbort, { once: true })
	})
}

async function refreshRegionPing(region: Archon.Servers.v1.RegionStock, signal: AbortSignal) {
	while (!signal.aborted) {
		const startedAt = Date.now()
		const ping = await pingRegion(region, signal)
		if (signal.aborted) return
		setRegionPing(region.region, ping)
		await sleep(PING_REFRESH_INTERVAL - (Date.now() - startedAt), signal)
	}
}

/**
 * Starts refreshing every region's ping, once the initial pings are done and while a refresh is requested.
 */
function startRefreshLoops() {
	if (!initialPingsDone || !refreshController || refreshLoopsController === refreshController) {
		return
	}
	refreshLoopsController = refreshController
	for (const region of pingedRegions) {
		void refreshRegionPing(region, refreshController.signal)
	}
}

/**
 * Region latencies shared across the page, so every purchase modal reuses a single round of pings and a single
 * connection to each region.
 */
export function useRegionPings() {
	/**
	 * Starts pinging every region, unless a previous call already did.
	 */
	function startRegionPings(regions: Archon.Servers.v1.RegionStock[]) {
		if (pingRun || regions.length === 0) return

		pingedRegions = regions
		pingRun = Promise.all(
			regions.map(async (region) => setRegionPing(region.region, await pingRegion(region))),
		).then(() => {
			initialPingsDone = true
			if (refreshController) {
				startRefreshLoops()
			} else {
				closeRegionPingers()
			}
		})
	}

	/**
	 * Resolves once every region has been pinged, or after `timeoutMs`, whichever comes first.
	 */
	function waitForRegionPings(timeoutMs: number): Promise<void> {
		if (!pingRun) return Promise.resolve()
		return Promise.race([pingRun, new Promise<void>((resolve) => setTimeout(resolve, timeoutMs))])
	}

	/**
	 * Keeps re-pinging every region every `PING_REFRESH_INTERVAL` ms over its open connection once the initial pings
	 * are done, until the returned function is called.
	 */
	function startPingRefresh(): () => void {
		refreshCount++
		if (refreshCount === 1) {
			refreshController = new AbortController()
			startRefreshLoops()
		}

		let stopped = false
		return () => {
			if (stopped) return
			stopped = true
			refreshCount--
			if (refreshCount === 0) {
				refreshController?.abort()
				refreshController = null
				if (initialPingsDone) closeRegionPingers()
			}
		}
	}

	return {
		regionPings: readonly(regionPings),
		startRegionPings,
		waitForRegionPings,
		startPingRefresh,
	}
}
