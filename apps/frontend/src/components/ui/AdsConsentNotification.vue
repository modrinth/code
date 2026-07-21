<script setup lang="ts">
import {
	defineMessages,
	injectNotificationManager,
	useVIntl,
	type WebNotification,
} from '@modrinth/ui'
import { onBeforeUnmount, onMounted } from 'vue'

type ConsentAction = 'accept' | 'reject' | 'manage'

interface TcfData {
	eventStatus?: string
	listenerId?: number
}

type TcfCallback = (data: TcfData, success: boolean) => void
type TcfApi = (command: string, version: number, callback: TcfCallback, parameter?: unknown) => void

const CMP_HIDDEN_CLASS = 'modrinth-cmp-summary-hidden'
const notificationManager = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'ads-consent.title',
		defaultMessage: 'Your privacy and how ads support Modrinth',
	},
	body: {
		id: 'ads-consent.body',
		defaultMessage:
			'Ads make Modrinth possible and fund creator payouts. Our partners may store or access cookies on the website to personalize ads and measure performance.',
	},
	manage: {
		id: 'ads-consent.manage',
		defaultMessage: 'Manage preferences',
	},
	reject: {
		id: 'ads-consent.reject',
		defaultMessage: 'Reject all',
	},
	accept: {
		id: 'ads-consent.accept',
		defaultMessage: 'Accept all',
	},
})

let notificationId: WebNotification['id'] | null = null
let tcfListenerId: number | undefined
let listenerInstalled = false
let managingPreferences = false
let consentContainerObserver: MutationObserver | undefined
const consentContainerContains = new Map<HTMLElement, HTMLElement['contains']>()

function getTcfApi(): TcfApi | undefined {
	return (window as typeof window & { __tcfapi?: TcfApi }).__tcfapi
}

function setConsentUiHidden(hidden: boolean) {
	document.documentElement.classList.toggle(CMP_HIDDEN_CLASS, hidden)
	patchConsentFocusTrapContainer()
}

function patchConsentFocusTrapContainer() {
	const container = document.querySelector<HTMLElement>('#qc-cmp2-ui')
	if (!container || consentContainerContains.has(container)) return

	const originalContains = container.contains
	// InMobi's focus trap otherwise cancels clicks outside its hidden container.
	container.contains = (node: Node | null) =>
		document.documentElement.classList.contains(CMP_HIDDEN_CLASS) ||
		originalContains.call(container, node)
	consentContainerContains.set(container, originalContains)
}

function getConsentContainers(): ParentNode[] {
	const containers = Array.from(
		document.querySelectorAll<HTMLElement>('#qc-cmp2-container, #qc-cmp2-main, #qc-cmp2-ui'),
	)

	return containers.length > 0 ? containers : [document]
}

function matchesButtonText(button: HTMLButtonElement, terms: string[]): boolean {
	const text = [button.textContent, button.getAttribute('aria-label')]
		.filter(Boolean)
		.join(' ')
		.trim()
		.toLowerCase()

	return terms.some((term) => text.includes(term))
}

function findConsentButton(action: ConsentAction): HTMLButtonElement | null {
	const containers = getConsentContainers()
	const summaryButtons = containers.flatMap((container) =>
		Array.from(container.querySelectorAll<HTMLButtonElement>('.qc-cmp2-summary-buttons button')),
	)
	const queryButton = (selector: string) =>
		containers
			.map((container) => container.querySelector<HTMLButtonElement>(selector))
			.find((button) => button && !button.disabled) ?? null

	if (action === 'accept') {
		const explicitAcceptButton = queryButton(
			'[data-testid="accept-all"], [data-testid="agree-button"], #accept-btn',
		)
		if (explicitAcceptButton) return explicitAcceptButton

		const textMatch = summaryButtons.find((button) =>
			matchesButtonText(button, ['accept all', 'agree to all', 'allow all']),
		)
		if (textMatch) return textMatch
		if (summaryButtons.length >= 3 && !summaryButtons[2].disabled) return summaryButtons[2]

		return queryButton('.qc-cmp2-summary-buttons button[mode="primary"]')
	}

	if (action === 'reject') {
		const explicitRejectButton = queryButton(
			'[data-testid="reject-all"], [data-testid="disagree-button"], #disagree-btn, #reject-btn',
		)
		if (explicitRejectButton) return explicitRejectButton

		const textMatch = summaryButtons.find((button) =>
			matchesButtonText(button, ['reject all', 'disagree', 'deny all']),
		)
		if (textMatch) return textMatch
		if (summaryButtons.length >= 3 && !summaryButtons[1].disabled) return summaryButtons[1]

		const secondaryButtons = containers.flatMap((container) =>
			Array.from(
				container.querySelectorAll<HTMLButtonElement>(
					'.qc-cmp2-summary-buttons button[mode="secondary"]',
				),
			),
		)
		if (secondaryButtons.length > 1 && !secondaryButtons[1].disabled) return secondaryButtons[1]

		return null
	}

	return (
		queryButton(
			'[data-testid="manage-preferences"], [data-testid="show-options"], .qc-cmp2-summary-buttons > button[mode="secondary"]:first-of-type',
		) ??
		summaryButtons.find((button) =>
			matchesButtonText(button, ['manage', 'preference', 'settings', 'options']),
		) ??
		summaryButtons.find((button) => !button.disabled) ??
		null
	)
}

