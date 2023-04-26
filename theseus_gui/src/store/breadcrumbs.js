import { defineStore } from 'pinia'

export const useBreadcrumbs = defineStore('breadcrumbsStore', {
  state: () => ({ names: new Map() }),
  actions: {
    getName(route) {
      return this.names.get(route) ?? route
    },
    setName(route, title) {
      this.names.set(route, title)
    },
  },
})
