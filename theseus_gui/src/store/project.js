import { defineStore } from 'pinia'

export const useProjectStore = defineStore({
  id: 'project',

  state: () => ({
    project: null,
  }),

  actions: {
    async fetchProject(id) {
      const apiUrl = `https://api.modrinth.com/v2/project/${id}`

      const response = await fetch(apiUrl)
      const data = await response.json()

      this.project = data
    },
  },
})
