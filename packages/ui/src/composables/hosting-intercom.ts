import {
	boot as bootIntercom,
	Intercom,
	shutdown as shutdownIntercom,
	update as updateIntercom,
} from '@intercom/messenger-js-sdk'
import { computed, type MaybeRefOrGetter,onBeforeUnmount, onMounted, ref, toValue, watch } from 'vue'

import { useModalStack } from './modal-stack'

type FetchIntercomToken = () => Promise<{ token: string }>

export interface UseHostingIntercomOptions {
	enabled: MaybeRefOrGetter<boolean>
	appId: MaybeRefOrGetter<string | undefined>
	fetchToken: FetchIntercomToken
	identityKey: MaybeRefOrGetter<string | null | undefined>
	horizontalPadding?: MaybeRefOrGetter<number | undefined>
}

const DEFAULT_PADDING = 20
const BUBBLE_WIDTH = 72
const INTERCOM_STYLE_ID = 'modrinth-hosting-intercom-style'
const RIGHT_VAR = '--modrinth-hosting-intercom-right'
const BOTTOM_VAR = '--modrinth-hosting-intercom-bottom'
const POINTER_EVENTS_VAR = '--modrinth-hosting-intercom-pointer-events'

function sanitizePixels(value: number | undefined, fallback = DEFAULT_PADDING) {
	if (typeof value !== 'number' || !Number.isFinite(value)) return fallback
	return Math.max(0, Math.ceil(value))
}

function ensureIntercomStyle() {
	if (typeof document === 'undefined' || document.getElementById(INTERCOM_STYLE_ID)) return

	const style = document.createElement('style')
	style.id = INTERCOM_STYLE_ID
	style.textContent = `
.intercom-lightweight-app,
.intercom-lightweight-app-launcher,
.intercom-lightweight-app-messenger,
.intercom-launcher-frame,
.intercom-messenger-frame,
#intercom-container,
#intercom-frame,
iframe[name='intercom-launcher-frame'],
iframe[name='intercom-messenger-frame'] {
	z-index: 98 !important;
	pointer-events: var(${POINTER_EVENTS_VAR}, auto) !important;
}

.intercom-lightweight-app-launcher,
.intercom-launcher-frame,
iframe[name='intercom-launcher-frame'] {
	right: var(${RIGHT_VAR}, ${DEFAULT_PADDING}px) !important;
	bottom: var(${BOTTOM_VAR}, ${DEFAULT_PADDING}px) !important;
}
`
	document.head.appendChild(style)
}

export function useHostingIntercom(options: UseHostingIntercomOptions) {
	const { stackCount } = useModalStack()
	const horizontalPaddingRequests = new Map<symbol, number>()
	const verticalClearanceRequests = new Map<symbol, number>()
	const requestedHorizontalPadding = ref<number | null>(null)
	const requestedVerticalClearance = ref<number | null>(null)
	let booted = false
	let booting = false
	let bootedIdentity: string | null = null
	let bootRun = 0
	let syncAfterBoot = false
	let stopSync: (() => void) | null = null
	let stopPositionSync: (() => void) | null = null
	let stopModalSync: (() => void) | null = null

	const horizontalPadding = computed(
		() =>
			requestedHorizontalPadding.value ??
			sanitizePixels(toValue(options.horizontalPadding), DEFAULT_PADDING),
	)
	const verticalPadding = computed(() => requestedVerticalClearance.value ?? DEFAULT_PADDING)
	const enabled = computed(() => Boolean(toValue(options.enabled)) && Boolean(toValue(options.appId)))
	const identity = computed(() => String(toValue(options.identityKey) ?? 'hosting'))

	function requestFromMap(
		requests: Map<symbol, number>,
		target: typeof requestedHorizontalPadding,
		id: symbol,
		value: number | null,
	) {
		if (value === null || !Number.isFinite(value)) {
			requests.delete(id)
		} else {
			requests.set(id, sanitizePixels(value))
		}

		target.value = requests.size > 0 ? Math.max(...requests.values()) : null
	}

	function applyPosition(updateSdk = false) {
		if (typeof document === 'undefined') return

		document.documentElement.style.setProperty(RIGHT_VAR, `${horizontalPadding.value}px`)
		document.documentElement.style.setProperty(BOTTOM_VAR, `${verticalPadding.value}px`)
		document.documentElement.style.setProperty(
			POINTER_EVENTS_VAR,
			stackCount.value > 0 ? 'none' : 'auto',
		)

		if (updateSdk && booted) {
			updateIntercom({
				horizontal_padding: horizontalPadding.value,
				vertical_padding: verticalPadding.value,
			})
		}
	}

	function clearPosition() {
		if (typeof document === 'undefined') return

		document.documentElement.style.removeProperty(RIGHT_VAR)
		document.documentElement.style.removeProperty(BOTTOM_VAR)
		document.documentElement.style.removeProperty(POINTER_EVENTS_VAR)
	}

	function stop() {
		bootRun++
		syncAfterBoot = false
		if (booted) shutdownIntercom()
		booting = false
		booted = false
		bootedIdentity = null
	}

	async function start(currentIdentity: string) {
		const appId = toValue(options.appId)
		if (!appId) return

		const run = ++bootRun
		booting = true

		try {
			const { token } = await options.fetchToken()
			if (run !== bootRun || !enabled.value || identity.value !== currentIdentity) return

			const settings = {
				app_id: appId,
				intercom_user_jwt: token,
				session_duration: 1000 * 60 * 60 * 24,
				alignment: 'right',
				horizontal_padding: horizontalPadding.value,
				vertical_padding: verticalPadding.value,
			}
			if (typeof window !== 'undefined' && window.Intercom) {
				bootIntercom(settings)
			} else {
				Intercom(settings)
			}
			booted = true
			bootedIdentity = currentIdentity
			applyPosition()
		} catch (error) {
			if (run === bootRun) {
				console.warn('[HOSTING][INTERCOM] failed to initialize secure support chat', error)
			}
		} finally {
			if (run !== bootRun) return
			booting = false
			if (syncAfterBoot) {
				syncAfterBoot = false
				sync()
			}
		}
	}

	function sync() {
		if (!enabled.value) {
			if (booted || booting) stop()
			return
		}

		if (booted && bootedIdentity === identity.value) {
			applyPosition(true)
		} else if (booting) {
			syncAfterBoot = true
		} else {
			if (booted) stop()
			void start(identity.value)
		}
	}

	onMounted(() => {
		ensureIntercomStyle()
		applyPosition()
		stopSync = watch([enabled, identity], sync, {
			immediate: true,
		})
		stopPositionSync = watch([horizontalPadding, verticalPadding], () => applyPosition(true))
		stopModalSync = watch(stackCount, () => applyPosition())
	})

	onBeforeUnmount(() => {
		stopSync?.()
		stopPositionSync?.()
		stopModalSync?.()
		stop()
		clearPosition()
	})

	return {
		intercomBubble: {
			width: computed(() => BUBBLE_WIDTH),
			horizontalPadding,
			requestHorizontalPadding: (id: symbol, value: number | null) =>
				requestFromMap(horizontalPaddingRequests, requestedHorizontalPadding, id, value),
			requestVerticalClearance: (id: symbol, value: number | null) =>
				requestFromMap(verticalClearanceRequests, requestedVerticalClearance, id, value),
		},
	}
}
