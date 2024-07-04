const startReport = (type: string, id: string) => {
  const prefill = new URLSearchParams()

  // type
  prefill.set('item', type)
  prefill.set('itemID', id)

  navigateTo('/report?' + prefill.toString())
}

export const reportProject = (id: string) => {
  return startReport('project', id)
}

export const reportVersion = (id: string) => {
  return startReport('version', id)
}

export const reportUser = (id: string) => {
  return startReport('user', id)
}
