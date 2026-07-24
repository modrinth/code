/**
 * @typedef {'idle' | 'initial' | 'reopened' | 'complete'} AdsConsentPhase
 * @typedef {'usp' | 'tcf'} AdsConsentVariant
 */

class AdsConsentState {
	constructor() {
		/** @type {AdsConsentPhase} */
		this.phase = 'idle'

		/** @type {AdsConsentVariant | null} */
		this.variant = null
	}

	/** @param {AdsConsentPhase} phase */
	setState(phase) {
		this.phase = phase
	}

	/** @param {AdsConsentVariant} variant */
	setVariant(variant) {
		this.variant = variant
	}
}
