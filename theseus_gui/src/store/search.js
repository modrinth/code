import { defineStore } from 'pinia'
import generated from '@/generated'

export const useSearch = defineStore('searchStore', {
  state: () => ({
    searchResults: [],
    searchInput: '',
    totalHits: 0,
    currentPage: 1,
    pageCount: 1,
    offset: 0,
    filter: 'Relevance',
    categories: {},
    loaders: {},
    environments: {
      client: false,
      server: false,
    },
    activeVersions: [],
    openSource: false,
    limit: 20,
  }),
  actions: {
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

      console.log(this.categories)
      console.log(this.loaders)
    },
    getQueryString() {
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

      return `?query=${this.searchInput || ''}&limit=${this.limit}&offset=${this.offset || 0}${
        facets || ''
      }&index=${indexSort}`
    },
    setSearchResults(response) {
      this.searchResults = [...response.hits]
      this.totalHits = response.total_hits
      this.offset = response.offset
      this.pageCount = Math.ceil(this.totalHits / this.limit)
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
    getCategoriesByResultId: (state) => {
      // Pulls all icons possible from generated.js for passing as props to ProjectCard
      return (id) => {
        let results = []
        const result = state.searchResults?.find((i) => i.project_id === id)

        result.categories?.forEach((cat) => {
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
  },
})
