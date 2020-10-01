<template>
  <div class="columns">
    <div class="content column-grow-5">
      <h2>Mods</h2>
      <section class="search-bar">
        <div class="iconified-input column-grow-2">
          <input
            id="search"
            v-model="query"
            type="search"
            name="search"
            placeholder="Search mods"
            @input="onSearchChange(1)"
          />
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
        </div>
        <div class="sort-paginate">
          <Multiselect
            v-model="sortType"
            class="sort-types"
            placeholder="Select one"
            track-by="display"
            label="display"
            :options="sortTypes"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
            @input="onSearchChange(1)"
          >
            <template slot="singleLabel" slot-scope="{ option }">{{
              option.display
            }}</template>
          </Multiselect>
          <div class="mobile-filters-button">
            <button @click="toggleFiltersMenu">Filter...</button>
          </div>
        </div>
        <pagination
          :current-page="currentPage"
          :pages="pages"
          @switch-page="onSearchChange"
        ></pagination>
      </section>
      <div class="results column-grow-4">
        <SearchResult
          v-for="(result, index) in results"
          :id="result.mod_id"
          :key="result.mod_id"
          :author="result.author"
          :name="result.title"
          :description="result.description"
          :latest-version="result.versions[0]"
          :created-at="result.date_created.substring(0, 10)"
          :updated-at="result.date_modified.substring(0, 10)"
          :downloads="formatNumber(result.downloads)"
          :icon-url="result.icon_url"
          :author-url="result.author_url"
          :page-url="result.page_url"
          :categories="result.categories"
          :is-ad="index % 5"
        />
      </div>
      <section v-if="pages.length > 1" class="search-bottom">
        <Multiselect
          v-model="maxResults"
          class="max-results"
          placeholder="Select one"
          :options="[5, 10, 15, 20, 50, 100]"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
          @input="onSearchChange(currentPage)"
        >
        </Multiselect>
        <pagination
          :current-page="currentPage"
          :pages="pages"
          @switch-page="onSearchChangeToTop"
        ></pagination>
      </section>
    </div>
    <section id="filters" class="filters">
      <!--#region filters  -->
      <div class="filters-wrapper">
        <section class="filter-group">
          <button class="filter-button-done" @click="toggleFiltersMenu">
            Done
          </button>
          <button @click="clearFilters">Clear Filters</button>
          <h3>Categories</h3>
          <SearchFilter
            :active-filters="facets"
            display-name="Technology"
            facet-name="categories:technology"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <line x1="22" y1="12" x2="2" y2="12"></line>
              <path
                d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"
              ></path>
              <line x1="6" y1="16" x2="6.01" y2="16"></line>
              <line x1="10" y1="16" x2="10.01" y2="16"></line>
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Adventure"
            facet-name="categories:adventure"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10"></circle>
              <polygon
                points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"
              ></polygon>
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Magic"
            facet-name="categories:magic"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M10.42 3C9.88 9.2 4 9.4 4 14.38c0 2.8 1.7 5.35 4.17 6.62.76-2.1 3.83-3.17 3.83-5.57a7.65 7.65 0 013.92 5.52C24.13 15.88 18.9 6.18 10.42 3z"
              />
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Utility"
            facet-name="categories:utility"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="2" y="7" width="20" height="14" rx="2" ry="2" />
              <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" />
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Decoration"
            facet-name="categories:decoration"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
              <polyline points="9 22 9 12 15 12 15 22" />
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Library"
            facet-name="categories:library"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
              <path
                d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"
              ></path>
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Cursed"
            facet-name="categories:cursed"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="7" y="7.5" width="10" height="14" rx="5" />
              <polyline points="2 12.5 4 14.5 7 14.5" />
              <polyline points="22 12.5 20 14.5 17 14.5" />
              <polyline points="3 21.5 5 18.5 7 17.5" />
              <polyline points="21 21.5 19 18.5 17 17.5" />
              <polyline points="3 8.5 5 10.5 7 11.5" />
              <polyline points="21 8.5 19 10.5 17 11.5" />
              <line x1="12" y1="7.5" x2="12" y2="21.5" />
              <path
                d="M15.38,8.82A3,3,0,0,0,16,7h0a3,3,0,0,0-3-3H11A3,3,0,0,0,8,7H8a3,3,0,0,0,.61,1.82"
              />
              <line x1="9" y1="4.5" x2="8" y2="2.5" />
              <line x1="15" y1="4.5" x2="16" y2="2.5" />
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Worldgen"
            facet-name="categories:worldgen"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M12 2v6.5M10 4l2 1 2-1M3.3 7L9 10.2m-4.9-.5L6 8.5l.1-2.2M3.3 17L9 13.7m-2.9 4L6 15.5l-1.9-1.2M12 22v-6.5m2 4.5l-2-1-2 1m5-6.2l5.6 3.3m-.7-2.8L18 15.5l-.1 2.2M20.7 7L15 10.3m2.9-4l.1 2.2 1.9 1.2M12 8.5l3 1.8v3.5l-3 1.8-3-1.8v-3.5l3-1.8z"
              />
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Storage"
            facet-name="categories:storage"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="21 8 21 21 3 21 3 8"></polyline>
              <rect x="1" y="3" width="22" height="5"></rect>
              <line x1="10" y1="12" x2="14" y2="12"></line>
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Food"
            facet-name="categories:food"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 8h1a4 4 0 0 1 0 8h-1"></path>
              <path d="M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z"></path>
              <line x1="6" y1="1" x2="6" y2="4"></line>
              <line x1="10" y1="1" x2="10" y2="4"></line>
              <line x1="14" y1="1" x2="14" y2="4"></line>
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Equipment"
            facet-name="categories:equipment"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M17.573 20.038L3.849 7.913 2.753 2.755 7.838 4.06 19.47 18.206l-1.898 1.832z"
              />
              <path d="M7.45 14.455l-3.043 3.661 1.887 1.843 3.717-3.25" />
              <path
                d="M16.75 10.82l3.333-2.913 1.123-5.152-5.091 1.28-2.483 2.985"
              />
              <path d="M21.131 16.602l-5.187 5.01 2.596-2.508 2.667 2.761" />
              <path d="M2.828 16.602l5.188 5.01-2.597-2.508-2.667 2.761" />
            </svg>
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Misc"
            facet-name="categories:misc"
            @toggle="toggleFacet"
          >
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="2" y1="12" x2="22" y2="12"></line>
              <path
                d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
              ></path>
            </svg>
          </SearchFilter>
          <h3>Loaders</h3>
          <SearchFilter
            :active-filters="facets"
            display-name="Forge"
            facet-name="categories:forge"
            @toggle="toggleFacet"
          >
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Fabric"
            facet-name="categories:fabric"
            @toggle="toggleFacet"
          >
          </SearchFilter>
          <h3>Platforms</h3>
          <SearchFilter
            :active-filters="facets"
            display-name="Modrinth"
            facet-name="host:modrinth"
            @toggle="toggleFacet"
          >
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Curseforge"
            facet-name="host:curseforge"
            @toggle="toggleFacet"
          >
          </SearchFilter>
          <h3>Versions</h3>
        </section>
        <multiselect
          v-model="selectedVersions"
          :options="versions"
          :loading="versions.length === 0"
          :multiple="true"
          :searchable="true"
          :show-no-results="false"
          :close-on-select="false"
          :clear-on-select="false"
          :show-labels="false"
          :limit="6"
          :hide-selected="true"
          placeholder="Choose versions..."
          @input="onSearchChange(1)"
        ></multiselect>
      </div>
      <!--#endregion -->
    </section>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import axios from 'axios'
