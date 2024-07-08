import { defineStore } from 'pinia'

export const useError = defineStore('errorsStore', {
  state: () => ({
    errorModal: null,
  }),
  actions: {
    setErrorModal(ref) {
      this.errorModal = ref
    },
    showError(error) {
      this.errorModal.show(error)
    },
  },
})

export const handleSevereError = (err) => {
  const error = useError()
  error.showError(err)
  console.error(err)
}
