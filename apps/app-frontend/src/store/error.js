import { defineStore } from 'pinia'

export const useError = defineStore('errorsStore', {
  state: () => ({
    errorModal: null,
  }),
  actions: {
    setErrorModal(ref) {
      this.errorModal = ref
    },
    showError(error, closable = true, source = null) {
      this.errorModal.show(error, closable, source)
    },
  },
})

export const handleSevereError = (err) => {
  const error = useError()
  error.showError(err)
  console.error(err)
}
