const ACTION_TIMEOUT = 10_000
const LAYOUT_DELAY = 100

class AdsConsentController {
	constructor() {
		/** @type {AdsConsentState} */
		this.state = new AdsConsentState()
	}

	/** @returns {void} */
	syncConsentPopup() {
		const cmpMain = document.getElementById('qc-cmp2-main')

		// no cmp container, remove our popup
		if (!cmpMain) {
			if (this.state.phase === 'idle') {
				document.documentElement.classList.remove('modrinth-ads-consent-overlay')
			} else if (this.state.phase !== 'complete') {
				this.finishConsentFlow()
			}
			return
		}

		if (this.state.phase !== 'idle') return

		// has cmp container, add our popup
		document.documentElement.classList.add('modrinth-ads-consent-overlay')
		const variant = this.detectVariant()
		if (!areConsentControlsPresent(variant)) return

		this.state.setState('initial')
		this.setPopupMode('custom')
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
			const result = await performDocumentConsentAction(action, variant)
			if (result !== 'handled') this.showNativeCmpFallback()
		} catch {
			this.showNativeCmpFallback()
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

		this.state.setState('reopened')
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

			if (!(await this.openConsentManagerWhenReady(ACTION_TIMEOUT))) {
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

			await new Promise((resolve) => setTimeout(resolve, 50))
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
		await this.showExpandedUi()

		if (!(await this.openConsentManagerWhenReady(ACTION_TIMEOUT))) {
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
		invokeAdsConsentPopupMode(mode)
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
		if (this.state.phase === 'idle' || this.state.phase === 'complete') {
			return
		}

		this.state.setState('complete')
		this.setPopupMode('hidden')
	}
}
