const DEBUG_STATE_MAX_STRING_LENGTH = 2000
const DEBUG_STATE_MAX_FRAMES = 50
const DEBUG_STATE_MAX_FRAME_LINKS = 10
const DEBUG_STATE_FRAME_RESPONSE_WINDOW = 500

/**
 * @param {unknown} value
 * @returns {string | null}
 */
function truncateDebugString(value) {
	if (typeof value !== 'string' || value.length === 0) return null
	return value.length > DEBUG_STATE_MAX_STRING_LENGTH
		? `${value.slice(0, DEBUG_STATE_MAX_STRING_LENGTH)}…`
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

/** @returns {unknown[]} */
function getPrebidInstances() {
	const names = Array.isArray(window._pbjsGlobals) ? window._pbjsGlobals : ['pbjs']
	return names
		.map((name) => window[name])
		.filter((pbjs) => typeof pbjs?.getBidResponses === 'function')
}

/** @param {any} bid */
function serializePrebidBid(bid) {
	return {
		bidder: bid.bidderCode ?? bid.bidder ?? null,
		ad_id: bid.adId ?? null,
		creative_id: bid.creativeId ?? null,
		deal_id: bid.dealId ?? null,
		ad_unit_code: bid.adUnitCode ?? null,
		media_type: bid.mediaType ?? null,
		size: bid.width && bid.height ? `${bid.width}x${bid.height}` : null,
		cpm: bid.cpm ?? null,
		currency: bid.currency ?? null,
		advertiser_domains: bid.meta?.advertiserDomains ?? null,
		advertiser_name: bid.meta?.advertiserName ?? null,
		advertiser_id: bid.meta?.advertiserId ?? null,
		brand_name: bid.meta?.brandName ?? null,
		network_name: bid.meta?.networkName ?? null,
		network_id: bid.meta?.networkId ?? null,
		status: bid.status ?? null,
		response_timestamp: bid.responseTimestamp ?? null,
	}
}

/** @returns {Map<string, any>} */
function collectPrebidBidsById() {
	const bids = new Map()

	for (const pbjs of getPrebidInstances()) {
		const responses = tryCollect(() => Object.values(pbjs.getBidResponses())) ?? []
		for (const response of responses) {
			for (const bid of response?.bids ?? []) {
				if (bid?.adId) bids.set(bid.adId, bid)
			}
		}

		for (const bid of tryCollect(() => pbjs.getAllWinningBids()) ?? []) {
			if (bid?.adId) bids.set(bid.adId, bid)
		}
	}

	return bids
}

/** @param {any} slot */
function getSlotTargeting(slot) {
	return Object.fromEntries(slot.getTargetingKeys().map((key) => [key, slot.getTargeting(key)]))
}

/**
 * @param {any} slot
 * @param {Map<string, any>} prebidBids
 */
function serializeGptSlot(slot, prebidBids) {
	const elementId = tryCollect(() => slot.getSlotElementId())
	const iframe = elementId ? document.getElementById(elementId)?.querySelector('iframe') : null
	const targeting = tryCollect(() => getSlotTargeting(slot))
	const prebidAdId = targeting?.hb_adid?.[0]
	const prebidBid = prebidAdId ? prebidBids.get(prebidAdId) : null

	return {
		element_id: elementId,
		ad_unit_path: tryCollect(() => slot.getAdUnitPath()),
		response: tryCollect(() => slot.getResponseInformation()),
		prebid_bid: prebidBid ? serializePrebidBid(prebidBid) : null,
		targeting,
		iframe: iframe
			? {
					id: iframe.id || null,
					src: truncateDebugString(iframe.src),
					size: `${iframe.width || iframe.clientWidth}x${iframe.height || iframe.clientHeight}`,
				}
			: null,
	}
}

/**
 * Answers a debug request from the top frame with this frame's URL and links, which lets the
 * top frame see inside cross-origin ad creatives.
 *
 * @param {MessageEvent} event
 * @returns {void}
 */
function handleAdsDebugFrameRequest(event) {
	if (!event.data?.modrinthAdsDebugRequest || event.source !== window.top) return

	const links = Array.from(document.querySelectorAll('a[href]'), (link) =>
		truncateDebugString(link.href),
	)
		.filter(Boolean)
		.slice(0, DEBUG_STATE_MAX_FRAME_LINKS)

	window.top.postMessage(
		{ modrinthAdsDebugFrame: { url: truncateDebugString(window.location.href), links } },
		event.origin,
	)
}

/**
 * @param {Window} root
 * @param {Window[]} frames
 * @returns {Window[]}
 */
function collectFrameWindows(root, frames = []) {
	for (let i = 0; i < root.frames.length && frames.length < DEBUG_STATE_MAX_FRAMES; i++) {
		frames.push(root.frames[i])
		collectFrameWindows(root.frames[i], frames)
	}
	return frames
}

/** @returns {Promise<{ origin: string, url: string | null, links: string[] }[]>} */
function collectAdFrames() {
	return new Promise((resolve) => {
		const frames = []

		/** @param {MessageEvent} event */
		const handleResponse = (event) => {
			const frame = event.data?.modrinthAdsDebugFrame
			if (!frame || frames.length >= DEBUG_STATE_MAX_FRAMES) return

			frames.push({
				origin: event.origin,
				url: truncateDebugString(frame.url),
				links: Array.isArray(frame.links)
					? frame.links
							.map(truncateDebugString)
							.filter(Boolean)
							.slice(0, DEBUG_STATE_MAX_FRAME_LINKS)
					: [],
			})
		}

		window.addEventListener('message', handleResponse)

		for (const frame of tryCollect(() => collectFrameWindows(window)) ?? []) {
			try {
				frame.postMessage({ modrinthAdsDebugRequest: true }, '*')
			} catch {}
		}

		setTimeout(() => {
			window.removeEventListener('message', handleResponse)
			resolve(frames)
		}, DEBUG_STATE_FRAME_RESPONSE_WINDOW)
	})
}

/**
 * Picks the URL that best identifies the rendered ad: its click-through link if a creative
 * exposes one, otherwise the creative frame itself.
 *
 * @param {{ origin: string, url: string | null, links: string[] }[]} frames
 * @returns {string | null}
 */
function getCurrentAdUrl(frames) {
	const adFrames = frames.filter((frame) => frame.origin !== window.location.origin)

	return (
		adFrames.flatMap((frame) => frame.links)[0] ??
		adFrames.find((frame) => frame.url && frame.url !== 'about:blank')?.url ??
		null
	)
}

/** @returns {Promise<Record<string, unknown>>} */
async function collectAdsDebugState() {
	const prebidBids = collectPrebidBidsById()
	const gptSlots = tryCollect(() => window.googletag?.pubads?.().getSlots()) ?? []
	const frames = await collectAdFrames()

	return {
		current_ad: getCurrentAdUrl(frames),
		url: window.location.href,
		user_agent: navigator.userAgent,
		viewport: `${window.innerWidth}x${window.innerHeight}@${window.devicePixelRatio}`,
		consent_phase: tryCollect(() => controller.state.phase),
		gpt_slots: gptSlots.map((slot) => serializeGptSlot(slot, prebidBids)),
		prebid_winning_bids: getPrebidInstances().flatMap(
			(pbjs) => tryCollect(() => pbjs.getAllWinningBids().map(serializePrebidBid)) ?? [],
		),
		frames,
	}
}

/** @returns {Promise<void>} */
async function reportAdsDebugState() {
	const invoke = getTauriInvoke()
	if (!invoke) return

	let debugState
	try {
		debugState = await collectAdsDebugState()
	} catch (error) {
		debugState = { url: window.location.href, error: String(error) }
	}

	try {
		await invoke('plugin:ads|report_ads_debug_state', { debugState })
	} catch {}
}
