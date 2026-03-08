import { defineStore } from 'pinia'

export const useError = defineStore('errorsStore', {
	state: () => ({
		errorModal: null,
		minecraftAuthErrorModal: null,
	}),
	actions: {
		setErrorModal(ref) {
			this.errorModal = ref
		},
		setMinecraftAuthErrorModal(ref) {
			this.minecraftAuthErrorModal = ref
		},
		showError(error, context, closable = true, source = null) {
			if (
				error.message &&
				error.message.includes('Minecraft authentication error:') &&
				this.minecraftAuthErrorModal
			) {
				this.minecraftAuthErrorModal.show(error)
				return
			}
			this.errorModal.show(error, context, closable, source)
		},
	},
})

export const handleSevereError = (err, context) => {
	const error = useError()
	error.showError(err, context)
	console.error(err)
}
