import { defineStore } from 'pinia'

export const useBreadcrumbs = defineStore('breadcrumbsStore', {
  state: () => ({
    names: new Map(),
    context: null,
    rootContext: null,
  }),
  actions: {
    getName(route: string) {
      return this.names.get(route) ?? route
    },
    setName(route: string, title: string) {
      this.names.set(route, title)
    },
    setContext(context) {
      this.context = context
    },
    setRootContext(context) {
      this.rootContext = context
    },
  },
})
