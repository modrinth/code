const LAYOUT_DELAY = 100
const CONSENT_TIMEOUT = 10_000

class AdsConsentController {
	constructor() {
		/** @type {AdsConsentState} */
		this.state = new AdsConsentState()

		/** @type {AdsConsentPhase | null} */
		this.preSubmissionPhase = null

		/** @type {ReturnType<typeof setTimeout> | null} */
		this.submissionTimeout = null

		/** @type {ReturnType<typeof setTimeout> | null} */
		this.popupReadinessTimeout = null
	}

	/** @returns {void} */
	syncConsentPopup() {
		// The CMP root persists while its internal views change, so only its removal finishes
		// a normal active flow. Submissions wait for the CMP event or the timeout fallback so
		// the consent has time to persist before the ads webview is refreshed.
		const cmpMain = document.getElementById('qc-cmp2-main')

		if (!cmpMain) {
			this.clearPopupReadinessTimeout()
			if (this.state.phase === 'idle') {
				document.documentElement.classList.remove('modrinth-ads-consent-overlay')
			} else if (this.state.phase !== 'submitting-consent' && this.state.phase !== 'finishing') {
				this.finishConsentFlow()
			}
			return
		}

		if (this.state.phase !== 'idle') return

		document.documentElement.classList.add('modrinth-ads-consent-overlay')
		const variant = this.detectVariant()
		if (!areConsentControlsPresent(variant)) {
			this.waitForConsentControls()
			return
		}

		this.clearPopupReadinessTimeout()
		this.state.setState('showing-popup')
		this.setPopupMode('custom')
	}

	/** @returns {void} */
	waitForConsentControls() {
		if (this.popupReadinessTimeout) return

		this.popupReadinessTimeout = setTimeout(() => {
			this.popupReadinessTimeout = null

			if (this.state.phase !== 'idle' || !document.getElementById('qc-cmp2-main')) return

			const variant = this.detectVariant()
			if (areConsentControlsPresent(variant)) {
				this.syncConsentPopup()
				return
			}

			this.state.setState('showing-popup')
			this.showNativeCmpFallback()
		}, CONSENT_TIMEOUT)
	}

	/** @returns {void} */
	clearPopupReadinessTimeout() {
		clearTimeout(this.popupReadinessTimeout ?? undefined)
		this.popupReadinessTimeout = null
	}

	/**
	 * Sends the clicks from custom privacy popup to native popup buttons
	 * @param {ConsentAction} action
	 * @returns {Promise<void>}
	 */
	async performAction(action) {
		if (!['accept', 'reject', 'manage'].includes(action)) return

		if (action === 'manage') {
			try {
				await this.openPreferences()
			} catch {
				this.showNativeCmpFallback()
			}
			return
		}

		const variant = this.detectVariant()
		if (!isConsentActionAvailable(action, variant)) {
			this.showNativeCmpFallback()
			return
		}

		try {
			const result = await performDocumentConsentAction(action, variant, () =>
				this.beginConsentSubmission(),
			)
			if (result !== 'handled') {
				this.cancelConsentSubmission()
				this.showNativeCmpFallback()
			}
		} catch {
			this.cancelConsentSubmission()
			this.showNativeCmpFallback()
		}
	}

	/** @returns {void} */
	beginConsentSubmission() {
		if (
			!['showing-popup', 'showing-preferences', 'showing-reopened-preferences'].includes(
				this.state.phase,
			)
		) {
			return
		}

		this.preSubmissionPhase = this.state.phase
		this.state.setState('submitting-consent')
		clearTimeout(this.submissionTimeout ?? undefined)
		this.submissionTimeout = setTimeout(() => {
			const preSubmissionPhase = this.preSubmissionPhase
			this.submissionTimeout = null
			const dialogId = this.state.variant === 'usp' ? 'qc-cmp2-usp' : 'qc-cmp2-ui'

			if (this.state.phase === 'submitting-consent' && !document.getElementById(dialogId)) {
				this.finishConsentFlow()
			} else if (this.state.phase === 'submitting-consent' && preSubmissionPhase) {
				this.state.setState(preSubmissionPhase)
				this.preSubmissionPhase = null
				this.showNativeCmpFallback()
			}
		}, CONSENT_TIMEOUT)
	}

	/** @returns {void} */
	cancelConsentSubmission() {
		clearTimeout(this.submissionTimeout ?? undefined)
		this.submissionTimeout = null

		if (this.state.phase === 'submitting-consent' && this.preSubmissionPhase) {
			this.state.setState(this.preSubmissionPhase)
		}
		this.preSubmissionPhase = null
	}

	/**
	 * @param {{ eventStatus?: string } | null | undefined} tcData
	 * @param {boolean} success
	 * @returns {void}
	 */
	handleTcfConsentEvent(tcData, success) {
		if (
			success &&
			tcData?.eventStatus === 'useractioncomplete' &&
			this.state.variant === 'tcf' &&
			this.state.phase !== 'idle' &&
			this.state.phase !== 'finishing'
		) {
			this.finishConsentFlow()
		}
	}