import SearchResult from '@/components/ModResult'
import Pagination from '@/components/Pagination'
import SearchFilter from '@/components/SearchFilter'

const config = {
  headers: {
    Accept: 'application/json',
  },
}

export default {
  components: {
    SearchResult,
    Pagination,
    Multiselect,
    SearchFilter,
  },
  data() {
    return {
      query: '',
      selectedVersions: [],
      versions: [],
      facets: [],
      results: [],
      pages: [],
      currentPage: 1,
      sortTypes: [
        { display: 'Relevance', name: 'relevance' },
        { display: 'Total Downloads', name: 'downloads' },
        { display: 'Newest', name: 'newest' },
        { display: 'Updated', name: 'updated' },
      ],
      sortType: { display: 'Relevance', name: 'relevance' },
      maxResults: 5,
    }
  },
  async created() {
    if (this.$route.query.q) this.query = this.$route.query.q
    if (this.$route.query.f) {
      const facets = this.$route.query.f.split(',')

      for (const facet of facets) await this.toggleFacet(facet, false)
    }
    if (this.$route.query.v)
      this.selectedVersions = this.$route.query.v.split(',')
    if (this.$route.query.s) {
      this.sortType = this.$route.query.s
    }
    if (this.$route.query.m) {
      this.maxResults = this.$route.query.m
    }
    if (this.$route.query.o)
      this.currentPage = Math.ceil(this.$route.query.o / this.maxResults) + 1

    await this.fillInitialVersions()
    await this.onSearchChange(this.currentPage)
  },
  methods: {
    async fillInitialVersions() {
      try {
        const res = await axios.get(
          'https://launchermeta.mojang.com/mc/game/version_manifest.json',
          config
        )

        const versions = res.data.versions
        const betaVersions = []
        const legacyVersions = []
        for (const version of versions) {
          if (version.type === 'release') this.versions.push(version.id)
          if (version.type === 'snapshot') betaVersions.push(version.id)
          if (version.type === 'old_beta' || version.type === 'old_alpha')
            legacyVersions.push(version.id)
        }
        this.versions = this.versions.concat(betaVersions, legacyVersions)
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(err)
      }
    },
    async clearFilters() {
      for (const facet of [...this.facets]) await this.toggleFacet(facet, true)

      this.selectedVersions = []
      await this.onSearchChange(1)
    },
    async toggleFacet(elementName, sendRequest) {
      const index = this.facets.indexOf(elementName)

      if (index !== -1) {
        this.facets.splice(index, 1)
      } else {
        this.facets.push(elementName)
      }

      if (!sendRequest) await this.onSearchChange(1)
    },
    async onSearchChangeToTop(newPageNumber) {
      if (process.client) window.scrollTo(0, 0)

      await this.onSearchChange(newPageNumber)
    },
    async onSearchChange(newPageNumber) {
      try {
        const params = [
          `limit=${this.maxResults}`,
          `index=${this.sortType.name}`,
        ]

        if (this.query.length > 0) {
          params.push(`query=${this.query.replace(/ /g, '+')}`)
        }

        if (this.facets.length > 0 || this.selectedVersions.length > 0) {
          const formattedFacets = []
          for (const facet of this.facets) {
            formattedFacets.push([facet])
          }

          if (this.selectedVersions.length > 0) {
            const versionFacets = []
            for (const facet of this.selectedVersions) {
              versionFacets.push('versions:' + facet)
            }
            formattedFacets.push(versionFacets)
          }

          params.push(`facets=${JSON.stringify(formattedFacets)}`)
        }

        const offset = (newPageNumber - 1) * this.maxResults
        if (newPageNumber !== 1) {
          params.push(`offset=${offset}`)
        }

        let url = 'https://api.modrinth.com/api/v1/mod'

        if (params.length > 0) {
          for (let i = 0; i < params.length; i++) {
            url += i === 0 ? `?${params[i]}` : `&${params[i]}`
          }
        }

        const res = await axios.get(url, config)
        this.results = res.data.hits

        const pageAmount = Math.ceil(res.data.total_hits / res.data.limit)

        this.currentPage = newPageNumber
        if (pageAmount > 4) {
          if (this.currentPage + 1 >= pageAmount) {
            this.pages = [
              1,
              pageAmount - 3,
              pageAmount - 2,
              pageAmount - 1,
              pageAmount,
            ]
          } else if (this.currentPage > 4) {
            this.pages = [
              1,
              this.currentPage - 1,
              this.currentPage,
              this.currentPage + 1,
              pageAmount,
            ]
          } else {
            this.pages = [1, 2, 3, 4, pageAmount]
          }
        } else {
          this.pages = Array.from({ length: pageAmount }, (_, i) => i + 1)
        }

        if (process.client) {
          url = `mods?q=${encodeURIComponent(this.query)}`

          if (offset > 0) url += `&o=${offset}`
          if (this.facets.length > 0)
            url += `&f=${encodeURIComponent(this.facets)}`
          if (this.selectedVersions.length > 0)
            url += `&v=${encodeURIComponent(this.selectedVersions)}`
          if (this.sortType.name !== 'relevance')
            url += `&s=${encodeURIComponent(this.sortType.name)}`
          if (this.maxResults > 5)
            url += `&m=${encodeURIComponent(this.maxResults)}`

          window.history.pushState(new Date(), 'Mods', url)
        }
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(err)
      }
    },
    formatNumber(x) {
      return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    },
    toggleFiltersMenu() {
      const filters = document.getElementById('filters')
      const currentlyActive = filters.className === 'filters active'
      filters.className = `filters${currentlyActive ? '' : ' active'}`
      document.body.style.overflow =
        document.body.style.overflow !== 'hidden' ? 'hidden' : 'auto'
    },
  },
}
</script>

