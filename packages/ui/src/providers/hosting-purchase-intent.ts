import type { Labrinth } from '@modrinth/api-client'
import { type Ref, ref, watch } from 'vue'

import { createContext } from '.'

const PURCHASE_INTENT_STORAGE_KEY = 'modrinth:servers-purchase-intent'

export type ServerBillingInterval = 'monthly' | 'quarterly' | 'yearly'

export type PendingPurchaseIntent = {
	interval: ServerBillingInterval
	planId: string | null
	source: string
	ts: number
}

export type GuestPlanContinuePayload = {
	interval: ServerBillingInterval
	planId: string | null
}

export type CheckoutModalHandle = {
	show: (interval: ServerBillingInterval, plan?: Labrinth.Billing.Internal.Product | null) => void
}

export type GuestPlanModalHandle = {
	show: (initialInterval?: ServerBillingInterval, initialPlanId?: string | null) => void
}

export interface HostingPurchaseIntentContext {
	openPurchaseModal: () => void
	handleGuestPlanContinue: (payload: GuestPlanContinuePayload) => void
	clearPurchaseIntent: () => void
}

export interface CreateHostingPurchaseIntentContextOptions {
	authRequestSignIn: (redirectPath: string) => void | Promise<void>
	signInRedirectPath: string
	intentSource: string
	loggedIn: Readonly<Ref<boolean>>
	availableProducts: Readonly<Ref<Labrinth.Billing.Internal.Product[]>>
	canOpenCheckout: Readonly<Ref<boolean>>
	guestPlanModal: Ref<GuestPlanModalHandle | null>
	checkoutModal: Ref<CheckoutModalHandle | null>
	onCheckoutPending: () => void
}

function readPendingPurchaseIntent(): PendingPurchaseIntent | null {
	if (typeof window === 'undefined') return null
	const rawIntent = window.sessionStorage.getItem(PURCHASE_INTENT_STORAGE_KEY)
	if (!rawIntent) return null

	try {
		const parsedIntent = JSON.parse(rawIntent) as Partial<PendingPurchaseIntent>
		if (
			(parsedIntent.interval === 'monthly' ||
				parsedIntent.interval === 'quarterly' ||
				parsedIntent.interval === 'yearly') &&
			(parsedIntent.planId === null || typeof parsedIntent.planId === 'string') &&
			typeof parsedIntent.source === 'string' &&
			typeof parsedIntent.ts === 'number'
		) {
			return parsedIntent as PendingPurchaseIntent
		}
	} catch {
		return null
	}

	return null
}

function writePendingPurchaseIntent(intent: PendingPurchaseIntent) {
	if (typeof window === 'undefined') return
	window.sessionStorage.setItem(PURCHASE_INTENT_STORAGE_KEY, JSON.stringify(intent))
}

function clearStoredPurchaseIntent() {
	if (typeof window === 'undefined') return
	window.sessionStorage.removeItem(PURCHASE_INTENT_STORAGE_KEY)
}

export function createHostingPurchaseIntentContext(
	options: CreateHostingPurchaseIntentContextOptions,
): HostingPurchaseIntentContext {
	const lastSelectedInterval = ref<ServerBillingInterval>('quarterly')
	const lastSelectedPlanId = ref<string | null | undefined>(undefined)
	const pendingResumeIntent = ref<PendingPurchaseIntent | null>(null)

	function openCheckoutFromIntent(intent: PendingPurchaseIntent): boolean {
		if (!options.checkoutModal.value || !options.canOpenCheckout.value) return false

		lastSelectedInterval.value = intent.interval
		lastSelectedPlanId.value = intent.planId

		const defaultPlan =
			options.availableProducts.value.find(
				(product) => product?.metadata?.type === 'pyro' && product.metadata.ram === 6144,
			) ??
			options.availableProducts.value.find((product) => product?.metadata?.type === 'pyro') ??
			options.availableProducts.value[0]

		const selectedPlan =
			intent.planId === null
				? null
				: (options.availableProducts.value.find((product) => product.id === intent.planId) ??
					defaultPlan)

		options.checkoutModal.value.show(intent.interval, selectedPlan)
		return true
	}

	function openPurchaseModal() {
		options.guestPlanModal.value?.show(lastSelectedInterval.value, lastSelectedPlanId.value)
	}

	function handleGuestPlanContinue(payload: GuestPlanContinuePayload) {
		const intent: PendingPurchaseIntent = {
			interval: payload.interval,
			planId: payload.planId,
			source: options.intentSource,
			ts: Date.now(),
		}

		lastSelectedInterval.value = payload.interval
		lastSelectedPlanId.value = payload.planId

		if (!options.loggedIn.value) {
			writePendingPurchaseIntent(intent)
			void options.authRequestSignIn(options.signInRedirectPath)
			return
		}

		pendingResumeIntent.value = intent
		if (openCheckoutFromIntent(intent)) {
			pendingResumeIntent.value = null
		} else {
			options.onCheckoutPending()
		}
	}

	function clearPurchaseIntent() {
		clearStoredPurchaseIntent()
		pendingResumeIntent.value = null
		lastSelectedInterval.value = 'quarterly'
		lastSelectedPlanId.value = undefined
	}

	watch(
		options.loggedIn,
		(isLoggedIn) => {
			if (!isLoggedIn) return

			const pendingIntent = readPendingPurchaseIntent()
			if (!pendingIntent || pendingIntent.source !== options.intentSource) return

			clearStoredPurchaseIntent()
			pendingResumeIntent.value = pendingIntent
			lastSelectedInterval.value = pendingIntent.interval
			lastSelectedPlanId.value = pendingIntent.planId
		},
		{ immediate: true },
	)

	watch(
		() =>
			[
				options.loggedIn.value,
				options.canOpenCheckout.value,
				pendingResumeIntent.value,
				options.checkoutModal.value,
			] as const,
		([isLoggedIn, canOpen, pendingIntent]) => {
			if (!isLoggedIn || !canOpen || !pendingIntent) return
			if (openCheckoutFromIntent(pendingIntent)) {
				pendingResumeIntent.value = null
			}
		},
		{ immediate: true },
	)

	return {
		openPurchaseModal,
		handleGuestPlanContinue,
		clearPurchaseIntent,
	}
}

export const [injectHostingPurchaseIntent, provideHostingPurchaseIntent] =
	createContext<HostingPurchaseIntentContext>('HostingManagePage', 'hostingPurchaseIntent')
