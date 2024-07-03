export const useLoading = () => useState('loading', () => false)

export const startLoading = () => {
  const loading = useLoading()

  loading.value = true
}

export const stopLoading = () => {
  const loading = useLoading()

  loading.value = false
}
