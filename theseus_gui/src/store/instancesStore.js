import { defineStore } from 'pinia'
import { ofetch } from 'ofetch'
import generated from '@/generated'

export const useInstances = defineStore('instanceStore', {
  state: () => ({
    fakeInstances: [],
    instances: [],
    searchInput: '',
    totalHits: 0,
    currentPage: 1,
    pageCount: 1,
    offset: 0,
    filter: 'Relevance',
    categories: {
      adventure: { label: 'Adventure', enabled: false },
      cursed: { label: 'Cursed', enabled: false },
      decoration: { label: 'Decoration', enabled: false },
      economy: { label: 'Economy', enabled: false },
      equipment: { label: 'Equipment', enabled: false },
      food: { label: 'Food', enabled: false },
      gameMechanics: { label: 'Game Mechanics', enabled: false },
      library: { label: 'Library', enabled: false },
      magic: { label: 'Magic', enabled: false },
      management: { label: 'Management', enabled: false },
      minigame: { label: 'Minigame', enabled: false },
      mobs: { label: 'Mobs', enabled: false },
      optimization: { label: 'Optimization', enabled: false },
      social: { label: 'Social', enabled: false },
      storage: { label: 'Storage', enabled: false },
      technology: { label: 'Technology', enabled: false },
      transportation: { label: 'Transportation', enabled: false },
      utility: { label: 'Utility', enabled: false },
      worldGeneration: { label: 'World Generation', enabled: false },
    },
    loaders: {
      fabric: { label: 'Fabric', enabled: false },
      forge: { label: 'Forge', enabled: false },
      quilt: { label: 'Quilt', enabled: false },
      rift: { label: 'Rift', enabled: false },
      liteLoader: { label: 'LiteLoader', enabled: false },
      risugami: { label: 'Risugami ModLoader', enabled: false },
    },
    environments: {
      client: false,
      server: false,
    },
    activeVersions: [],
    openSource: false,
    limit: 20,
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
      const activeCategories = Object.keys(this.categories).filter(
        (cat) => this.categories[cat].enabled === true
      )
      const activeLoaders = Object.keys(this.loaders).filter(
        (loader) => this.loaders[loader].enabled === true
      )
      const activeEnvs = Object.keys(this.environments).filter(
        (env) => this.environments[env] === true
      )

      let facets = ''
      if (
        activeCategories.length > 0 ||
        activeLoaders.length > 0 ||
        activeEnvs.length > 0 ||
        this.activeVersions.length > 0 ||
        this.openSource === true
      ) {
        facets = '&facets=['
        activeCategories.forEach((cat) => (facets += `["categories:${cat}"],`))
        activeLoaders.forEach((loader) => (facets += `["categories:${loader}"],`))
        this.activeVersions.forEach((ver) => (facets += `["versions:${ver}"],`))

        if (this.environments.client === true) {
          facets +=
            '["client_side:optional"],["client_side:required"],["server_side:optional"],["server_side:unsupported"],'
        }
        if (this.environments.server === true) {
          facets +=
            '["server_side:optional"],["server_side:required"],["client_side:optional"],["client_side:unsupported"],'
        }

        if (this.openSource === true) facets += '["open_source:true"],'

        facets = facets.slice(0, facets.length - 1)
        facets += ']'
      }

      let indexSort
      switch (this.filter) {
        case 'Download count':
          indexSort = 'downloads'
          break
        case 'Follow count':
          indexSort = 'follows'
          break
        case 'Recently published':
          indexSort = 'newest'
          break
        case 'Recently updated':
          indexSort = 'updated'
          break
        default:
          indexSort = 'relevance'
      }

      const response = await ofetch(
        `https://api.modrinth.com/v2/search?query=${this.searchInput || ''}&limit=${
          this.limit
        }&offset=${this.offset || 0}${facets || ''}&index=${indexSort}`
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
    toggleCategory(cat) {
      this.categories[cat] = !this.categories[cat]
    },
    toggleLoader(loader) {
      this.loaders[loader] = !this.loaders[loader]
    },
    toggleEnv(env) {
      this.environments[env] = !this.environments[env]
    },
    setVersions(versions) {
      this.activeVersions = versions
    },
    setLimit(newLimit) {
      this.limit = newLimit
    },
    resetFilters() {
      Object.keys(this.categories).forEach((cat) => {
        this.categories[cat].enabled = false
      })
      Object.keys(this.loaders).forEach((loader) => {
        this.loaders[loader].enabled = false
      })
      Object.keys(this.environments).forEach((env) => {
        this.environments[env] = false
      })
      this.activeVersions = []
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
          let iconObj = generated.categories.find((c) => {
            if (c.name === cat) return c
          })

          // If an icon wasn't found in categories, search the loaders array
          if (!iconObj) {
            iconObj = generated.loaders.find((l) => {
              if (l.name === cat) return l
            })
          }

          // Push the category for display if an icon was found
          results.push({ name: cat, icon: iconObj?.icon })
        })

        return results
      }
    },
    getIconByFilter: (_) => {
      return (filter) => {
        let iconObj = generated.categories.find((c) => {
          if (c.name === filter) return c
        })

        if (iconObj) return iconObj.icon

        iconObj = generated.loaders.find((l) => {
          if (l.name === filter) return l
        })

        if (iconObj) return iconObj.icon

        if (filter === 'client')
          return '<svg data-v-d754391f="" xmlns="http://www.w3.org/2000/svg" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17 9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2z"></path></svg>'

        if (filter === 'server')
          return '<svg data-v-d754391f="" xmlns="http://www.w3.org/2000/svg" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" aria-hidden="true"><path d="M22 12H2M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11zM6 16h.01M10 16h.01"></path></svg>'

        if (filter === 'gameMechanics')
          return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="21" x2="4" y2="14"></line><line x1="4" y1="10" x2="4" y2="3"></line><line x1="12" y1="21" x2="12" y2="12"></line><line x1="12" y1="8" x2="12" y2="3"></line><line x1="20" y1="21" x2="20" y2="16"></line><line x1="20" y1="12" x2="20" y2="3"></line><line x1="1" y1="14" x2="7" y2="14"></line><line x1="9" y1="8" x2="15" y2="8"></line><line x1="17" y1="16" x2="23" y2="16"></line></svg>'
        if (filter === 'worldGeneration')
          return '<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>'

        if (filter === 'liteLoader')
          return '<svg clip-rule="evenodd" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="1.5" version="1.1" viewBox="0 0 24 24" xml:space="preserve" xmlns="http://www.w3.org/2000/svg"><rect width="24" height="24" fill="none"></rect><path d="m3.924 21.537s3.561-1.111 8.076-6.365c2.544-2.959 2.311-1.986 4-4.172" fill="none" stroke="currentColor" stroke-width="2px"></path><path d="m7.778 19s1.208-0.48 4.222 0c2.283 0.364 6.037-4.602 6.825-6.702 1.939-5.165 0.894-10.431 0.894-10.431s-4.277 4.936-6.855 7.133c-5.105 4.352-6.509 11-6.509 11" fill="none" stroke="currentColor" stroke-width="2px"></path></svg>'

        if (filter === 'risugami')
          return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" xml:space="preserve"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M1.4 18V6h3.8v1.5h1.5V9h1.5V7.5h1.5V6h3.8v12H9.7v-5.3H9v1.5H6v-1.5h-.8V18H1.4zm12.1 0V6h3.8v9h5.3v3h-9.1z"></path></svg>'
        return '<div />'
      }
    },
  },
})
