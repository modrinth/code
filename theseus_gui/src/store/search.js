import { defineStore } from 'pinia'

export const useSearch = defineStore('searchStore', {
  state: () => ({
    searchResults: [],
    searchInput: '',
    totalHits: 0,
    currentPage: 1,
    pageCount: 1,
    offset: 0,
    filter: 'Relevance',
    projectType: '',
    facets: [],
    orFacets: [],
    environments: {
      client: false,
      server: false,
    },
    activeVersions: [],
    openSource: false,
    limit: 20,
    instanceContext: null,
    ignoreInstance: false,
  }),
  actions: {
    getQueryString() {
      let andFacets = [`project_type:${this.projectType === 'datapack' ? 'mod' : this.projectType}`]

      if (this.instanceContext && !this.ignoreInstance) {
        this.activeVersions = [this.instanceContext.metadata.game_version]
      }

      // Iterate through possible andFacets
      this.facets.forEach((facet) => {
        andFacets.push(facet)
      })
      // Add open source to andFacets if enabled
      if (this.openSource) andFacets.push('open_source:true')

      // Create andFacet string
      let formattedAndFacets = ''
      if (this.projectType === 'datapack') {
        ;[...andFacets, `categories:${encodeURIComponent('datapack')}`].forEach(
          (f) => (formattedAndFacets += `["${f}"],`)
        )
      } else if (this.instanceContext && !this.ignoreInstance && this.projectType === 'mod') {
        ;[
          ...andFacets,
          `categories:${encodeURIComponent(this.instanceContext.metadata.loader)}`,
        ].forEach((f) => (formattedAndFacets += `["${f}"],`))
      } else {
        andFacets.forEach((f) => (formattedAndFacets += `["${f}"],`))
      }
      formattedAndFacets = formattedAndFacets.slice(0, formattedAndFacets.length - 1)
      formattedAndFacets += ''

      // If orFacets are present, start building formatted orFacet filter
      let formattedOrFacets = ''
      if (this.orFacets.length > 0) {
        formattedOrFacets += '['
        this.orFacets.forEach((orF) => (formattedOrFacets += `"${orF}",`))
        formattedOrFacets = formattedOrFacets.slice(0, formattedOrFacets.length - 1)
        formattedOrFacets += '],'
      }

      // Snip normal orFacets and start version orFacets
      if (this.activeVersions.length > 0) {
        formattedOrFacets += '['
        this.activeVersions.forEach((ver) => (formattedOrFacets += `"versions:${ver}",`))
        formattedOrFacets = formattedOrFacets.slice(0, formattedOrFacets.length - 1)
        formattedOrFacets += '],'
      }

      // Add environments to orFacets if enabled
      if (this.environments.client)
        formattedOrFacets += '["client_side:optional","client_side:required"]]'
      if (this.environments.server)
        formattedOrFacets += '["server_side:optional","server_side:required"]]'

      formattedOrFacets = formattedOrFacets.slice(0, formattedOrFacets.length - 1)

      // Aggregate facet query string
      const facets = `&facets=[${formattedAndFacets}${
        formattedOrFacets.length > 0 ? `,${formattedOrFacets}` : ''
      }]`

      // Configure results sorting
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
    resetFilters() {
      this.facets = []
      this.orFacets = []
      Object.keys(this.environments).forEach((env) => {
        this.environments[env] = false
      })
      this.activeVersions = []
      this.openSource = false
    },
  },
})
