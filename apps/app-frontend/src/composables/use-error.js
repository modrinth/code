import { reactive } from 'vue'

import { findMinecraftAuthError } from '@/components/ui/minecraft-auth-error-modal/minecraft-auth-errors'

const errorState = reactive({
	errorModal: null,
	minecraftAuthErrorModal: null,
	minecraftRequiredModal: null,
	setErrorModal(ref) {
		this.errorModal = ref
	},
	setMinecraftAuthErrorModal(ref) {
		this.minecraftAuthErrorModal = ref
	},
	setMinecraftRequiredModal(ref) {
		this.minecraftRequiredModal = ref
	},
	showError(error, context, closable = true, source = null) {
		const errorMessage = error.message?.toLowerCase()
		if (
			(errorMessage?.includes('user is not logged in') ||
				errorMessage?.includes('cannot play instance since minecraft is required')) &&
			this.minecraftRequiredModal
		) {
			this.minecraftRequiredModal.show()
			return
		}
		if (
			error.message &&
			(error.message.includes('Minecraft authentication error:') ||
				findMinecraftAuthError(error.message)) &&
			this.minecraftAuthErrorModal
		) {
			this.minecraftAuthErrorModal.show(error)
			return
		}
		this.errorModal.show(error, context, closable, source)
	},
})

export function useError() {
	return errorState
}

export const handleSevereError = (error, context) => {
	useError().showError(error, context)
	console.error(error)
}
