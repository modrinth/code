import { defineStore } from 'pinia'
import { get_search_results } from '@/helpers/cache.js'

export const useFeaturedProjects = defineStore('featuredProjects', {
  state: () => ({
    modpacks: undefined,
    mods: undefined,
  }),
  actions: {
    async getFeaturedModpack(filter) {
      if (this.modpacks !== undefined && this.modpacks.length !== 0) {
        return this.modpacks
      }

      let offset = Math.floor(Math.random() * 100)

      const response = await get_search_results(
        `?facets=[["project_type:modpack"]]&limit=10&filters=${filter.value}&offset=${offset}`,
      )

      if (response) {
        this.modpacks = response.result.hits
      } else {
        this.modpacks = []
      }

      return this.modpacks
    },
    async getFeaturedMods() {
      if (this.mods !== undefined && this.mods.length !== 0) {
        return this.mods
      }

      let offset = Math.floor(Math.random() * 100)

      const response = await get_search_results(
        `?facets=[["project_type:mod"]]&limit=10&offset=${offset}`,
      )

      if (response) {
        this.mods = response.result.hits
      } else {
        this.mods = []
      }

      return this.mods
    },
  },
})