<style src="vue-multiselect/dist/vue-multiselect.min.css"></style>
<style lang="scss">
.search-bar {
  align-items: center;
  display: flex;
  justify-content: space-between;
  flex-flow: column;
  .iconified-input {
    width: 100%;
  }
  .sort-paginate {
    display: flex;
    width: 100%;
  }
  @media screen and (min-width: 900px) {
    flex-flow: row;
    .iconified-input {
      width: auto;
    }
    .sort-paginate {
      display: block;
      width: auto;
    }
  }
}

.search-bottom {
  align-items: center;
  display: flex;
  justify-content: flex-end;
  select {
    width: 100px;
    margin-right: 20px;
  }
}

.content {
  min-height: 96vh;
}

.mobile-filters-button {
  display: inline-block;
  button {
    background: var(--color-bg);
    color: var(--color-text);
    border: 2px solid var(--color-grey-3);
    border-radius: var(--size-rounded-sm);
    padding: 0.5rem;
  }

  // Hide button on larger screens where it's not needed
  @media screen and (min-width: 900px) {
    display: none;
  }
}

.filters {
  overflow-y: auto;
  background-color: var(--color-bg);
  border-left: 1px solid var(--color-grey-2);
  position: fixed;
  width: 100vw;
  right: -100vw;
  max-height: 100vh;
  min-width: 15%;
  top: 3.5rem;
  height: calc(100vh - 3.5rem);
  transition: right 150ms;
  flex-shrink: 0; // Stop shrinking when page contents change

  .filters-wrapper {
    padding: 0 0.75rem;
  }

  h3 {
    color: #718096;
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    text-transform: uppercase;
  }

  // Larger screens that don't need to collapse
  @media screen and (min-width: 900px) {
    position: sticky;
    width: 215px;
    padding-right: 1rem;
    transition: none;
  }

  // Desktop
  @media screen and (min-width: 1145px) {
    top: 0;
    height: 100vh;
  }

  &.active {
    right: 0;
  }
}

