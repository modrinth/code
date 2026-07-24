<script setup lang="ts">
import {
	defineMessages,
	injectNotificationManager,
	useVIntl,
	type WebNotification,
} from '@modrinth/ui'
import { onBeforeUnmount, onMounted } from 'vue'

type ConsentAction = 'accept' | 'reject' | 'manage'
type ConsentVariant = 'tcf' | 'usp'

interface TcfData {
	eventStatus?: string
	listenerId?: number
}

type TcfCallback = (data: TcfData, success: boolean) => void
type TcfApi = (command: string, version: number, callback: TcfCallback, parameter?: unknown) => void

interface UspControls {
	toggles: HTMLButtonElement[]
	confirmButton: HTMLButtonElement
}

interface GppData {
	eventName?: string
	listenerId?: number
}

type GppCallback = (data: GppData, success: boolean) => void
type GppApi = (command: string, callback: GppCallback, parameter?: unknown) => void

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
let gppListenerId: number | undefined
let listenerInstalled = false
let gppListenerInstalled = false
let gppListenerAttempts = 0
let gppInstallTimeout: ReturnType<typeof setTimeout> | undefined
let managingPreferences = false
let consentComplete = false
let consentVariant: ConsentVariant | null = null
let uspConsentCommitPending = false
let uspSuccessModalDismissed = false
let uspCommitTimeout: ReturnType<typeof setTimeout> | undefined
let consentContainerObserver: MutationObserver | undefined
const consentContainerContains = new Map<HTMLElement, HTMLElement['contains']>()

function getTcfApi(): TcfApi | undefined {
	return (window as typeof window & { __tcfapi?: TcfApi }).__tcfapi
}

function getGppApi(): GppApi | undefined {
	return (window as typeof window & { __gpp?: GppApi }).__gpp
}

function setConsentUiHidden(hidden: boolean) {
	document.documentElement.classList.toggle(CMP_HIDDEN_CLASS, hidden)
	patchConsentFocusTrapContainer()
}

function patchConsentFocusTrapContainer() {
	const containers = document.querySelectorAll<HTMLElement>('#qc-cmp2-ui, #qc-cmp2-usp')

	for (const container of containers) {
		if (consentContainerContains.has(container)) continue

		const originalContains = container.contains
		// InMobi's focus trap otherwise cancels clicks outside its hidden container.
		container.contains = (node: Node | null) =>
			document.documentElement.classList.contains(CMP_HIDDEN_CLASS) ||
			originalContains.call(container, node)
		consentContainerContains.set(container, originalContains)
	}
}

function detectConsentVariant(): ConsentVariant | null {
	let variant: ConsentVariant | null = null

	if (document.getElementById('qc-cmp2-usp')) {
		variant = 'usp'
	} else if (document.getElementById('qc-cmp2-ui')) {
		variant = 'tcf'
	}

	if (variant) consentVariant = variant
	return variant
}

function getConsentContainers(): ParentNode[] {
	const containers = Array.from(
		document.querySelectorAll<HTMLElement>(
			'#qc-cmp2-container, #qc-cmp2-main, #qc-cmp2-ui, #qc-cmp2-usp',
		),
	)

	return containers.length > 0 ? containers : [document]
}

function getUspConsentControls(): UspControls | null {
	const dialog = document.getElementById('qc-cmp2-usp')
	if (!dialog) return null

	const toggles = Array.from(
		dialog.querySelectorAll<HTMLButtonElement>(
			'.qc-usp-container button.qc-cmp2-toggle[role="switch"]',
		),
	)
	const confirmButton = dialog.querySelector<HTMLButtonElement>(
		'.qc-usp-ui-form-content button[mode="primary"]',
	)

	if (
		toggles.length === 0 ||
		!confirmButton ||
		confirmButton.disabled ||
		toggles.some(
			(toggle) =>
				toggle.disabled || !['true', 'false'].includes(toggle.getAttribute('aria-checked') ?? ''),
		)
	) {
		return null
	}

	return { toggles, confirmButton }
}