	/**
	 * @param {{ eventName?: string } | null | undefined} gppData
	 * @param {boolean} success
	 * @returns {void}
	 */
	handleGppConsentEvent(gppData, success) {
		if (
			success &&
			gppData?.eventName === 'sectionChange' &&
			this.state.variant === 'usp' &&
			this.state.phase === 'submitting-consent'
		) {
			this.finishConsentFlow()
		}
	}

	/** @returns {void} */
	handleTcfClose() {
		if (this.state.variant !== 'tcf') return

		if (this.state.phase === 'showing-preferences') {
			this.state.setState('showing-popup')
			this.concealPreferences()
			this.setPopupMode('custom')
		} else if (this.state.phase === 'showing-reopened-preferences') {
			this.finishReopenedPopup()
		}
	}

	/** @returns {Promise<void>} */
	async reopenPreferences() {
		if (document.documentElement.classList.contains('modrinth-ads-consent-overlay')) {
			try {
				await this.openPreferences()
			} catch {
				this.showNativeCmpFallback()
			}
			return
		}

		this.state.setState('showing-reopened-preferences')
		this.preparePreferences()

		try {
			await this.waitForLayout()
			await expandAdsConsentWebview()
			await this.waitForLayout()
			this.revealPreferences()
			window.dispatchEvent(new Event('resize'))

			if (!(await this.displayReopenedPopup())) {
				this.finishReopenedPopup()
				return
			}

			if (!(await this.openConsentManagerWhenReady(CONSENT_TIMEOUT))) {
				this.showNativeCmpFallback()
			}
		} catch {
			this.finishReopenedPopup()
		}
	}

	/** @returns {AdsConsentVariant | null} */
	detectVariant() {
		const variant = detectConsentVariant()
		if (variant) this.state.setVariant(variant)
		return variant
	}

	/**
	 * @param {number} timeoutMs
	 * @returns {Promise<boolean>}
	 */
	async openConsentManagerWhenReady(timeoutMs) {
		const deadline = Date.now() + timeoutMs

		while (true) {
			try {
				const result = await performDocumentConsentAction('manage', this.detectVariant())
				if (result === 'handled') return true
				if (result === 'failed' || Date.now() >= deadline) return false
			} catch {
				return false
			}

			await new Promise((resolve) => setTimeout(resolve, 200))
		}
	}

	/** @returns {void} */
	preparePreferences() {
		document.documentElement.classList.add('modrinth-ads-consent-preferences')
		document.documentElement.classList.remove('modrinth-ads-consent-preferences-visible')
	}

	/** @returns {void} */
	revealPreferences() {
		document.documentElement.classList.add('modrinth-ads-consent-preferences-visible')
	}

	/** @returns {void} */
	concealPreferences() {
		document.documentElement.classList.remove('modrinth-ads-consent-preferences')
		document.documentElement.classList.remove('modrinth-ads-consent-preferences-visible')
	}

	/** @returns {Promise<void>} */
	waitForLayout() {
		return new Promise((resolve) => setTimeout(resolve, LAYOUT_DELAY))
	}

	/** @returns {Promise<void>} */
	async showExpandedUi() {
		this.preparePreferences()
		await this.waitForLayout()
		await expandAdsConsentWebview()
		await this.waitForLayout()
		this.revealPreferences()
		window.dispatchEvent(new Event('resize'))
	}

	/** @returns {Promise<boolean>} */
	async openPreferences() {
		this.state.setState('showing-preferences')
		await this.showExpandedUi()

		if (!(await this.openConsentManagerWhenReady(CONSENT_TIMEOUT))) {
			this.showNativeCmpFallback()
			return false
		}

		return true
	}

	/** @returns {Promise<boolean>} */
	async displayReopenedPopup() {
		if ((this.state.variant === 'usp' || (await isUspConsentApplicable())) && window.__uspapi) {
			this.state.setVariant('usp')
			return displayUspConsentUi()
		}

		if (!window.__tcfapi) return false
		this.state.setVariant('tcf')
		window.__tcfapi('displayConsentUi', 2, () => {})
		return true
	}

	/**
	 * @param {AdsConsentPopupMode} mode
	 * @returns {void}
	 */
	setPopupMode(mode) {
		const shown = mode !== 'hidden'
		const hidden = mode === 'hidden'
		document.documentElement.classList.toggle('modrinth-ads-consent-overlay', shown)
		document.documentElement.classList.toggle('modrinth-ads-consent-fallback', mode === 'fallback')
		if (hidden) this.concealPreferences()
		void invokeAdsConsentPopupMode(mode)
	}

	/** @returns {void} */
	showNativeCmpFallback() {
		this.concealPreferences()
		this.setPopupMode('fallback')
	}

	/** @returns {void} */
	finishReopenedPopup() {
		this.finishConsentFlow()
	}

	/** @returns {void} */
	finishConsentFlow() {
		if (this.state.phase === 'idle' || this.state.phase === 'finishing') {
			return
		}

		clearTimeout(this.submissionTimeout ?? undefined)
		this.submissionTimeout = null
		this.clearPopupReadinessTimeout()
		this.preSubmissionPhase = null
		this.state.setState('finishing')
		this.setPopupMode('hidden')
	}
}