.filter-group {
  margin-top: 1em;

  button {
    width: 100%;
    padding: 5px 0;
    outline: none;
    color: var(--color-grey-5);
    background-color: var(--color-grey-1);
    border: none;
    border-radius: 5px;

    &:hover {
      background-color: var(--color-grey-2);
      color: var(--color-text);
    }
  }

  .filter-button-done {
    display: block;
  }

  // Large screens that don't collapse
  @media screen and (min-width: 900px) {
    .filter-button-done {
      display: none;
    }
  }

  // Desktop
  @media screen and (min-width: 1145px) {
    margin-top: 2em;
  }
}

.iconified-select {
  margin-left: 1em;
  align-items: center;
  display: inline-flex;
  flex-direction: row-reverse;

  select {
    padding-left: 2.5rem;

    &:hover {
      & + svg {
        color: var(--color-grey-6);
      }
    }

    &:focus {
      & + svg {
        color: var(--color-text);
      }
    }
  }

  svg {
    color: var(--color-grey-5);
    margin-right: -2rem;
    width: 24px;
    height: 24px;
  }
}

select {
  width: 220px;
  padding: 0.5rem 1rem;
  background: var(--color-bg);
  border: 2px solid var(--color-grey-3);
  border-radius: var(--size-rounded-sm);
  color: var(--color-grey-9);
  font-size: 1rem;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;

  &:hover {
    border-color: var(--color-grey-4);
    color: var(--color-text);
  }
}

.sort-types {
  min-width: 200px;
  padding-y: 1rem;
  border: 2px solid var(--color-grey-3);
  border-radius: var(--size-rounded-sm);

  .multiselect__tags {
    padding: 10px 50px 0 8px;
    border: none;
  }
}

.max-results {
  max-width: 80px;
}

.multiselect__content-wrapper {
  overflow-x: hidden;
}

.multiselect__tags,
.multiselect__spinner {
  background: var(--color-bg);
}

.multiselect__spinner::before,
.multiselect__spinner::after {
  border-top-color: var(--color-brand);
}

.multiselect__option--selected.multiselect__option--highlight,
.multiselect__option,
.multiselect__single,
.multiselect__input {
  color: var(--color-text);
  background: var(--color-bg);
}

.multiselect__option--highlight,
.multiselect__tag {
  background: var(--color-brand);
}
</style>
