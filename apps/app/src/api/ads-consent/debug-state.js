const AD_REPORT_MAX_URL_LENGTH = 2000
const AD_REPORT_MAX_FRAMES = 100
const AD_REPORT_FRAME_RESPONSE_WINDOW = 500

/**
 * @param {unknown} value
 * @returns {string | null}
 */
function truncateUrl(value) {
	if (typeof value !== 'string' || value.length === 0) return null
	return value.length > AD_REPORT_MAX_URL_LENGTH
		? `${value.slice(0, AD_REPORT_MAX_URL_LENGTH)}…`
		: value
}

/**
 * @template T
 * @param {() => T} fn
 * @returns {T | null}
 */
function tryCollect(fn) {
	try {
		return fn() ?? null
	} catch {
		return null
	}
}

/**
 * @param {Record<string, unknown>} value
 * @returns {Record<string, unknown>}
 */
function compact(value) {
	return Object.fromEntries(
		Object.entries(value).filter(
			([, entry]) => entry != null && !(Array.isArray(entry) && entry.length === 0),
		),
	)
}

/** @returns {any[]} */
function getPrebidWinningBids() {
	const names = Array.isArray(window._pbjsGlobals) ? window._pbjsGlobals : ['pbjs']
	return names.flatMap((name) => tryCollect(() => window[name].getAllWinningBids()) ?? [])
}

/**
 * Finds the most recently rendered Prebid winning bid. Aditude Serve renders Prebid bids directly,
 * so this is the ad on screen unless an Amazon or direct deal has replaced it since.
 *
 * @returns {any | null}
 */
function findRenderedPrebidBid() {
	return (
		getPrebidWinningBids()
			.filter((bid) => bid.status === 'rendered')
			.sort((a, b) => (b.responseTimestamp ?? 0) - (a.responseTimestamp ?? 0))[0] ?? null
	)
}

/**
 * Answers a request from the top frame with this frame's outbound links, which lets the top
 * frame find click-through URLs inside cross-origin ad creatives.
 *
 * @param {MessageEvent} event
 * @returns {void}
 */
function handleAdsDebugFrameRequest(event) {
	if (!event.data?.modrinthAdsDebugRequest || event.source !== window.top) return

	const links = Array.from(document.querySelectorAll('a[href]'), (link) => link.href).filter(
		(href) => tryCollect(() => !new URL(href).hostname.endsWith('modrinth.com')),
	)

	window.top.postMessage({ modrinthAdsDebugFrame: { links } }, event.origin)
}

/**
 * @param {Window} root
 * @param {Window[]} frames
 * @returns {Window[]}
 */
function collectFrameWindows(root, frames = []) {
	for (let i = 0; i < root.frames.length && frames.length < AD_REPORT_MAX_FRAMES; i++) {
		frames.push(root.frames[i])
		collectFrameWindows(root.frames[i], frames)
	}
	return frames
}

/** @returns {Promise<string | null>} */
function collectClickUrl() {
	return new Promise((resolve) => {
		/** @param {string | null} clickUrl */
		const finish = (clickUrl) => {
			clearTimeout(timeout)
			window.removeEventListener('message', handleResponse)
			resolve(clickUrl)
		}

		/** @param {MessageEvent} event */
		const handleResponse = (event) => {
			const links = event.data?.modrinthAdsDebugFrame?.links
			const clickUrl = Array.isArray(links) ? truncateUrl(links[0]) : null
			if (clickUrl) finish(clickUrl)
		}

		const timeout = setTimeout(() => finish(null), AD_REPORT_FRAME_RESPONSE_WINDOW)
		window.addEventListener('message', handleResponse)

		for (const frame of tryCollect(() => collectFrameWindows(window)) ?? []) {
			try {
				frame.postMessage({ modrinthAdsDebugRequest: true }, '*')
			} catch {}
		}
	})
}

/**
 * Collects what's needed to block the ad on screen: the advertiser domain and creative declared
 * by the winning Prebid bid, and the click-through URL for ads that don't declare one (e.g.
 * Amazon TAM or direct deals).
 *
 * @returns {Promise<Record<string, unknown>>}
 */
async function collectAdReport() {
	const bid = findRenderedPrebidBid()
	const clickUrl = await collectClickUrl()
	const advertiserDomains = (bid?.meta?.advertiserDomains ?? []).filter(
		(domain) => typeof domain === 'string' && domain.length > 0,
	)

	return compact({
		current_ad: advertiserDomains[0] ?? clickUrl,
		captured_at: new Date().toISOString(),
		advertiser_domains: advertiserDomains,
		advertiser_name: bid?.meta?.advertiserName,
		brand_name: bid?.meta?.brandName,
		bidder: bid?.bidderCode ?? bid?.bidder,
		network_name: bid?.meta?.networkName,
		creative_id: bid?.creativeId,
		deal_id: bid?.dealId,
		click_url: clickUrl,
	})
}

/** @returns {Promise<void>} */
async function reportAdsDebugState() {
	const invoke = getTauriInvoke()
	if (!invoke) return

	let debugState
	try {
		debugState = await collectAdReport()
	} catch (error) {
		debugState = { error: String(error) }
	}

	try {
		await invoke('plugin:ads|report_ads_debug_state', { debugState })
	} catch {}
}
