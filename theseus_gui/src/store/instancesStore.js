import { defineStore } from 'pinia'
import { ofetch } from 'ofetch'
import categoryIcons from '@/categoryIcons'

export const useInstances = defineStore('instanceStore', {
  state: () => ({
    fakeInstances: [],
    instances: [],
    searchInput: '',
    totalHits: 0,
    currentPage: 1,
    pageCount: 1,
    offset: 0,
    filter: '',
  }),
  actions: {
    fetchInstances() {
      // Fetch from Tauri backend. We will repurpose this to get current instances, news, and popular packs. This action is distinct from the search action
      const instances = [
        {
          id: 1,
          name: 'Fabulously Optimized',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.18.1',
          downloads: 10,
          trending: true,
          img: 'https://cdn.modrinth.com/user/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613.jpeg',
        },
        {
          id: 2,
          name: 'New Caves',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.18 ',
          downloads: 8,
          trending: true,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 3,
          name: 'All the Mods 6',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.16.5',
          downloads: 4,
          trending: true,
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
        {
          id: 4,
          name: 'Bees',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 5,
          name: 'SkyFactory 4',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 1000,
          trending: false,
          img: 'https://cdn.modrinth.com/user/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613.jpeg',
        },
        {
          id: 6,
          name: 'RLCraft',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.12.2',
          downloads: 10000,
          trending: false,
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
        {
          id: 7,
          name: 'Regrowth',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.7.10',
          downloads: 1000,
          trending: false,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 8,
          name: 'Birds',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://avatars.githubusercontent.com/u/83074853?v=4',
        },
        {
          id: 9,
          name: 'Dogs',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://cdn.modrinth.com/user/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613.jpeg',
        },
        {
          id: 10,
          name: 'Cats',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://cdn.modrinth.com/data/ssUbhMkL/icon.png',
        },
        {
          id: 11,
          name: 'Rabbits',
          description: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit',
          version: '1.15.2',
          downloads: 9,
          trending: false,
          img: 'https://avatars1.githubusercontent.com/u/6166773?v=4',
        },
      ]

      this.fakeInstances = [...instances]
    },
    async searchInstances() {
      const response = await ofetch(
        `https://staging-api.modrinth.com/v2/search?query=${this.searchInput || ''}&offset=${
          this.offset || 0
        }`
      )
      this.instances = [...response.hits]
      this.totalHits = response.total_hits
      this.offset = response.offset
      this.pageCount = Math.ceil(this.totalHits / 10)
    },
    setSearchInput(newInput) {
      this.searchInput = newInput
    },
    setCurrentPage(newPage) {
      this.currentPage = newPage

      if (newPage === 1) this.offset = 0
      else this.offset = this.currentPage * 10 - 10
    },
    setFilter(newFilter) {
      this.filter = newFilter
    },
  },
  getters: {
    getFilteredInstances: (state) => {
      const filteredInstances = state.fakeInstances.filter((i) => {
        // When time comes, do more advanced client-side filtering here if wise
        const normalizedInstanceName = i.name?.toLowerCase()
        if (normalizedInstanceName.includes(state.filter.toLowerCase())) return i
      })

      if (filteredInstances && filteredInstances.length > 0) return filteredInstances

      return state.fakeInstances
    },
    getCategoriesByInstanceId: (state) => {
      return (id) => {
        let results = []
        const instance = state.instances?.find((i) => i.project_id === id)

        instance.categories?.forEach((cat) => {
          // First look for an icon in the categories array
          let iconObj = categoryIcons.categories.find((c) => {
            if (c.name === cat) return c
          })

          // If an icon wasn't found in categories, search the loaders array
          if (!iconObj) {
            iconObj = categoryIcons.loaders.find((l) => {
              if (l.name === cat) return l
            })
          }

          // Push the category for display if an icon was found
          results.push({ name: cat, icon: iconObj?.icon })
        })

        return results
      }
    },
  },
})