function waitForUspToggleState(
	index: number,
	checked: boolean,
	expectedCount: number,
	deadline: number,
): Promise<UspControls | null> {
	return new Promise((resolve) => {
		function checkState() {
			const controls = getUspConsentControls()
			if (
				controls &&
				controls.toggles.length === expectedCount &&
				controls.toggles[index]?.getAttribute('aria-checked') === String(checked)
			) {
				resolve(controls)
			} else if (Date.now() >= deadline) {
				resolve(null)
			} else {
				setTimeout(checkState, 50)
			}
		}

		checkState()
	})
}

async function setUspToggleStates(
	checked: boolean,
	controls: UspControls,
	timeoutMs: number,
): Promise<UspControls | null> {
	const expectedCount = controls.toggles.length
	const deadline = Date.now() + timeoutMs

	for (let index = 0; index < expectedCount; index += 1) {
		const currentControls = getUspConsentControls()
		if (!currentControls || currentControls.toggles.length !== expectedCount) return null
		controls = currentControls

		const toggle = controls.toggles[index]
		if (toggle.getAttribute('aria-checked') !== String(checked)) {
			toggle.click()
			const settledControls = await waitForUspToggleState(index, checked, expectedCount, deadline)
			if (!settledControls) return null
			controls = settledControls
		}
	}

	return controls
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
	const variant = detectConsentVariant()

	if (action === 'manage') {
		managingPreferences = true
		setConsentUiHidden(false)
		if (variant === 'usp') return
	}

	if (variant === 'usp' && action !== 'manage') {
		const controls = getUspConsentControls()
		const settledControls = controls
			? await setUspToggleStates(action === 'reject', controls, 2000)
			: null

		if (settledControls) {
			beginUspConsentCommit()
			settledControls.confirmButton.click()
			return
		}

		managingPreferences = false
		setConsentUiHidden(false)
		return
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
	if (consentComplete) return

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
	consentComplete = true
	uspConsentCommitPending = false
	clearTimeout(uspCommitTimeout)
	uspCommitTimeout = undefined
	setConsentUiHidden(false)

	if (notificationId !== null) {
		notificationManager.removeNotification(notificationId)
		notificationId = null
	}
}

function beginUspConsentCommit() {
	if (!document.getElementById('qc-cmp2-usp')) return

	consentVariant = 'usp'
	uspConsentCommitPending = true
	uspSuccessModalDismissed = false
	clearTimeout(uspCommitTimeout)

	const deadline = Date.now() + 2500
	function checkForDialogClosure() {
		if (!uspConsentCommitPending) return

		if (!document.getElementById('qc-cmp2-usp')) {
			finishConsent()
		} else if (Date.now() >= deadline) {
			uspConsentCommitPending = false
			uspCommitTimeout = undefined
		} else {
			uspCommitTimeout = setTimeout(checkForDialogClosure, 50)
		}
	}

	uspCommitTimeout = setTimeout(checkForDialogClosure, 50)
}

function dismissUspSuccessModal() {
	if ((!uspConsentCommitPending && !consentComplete) || uspSuccessModalDismissed) return

	const closeButton = document.querySelector<HTMLElement>(
		'#qc-cmp2-usp [aria-label="Close success modal"]',
	)
	if (!closeButton || (closeButton instanceof HTMLButtonElement && closeButton.disabled)) return

	uspSuccessModalDismissed = true
	closeButton.click()
}

function syncConsentUi() {
	patchConsentFocusTrapContainer()
	installGppConsentListener()
	const variant = detectConsentVariant()
	dismissUspSuccessModal()

	if (
		variant === 'usp' &&
		!consentComplete &&
		!uspConsentCommitPending &&
		!managingPreferences &&
		getUspConsentControls()
	) {
		showConsentNotification()
	}

	if (uspConsentCommitPending && !document.getElementById('qc-cmp2-usp')) {
		finishConsent()
	}
}

function handleTcfConsentEvent(data: TcfData, success: boolean) {
	if (!success) return
	detectConsentVariant()

	if (data.listenerId !== undefined) {
		tcfListenerId = data.listenerId
	}

	if (data.eventStatus === 'cmpuishown') {
		if (!managingPreferences) {
			showConsentNotification()
		}
	} else if (data.eventStatus === 'useractioncomplete' && consentVariant === 'tcf') {
		finishConsent()
	}
}

function handleGppConsentEvent(data: GppData, success: boolean) {
	if (data.listenerId !== undefined) {
		gppListenerId = data.listenerId
	}

	if (
		success &&
		data.eventName === 'sectionChange' &&
		consentVariant === 'usp' &&
		uspConsentCommitPending
	) {
		finishConsent()
	}
}

function installGppConsentListener() {
	if (gppListenerInstalled) return

	const gppApi = getGppApi()
	if (!gppApi) {
		if (gppListenerAttempts < 60 && gppInstallTimeout === undefined) {
			gppListenerAttempts += 1
			gppInstallTimeout = setTimeout(() => {
				gppInstallTimeout = undefined
				installGppConsentListener()
			}, 500)
		}
		return
	}

	gppListenerInstalled = true
	clearTimeout(gppInstallTimeout)
	gppInstallTimeout = undefined
	gppApi('addEventListener', handleGppConsentEvent)
}

function installTcfConsentListener() {
	if (listenerInstalled) return

	const tcfApi = getTcfApi()
	if (!tcfApi) return

	listenerInstalled = true
	tcfApi('addEventListener', 2, handleTcfConsentEvent)
}

function restoreConsentNotification() {
	setTimeout(() => {
		managingPreferences = false
		showConsentNotification()
	})
}

function handleDocumentClick(event: MouseEvent) {
	if (!(event.target instanceof Element)) return

	if (event.target.closest('#qc-cmp2-usp .qc-usp-ui-form-content button[mode="primary"]')) {
		beginUspConsentCommit()
		return
	}

	if (!managingPreferences) return
	if (!event.target.closest('.qc-cmp2-close-icon, .qc-usp-close-icon')) return

	restoreConsentNotification()
}

function handleDocumentKeydown(event: KeyboardEvent) {
	if (event.key === 'Escape' && managingPreferences) {
		restoreConsentNotification()
	}
}

onMounted(() => {
	consentContainerObserver = new MutationObserver(syncConsentUi)
	consentContainerObserver.observe(document.body, {
		childList: true,
		subtree: true,
		attributes: true,
		attributeFilter: ['aria-label', 'disabled'],
	})
	syncConsentUi()
	installTcfConsentListener()
	installGppConsentListener()
	window.addEventListener('modrinth-cmp-ready', installTcfConsentListener)
	document.addEventListener('click', handleDocumentClick, true)
	document.addEventListener('keydown', handleDocumentKeydown, true)
})

onBeforeUnmount(() => {
	consentContainerObserver?.disconnect()
	clearTimeout(uspCommitTimeout)
	clearTimeout(gppInstallTimeout)
	window.removeEventListener('modrinth-cmp-ready', installTcfConsentListener)
	document.removeEventListener('click', handleDocumentClick, true)
	document.removeEventListener('keydown', handleDocumentKeydown, true)
	setConsentUiHidden(false)
	for (const [container, originalContains] of consentContainerContains) {
		container.contains = originalContains
	}
	consentContainerContains.clear()

	const tcfApi = getTcfApi()
	if (tcfApi && tcfListenerId !== undefined) {
		tcfApi('removeEventListener', 2, () => {}, tcfListenerId)
	}

	const gppApi = getGppApi()
	if (gppApi && gppListenerId !== undefined) {
		gppApi('removeEventListener', () => {}, gppListenerId)
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
html.modrinth-cmp-summary-hidden #qc-cmp2-ui,
html.modrinth-cmp-summary-hidden #qc-cmp2-usp {
	display: none !important;
	z-index: -1 !important;
	pointer-events: none !important;
}
</style>
