import { defineStore } from 'pinia'

export const useError = defineStore('errorsStore', {
  state: () => ({
    errorModal: null,
  }),
  actions: {
    setErrorModal(ref) {
      this.errorModal = ref
    },
    showError(error, context, closable = true, source = null) {
      this.errorModal.show(error, context, closable, source)
    },
  },
})

export const handleSevereError = (err, context) => {
  const error = useError()
  error.showError(err, context)
  console.error(err)
}
