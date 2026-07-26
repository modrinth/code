/**
 * @typedef {'accept' | 'reject' | 'manage'} ConsentAction
 * @typedef {'handled' | 'not-ready' | 'failed'} ConsentActionResult
 *
 * @typedef {object} UspConsentControls
 * @property {HTMLButtonElement[]} toggles
 * @property {HTMLButtonElement} confirmButton
 *
 * @typedef {object} UspPingData
 * @property {string | string[]} [mode]
 * @property {string | string[]} [jurisdiction]
 * @property {string} [location]
 */

/** @type {Readonly<Record<ConsentAction, string>>} */
const ACTION_BUTTON_IDS = {
	accept: 'accept-btn',
	reject: 'disagree-btn',
	manage: 'more-options-btn',
}

/** @type {Readonly<Record<AdsConsentVariant, string>>} */
const DIALOG_IDS = {
	tcf: 'qc-cmp2-ui',
	usp: 'qc-cmp2-usp',
}

/** @type {Readonly<{ toggles: string, confirmButton: string }>} */
const USP_CONTROL_SELECTORS = {
	toggles: '.qc-usp-container button.qc-cmp2-toggle[role="switch"]',
	confirmButton: '.qc-usp-ui-form-content button[mode="primary"]',
}

/**
 * @param {HTMLButtonElement | null} button
 * @returns {boolean | null}
 */
function isButtonEnabled(button) {
	return button && !button.disabled && button.getAttribute('aria-disabled') !== 'true'
}

/** @returns {AdsConsentVariant | null} */
function detectConsentVariant() {
	if (document.getElementById(DIALOG_IDS.usp)) return 'usp'
	if (document.getElementById(DIALOG_IDS.tcf)) return 'tcf'
	return null
}

/**
 * @param {ConsentAction} action
 * @returns {HTMLButtonElement | null}
 */
function findTcfConsentButton(action) {
	const dialog = document.getElementById(DIALOG_IDS.tcf)
	if (!dialog) return null

	const button = /** @type {HTMLButtonElement | null} */ (
		dialog.querySelector(`#${ACTION_BUTTON_IDS[action]}`)
	)
	return isButtonEnabled(button) ? button : null
}

/** @returns {UspConsentControls | null} */
function getUspConsentControls() {
	const dialog = document.getElementById(DIALOG_IDS.usp)
	if (!dialog) return null

	const toggles = Array.from(
		/** @type {NodeListOf<HTMLButtonElement>} */ (
			dialog.querySelectorAll(USP_CONTROL_SELECTORS.toggles)
		),
	)
	const confirmButton = /** @type {HTMLButtonElement | null} */ (
		dialog.querySelector(USP_CONTROL_SELECTORS.confirmButton)
	)

	if (
		toggles.length === 0 ||
		!isButtonEnabled(confirmButton) ||
		toggles.some(
			(toggle) =>
				!isButtonEnabled(toggle) ||
				!['true', 'false'].includes(toggle.getAttribute('aria-checked') ?? ''),
		)
	) {
		return null
	}

	return { toggles, confirmButton }
}

/**
 * @param {AdsConsentVariant | null} variant
 * @returns {boolean}
 */
function areConsentControlsPresent(variant) {
	if (variant === 'tcf') {
		const dialog = document.getElementById(DIALOG_IDS.tcf)
		return (
			dialog !== null &&
			/** @type {ConsentAction[]} */ (['accept', 'reject', 'manage']).every((action) =>
				dialog.querySelector(`#${ACTION_BUTTON_IDS[action]}`),
			)
		)
	}

	if (variant === 'usp') {
		const dialog = document.getElementById(DIALOG_IDS.usp)
		return (
			dialog !== null &&
			dialog.querySelectorAll(USP_CONTROL_SELECTORS.toggles).length > 0 &&
			dialog.querySelector(USP_CONTROL_SELECTORS.confirmButton) !== null
		)
	}

	return false
}

/**
 * @param {ConsentAction} action
 * @param {AdsConsentVariant | null} variant
 * @returns {boolean}
 */
function isConsentActionAvailable(action, variant) {
	if (variant === 'tcf') return findTcfConsentButton(action) !== null
	if (variant === 'usp') {
		if (action === 'manage') return document.getElementById(DIALOG_IDS.usp) !== null
		return getUspConsentControls() !== null
	}

	return false
}

/**
 * @param {number} index
 * @param {boolean} checked
 * @param {number} expectedCount
 * @param {number} deadline
 * @returns {Promise<UspConsentControls | null>}
 */
function waitForUspToggleState(index, checked, expectedCount, deadline) {
	return new Promise((resolve) => {
		/** @returns {void} */
		const checkState = () => {
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

/**
 * @param {boolean} checked
 * @param {UspConsentControls} controls
 * @param {number} timeoutMs
 * @returns {Promise<UspConsentControls | null>}
 */
async function setUspToggleStates(checked, controls, timeoutMs) {
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

/**
 * @param {ConsentAction} action
 * @param {AdsConsentVariant | null} variant
 * @param {(() => void) | undefined} onSubmit
 * @returns {Promise<ConsentActionResult>}
 */
async function performDocumentConsentAction(action, variant, onSubmit) {
	if (variant === 'usp') {
		if (action === 'manage') {
			return 'handled'
		}

		const controls = getUspConsentControls()
		if (!controls) return 'not-ready'

		const settledControls = await setUspToggleStates(action === 'reject', controls, 2_000)
		if (!settledControls) return 'failed'

		onSubmit?.()
		settledControls.confirmButton.click()
		return 'handled'
	}

	if (variant === 'tcf') {
		const button = findTcfConsentButton(action)
		if (!button) return 'not-ready'

		onSubmit?.()
		button.click()
		return 'handled'
	}

	return 'not-ready'
}

/** @returns {boolean} */
function displayUspConsentUi() {
	if (!window.__uspapi) return false

	try {
		window.__uspapi('displayUspUi', 1, () => {})
		return true
	} catch {
		return false
	}
}

/** @returns {Promise<boolean>} */
function isUspConsentApplicable() {
	if (detectConsentVariant() === 'usp') return Promise.resolve(true)
	if (!window.__uspapi) return Promise.resolve(false)

	return new Promise((resolve) => {
		let settled = false
		/**
		 * @param {boolean} applicable
		 * @returns {void}
		 */
		const settle = (applicable) => {
			if (settled) return
			settled = true
			clearTimeout(timeout)
			resolve(applicable)
		}
		const timeout = setTimeout(() => settle(false), 500)

		try {
			window.__uspapi?.(
				'uspPing',
				1,
				/**
				 * @param {UspPingData | null | undefined} data
				 * @param {boolean} success
				 * @returns {void}
				 */
				(data, success) => {
					if (!success || !data) {
						settle(false)
						return
					}

					const modes = Array.isArray(data.mode) ? data.mode : [data.mode]
					const jurisdictions = Array.isArray(data.jurisdiction)
						? data.jurisdiction
						: [data.jurisdiction]
					const location = String(data.location ?? '').toUpperCase()
					const hasUspMode = modes.some((mode) =>
						String(mode ?? '')
							.toUpperCase()
							.includes('USP'),
					)
					const locationApplies =
						!location ||
						jurisdictions.some((jurisdiction) =>
							String(jurisdiction ?? '')
								.toUpperCase()
								.includes(location),
						)

					settle(hasUspMode && locationApplies)
				},
			)
		} catch {
			settle(false)
		}
	})
}
