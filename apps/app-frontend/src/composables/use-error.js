import { reactive } from 'vue'

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
		if (error.code === 'not_logged_in' && this.minecraftRequiredModal) {
			this.minecraftRequiredModal.show()
			return
		}
		if (error.code?.startsWith('minecraft_auth_') && this.minecraftAuthErrorModal) {
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