function clickConsentButtonWhenReady(action: ConsentAction, timeoutMs: number): Promise<boolean> {
	const deadline = Date.now() + timeoutMs

	return new Promise((resolve) => {
		function tryClick() {
			const button = findConsentButton(action)
			if (button) {
				button.click()
				resolve(true)
			} else if (Date.now() >= deadline) {
				resolve(false)
			} else {
				setTimeout(tryClick, 50)
			}
		}

		tryClick()
	})
}

async function performConsentAction(action: ConsentAction) {
	if (action === 'manage') {
		managingPreferences = true
		setConsentUiHidden(false)
	}

	const clicked = await clickConsentButtonWhenReady(action, action === 'manage' ? 2500 : 1000)

	if (clicked) return

	managingPreferences = action === 'manage'
	setConsentUiHidden(false)

	if (action === 'manage') {
		const tcfApi = getTcfApi()
		if (tcfApi) {
			tcfApi('displayConsentUi', 2, () => {})
		}
	}
}

function showConsentNotification() {
	setConsentUiHidden(true)

	if (
		notificationId !== null &&
		notificationManager
			.getNotifications()
			.some((notification) => notification.id === notificationId)
	)
		return

	const notification = notificationManager.addNotification({
		title: formatMessage(messages.title),
		text: formatMessage(messages.body),
		type: 'neutral',
		autoCloseMs: null,
		dismissible: false,
		copyable: false,
		noIcon: true,
		containerClass: 'py-2 !rounded-2xl',
		buttons: [
			{
				label: formatMessage(messages.manage),
				action: () => performConsentAction('manage'),
				keepOpen: true,
			},
			{
				label: formatMessage(messages.reject),
				action: () => performConsentAction('reject'),
				color: 'brand',
				keepOpen: true,
			},
			{
				label: formatMessage(messages.accept),
				action: () => performConsentAction('accept'),
				color: 'brand',
				keepOpen: true,
			},
		],
	})

	notificationId = notification.id
}

function finishConsent() {
	managingPreferences = false
	setConsentUiHidden(false)

	if (notificationId !== null) {
		notificationManager.removeNotification(notificationId)
		notificationId = null
	}
}

function handleTcfConsentEvent(data: TcfData, success: boolean) {
	if (!success) return

	if (data.listenerId !== undefined) {
		tcfListenerId = data.listenerId
	}

	if (data.eventStatus === 'cmpuishown') {
		if (!managingPreferences) {
			showConsentNotification()
		}
	} else if (data.eventStatus === 'useractioncomplete') {
		finishConsent()
	}
}

function installTcfConsentListener() {
	if (listenerInstalled) return

	const tcfApi = getTcfApi()
	if (!tcfApi) return

	listenerInstalled = true
	tcfApi('addEventListener', 2, handleTcfConsentEvent)
}

function handleDocumentClick(event: MouseEvent) {
	if (!managingPreferences || !(event.target instanceof Element)) return
	if (!event.target.closest('.qc-cmp2-close-icon')) return

	setTimeout(() => {
		if (notificationId === null) return

		managingPreferences = false
		setConsentUiHidden(true)
	})
}

onMounted(() => {
	consentContainerObserver = new MutationObserver(patchConsentFocusTrapContainer)
	consentContainerObserver.observe(document.body, { childList: true, subtree: true })
	patchConsentFocusTrapContainer()
	installTcfConsentListener()
	window.addEventListener('modrinth-cmp-ready', installTcfConsentListener)
	document.addEventListener('click', handleDocumentClick, true)
})

onBeforeUnmount(() => {
	consentContainerObserver?.disconnect()
	window.removeEventListener('modrinth-cmp-ready', installTcfConsentListener)
	document.removeEventListener('click', handleDocumentClick, true)
	setConsentUiHidden(false)
	for (const [container, originalContains] of consentContainerContains) {
		container.contains = originalContains
	}
	consentContainerContains.clear()

	const tcfApi = getTcfApi()
	if (tcfApi && tcfListenerId !== undefined) {
		tcfApi('removeEventListener', 2, () => {}, tcfListenerId)
	}

	if (notificationId !== null) {
		notificationManager.removeNotification(notificationId)
	}
})
</script>

<style>
html.modrinth-cmp-summary-hidden .qc-cmp2-container,
html.modrinth-cmp-summary-hidden #qc-cmp2-container,
html.modrinth-cmp-summary-hidden #qc-cmp2-main,
html.modrinth-cmp-summary-hidden #qc-cmp2-ui {
	display: none !important;
	z-index: -1 !important;
	pointer-events: none !important;
}
</style>
