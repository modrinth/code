import { defineStore } from 'pinia'
import { ofetch } from 'ofetch'
import generated from '@/generated'
import { format } from 'prettier'

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
      //   adventure: { label: 'Adventure', enabled: false },
      //   cursed: { label: 'Cursed', enabled: false },
      //   decoration: { label: 'Decoration', enabled: false },
      //   economy: { label: 'Economy', enabled: false },
      //   equipment: { label: 'Equipment', enabled: false },
      //   food: { label: 'Food', enabled: false },
      //   'game-mechanics': { label: 'Game Mechanics', enabled: false },
      //   library: { label: 'Library', enabled: false },
      //   magic: { label: 'Magic', enabled: false },
      //   management: { label: 'Management', enabled: false },
      //   minigame: { label: 'Minigame', enabled: false },
      //   mobs: { label: 'Mobs', enabled: false },
      //   optimization: { label: 'Optimization', enabled: false },
      //   social: { label: 'Social', enabled: false },
      //   storage: { label: 'Storage', enabled: false },
      //   technology: { label: 'Technology', enabled: false },
      //   transportation: { label: 'Transportation', enabled: false },
      //   utility: { label: 'Utility', enabled: false },
      //   worldgen: { label: 'World Generation', enabled: false },
    },
    loaders: {
      // fabric: { label: 'Fabric', enabled: false },
      // forge: { label: 'Forge', enabled: false },
      // quilt: { label: 'Quilt', enabled: false },
      // rift: { label: 'Rift', enabled: false },
      // liteloader: { label: 'LiteLoader', enabled: false },
      // risugami: { label: 'Risugami ModLoader', enabled: false },
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
    initFacets() {
      var categories = generated.categories.filter((cat) => cat.project_type === 'modpack')

      var loaders = generated.loaders.filter((loader) => {
        // Some loaders don't have any supported proj types set
        if (!loader.supported_project_types) return
        if (loader.supported_project_types.includes('modpack')) return loader
      })

      categories.forEach((cat) => {
        this.categories[cat.name] = {
          icon: cat.icon,
          name: cat.name,
          enabled: false,
        }
      })

      loaders.forEach((loader) => {
        this.loaders[loader.name] = {
          icon: loader.icon,
          name: loader.name,
          enabled: false,
        }
      })
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

      let formattedFacets = ['project_type:modpack']
      let envFacets = []
      if (
        activeCategories.length > 0 ||
        activeLoaders.length > 0 ||
        activeEnvs.length > 0 ||
        this.activeVersions.length > 0 ||
        this.openSource === true
      ) {
        activeCategories.forEach((cat) => formattedFacets.push(`categories:${cat}`))
        activeCategories.forEach((cat) => formattedFacets.push(`categories:${cat}`))
        activeLoaders.forEach((loader) => formattedFacets.push(`categories:${loader}`))
        this.activeVersions.forEach((ver) => formattedFacets.push(`versions:${ver}`))

        // If both are on or off, adding filters is pointless
        if (this.environments.client === true && this.environments.server === false)
          formattedFacets.push('client_side:required')
        if (this.environments.client === false && this.environments.server === true)
          formattedFacets.push('server_side:required')

        if (this.openSource === true) formattedFacets.push('open_source:true')

        formattedFacets = [...formattedFacets, ...envFacets]
      }

      let facets = '&facets=['
      formattedFacets.forEach((facet) => (facets += `["${facet}"],`))
      facets = facets.slice(0, facets.length - 1)
      facets += ']'

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

        if (filter === 'risugami')
          return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" xml:space="preserve"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M1.4 18V6h3.8v1.5h1.5V9h1.5V7.5h1.5V6h3.8v12H9.7v-5.3H9v1.5H6v-1.5h-.8V18H1.4zm12.1 0V6h3.8v9h5.3v3h-9.1z"></path></svg>'
        return '<div />'
      }
    },
  },
})
