<template>
  <div
    :class="{
      'normal-page': true,
      'alt-layout': $store.state.cosmetics.searchLayout,
    }"
  >
    <aside class="normal-page__sidebar" aria-label="Filters">
      <section class="card filters-card" role="presentation">
        <button
          class="iconified-button sidebar-menu-close-button"
          @click="sidebarMenuOpen = !sidebarMenuOpen"
        >
          <EyeOffIcon v-if="sidebarMenuOpen" aria-hidden="true" />
          <EyeIcon v-else aria-hidden="true" />
          {{ sidebarMenuOpen ? 'Hide filters' : 'Show filters' }}
        </button>
        <div
          class="sidebar-menu"
          :class="{ 'sidebar-menu_open': sidebarMenuOpen }"
        >
          <button
            :disabled="
              selectedLicenses.length === 0 &&
              selectedEnvironments.length === 0 &&
              selectedVersions.length === 0 &&
              facets.length === 0 &&
              orFacets.length === 0
            "
            class="iconified-button"
            @click="clearFilters"
          >
            <ClearIcon aria-hidden="true" />
            Clear filters
          </button>
          <section aria-label="Category filters">
            <h3
              v-if="
                $tag.categories.filter((x) => x.project_type === projectType)
                  .length > 0
              "
              class="sidebar-menu-heading"
            >
              Categories
            </h3>
            <SearchFilter
              v-for="category in $tag.categories.filter(
                (x) => x.project_type === projectType
              )"
              :key="category.name"
              :active-filters="facets"
              :display-name="category.name"
              :facet-name="`categories:${category.name}`"
              :icon="category.icon"
              @toggle="toggleFacet"
            />
          </section>
          <section aria-label="Loader filters">
            <h3
              v-if="
                $tag.loaders.filter((x) =>
                  x.supported_project_types.includes(projectType)
                ).length > 0
              "
              class="sidebar-menu-heading"
            >
              Loaders
            </h3>
            <SearchFilter
              v-for="loader in $tag.loaders.filter((x) => {
                if (
                  !showAllLoaders &&
                  x.name !== 'forge' &&
                  x.name !== 'fabric' &&
                  x.name !== 'quilt'
                ) {
                  return false
                }
                return x.supported_project_types.includes(projectType)
              })"
              :key="loader.name"
              :active-filters="orFacets"
              :display-name="
                loader.name === 'modloader' ? 'ModLoader' : loader.name
              "
              :facet-name="`categories:${loader.name}`"
              :icon="loader.icon"
              @toggle="toggleOrFacet"
            />
            <Checkbox
              v-model="showAllLoaders"
              :label="showAllLoaders ? 'Less' : 'More'"
              description="Show all loaders"
              style="margin-bottom: 0.5rem"
              :border="false"
              :collapsing-toggle-style="true"
            />
          </section>
          <section aria-label="Environment filters">
            <h3 class="sidebar-menu-heading">Environments</h3>
            <SearchFilter
              :active-filters="selectedEnvironments"
              display-name="Client"
              facet-name="client"
              @toggle="toggleEnv"
            >
              <ClientSide aria-hidden="true" />
            </SearchFilter>
            <SearchFilter
              :active-filters="selectedEnvironments"
              display-name="Server"
              facet-name="server"
              @toggle="toggleEnv"
            >
              <ServerSide aria-hidden="true" />
            </SearchFilter>
          </section>
          <h3 class="sidebar-menu-heading">Minecraft versions</h3>
          <Checkbox
            v-model="showSnapshots"
            label="Include snapshots"
            description="Include snapshots"
            style="margin-bottom: 0.5rem"
            :border="false"
          />
          <multiselect
            v-model="selectedVersions"
            :options="
              showSnapshots
                ? $tag.gameVersions.map((x) => x.version)
                : $tag.gameVersions
                    .filter((it) => it.version_type === 'release')
                    .map((x) => x.version)
            "
            :multiple="true"
            :searchable="true"
            :show-no-results="false"
            :close-on-select="false"
            :clear-search-on-select="false"
            :show-labels="false"
            :selectable="() => selectedVersions.length <= 6"
            placeholder="Choose versions..."
            @input="onSearchChange(1)"
          ></multiselect>
          <h3 class="sidebar-menu-heading">Licenses</h3>
          <Multiselect
            v-model="selectedLicenses"
            placeholder="Choose licenses..."
            :loading="$tag.licenses.length === 0"
            :options="$tag.licenses.map((x) => x.short.toUpperCase())"
            :multiple="true"
            :searchable="true"
            :close-on-select="false"
            :show-labels="false"
            :allow-empty="true"
            @input="onSearchChange(1)"
          />
        </div>
      </section>
    </aside>
    <section class="normal-page__content">
      <div
        v-if="projectType === 'modpack'"
        class="card warning"
        aria-label="Warning"
      >
        Modpack support is currently in alpha, and you may encounter issues. Our
        documentation includes instructions on
        <a
          href="https://docs.modrinth.com/docs/modpacks/playing_modpacks/"
          target="_blank"
          >playing modpacks</a
        >
        with
        <a href="https://atlauncher.com/about" target="_blank">ATLauncher</a>,
        <a href="https://multimc.org/" target="_blank">MultiMC</a>, and
        <a href="https://polymc.org" target="_blank">PolyMC</a>. Pack creators
        can reference our documentation on
        <a
          href="https://docs.modrinth.com/docs/modpacks/creating_modpacks/"
          target="_blank"
          >creating modpacks</a
        >. Join us on
        <a href="https://discord.gg/EUHuJHt" target="_blank">Discord</a>
        for support.
      </div>
      <div class="card search-controls">
        <div class="iconified-input">
          <label class="hidden" for="search">Search</label>
          <SearchIcon aria-hidden="true" />
          <input
            id="search"
            v-model="query"
            type="search"
            name="search"
            placeholder="Search..."
            autocomplete="off"
            @input="onSearchChange(1)"
          />
        </div>
        <div class="sort-controls">
          <div class="labeled-control">
            <span class="labeled-control__label">Sort by</span>
            <Multiselect
              v-model="sortType"
              placeholder="Select one"
              class="search-controls__sorting labeled-control__control"
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
          </div>
          <div class="labeled-control">
            <span class="labeled-control__label">Show per page</span>
            <Multiselect
              v-model="maxResults"
              placeholder="Select one"
              class="labeled-control__control"
              :options="[5, 10, 15, 20, 50, 100]"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
              @input="onSearchChange(currentPage)"
            />
          </div>
        </div>
      </div>
      <pagination
        :current-page="currentPage"
        :pages="pages"
        @switch-page="onSearchChange"
      ></pagination>
      <div>
        <Advertisement
          type="banner"
          small-screen="square"
          ethical-ads-small
          ethical-ads-big
        />
        <div v-if="$fetchState.pending" class="no-results">
          <LogoAnimated aria-hidden="true" />
          <p>Loading...</p>
        </div>
        <div v-else id="search-results" role="list" aria-label="Search results">
          <SearchResult
            v-for="result in results"
            :id="result.slug ? result.slug : result.project_id"
            :key="result.project_id"
            :type="result.project_type"
            :author="result.author"
            :name="result.title"
            :description="result.description"
            :created-at="result.date_created"
            :updated-at="result.date_modified"
            :downloads="result.downloads.toString()"
            :follows="result.follows.toString()"
            :icon-url="result.icon_url"
            :client-side="result.client_side"
            :server-side="result.server_side"
            :categories="result.categories"
          />
          <div v-if="results && results.length === 0" class="no-results">
            <p>No results found for your query!</p>
          </div>
        </div>
      </div>
      <pagination
        :current-page="currentPage"
        :pages="pages"
        @switch-page="onSearchChangeToTop"
      ></pagination>
    </section>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import SearchResult from '~/components/ui/ProjectCard'
import Pagination from '~/components/ui/Pagination'
import SearchFilter from '~/components/ui/search/SearchFilter'
import LogoAnimated from '~/components/ui/search/LogoAnimated'
import Checkbox from '~/components/ui/Checkbox'

import ClientSide from '~/assets/images/categories/client.svg?inline'
import ServerSide from '~/assets/images/categories/server.svg?inline'

import SearchIcon from '~/assets/images/utils/search.svg?inline'
import ClearIcon from '~/assets/images/utils/clear.svg?inline'
import EyeIcon from '~/assets/images/utils/eye.svg?inline'
import EyeOffIcon from '~/assets/images/utils/eye-off.svg?inline'

import Advertisement from '~/components/ads/Advertisement'

export default {
  auth: false,
  components: {
    Advertisement,
    SearchResult,
    Pagination,
    Multiselect,
    SearchFilter,
    Checkbox,
    ClientSide,
    ServerSide,
    SearchIcon,
    ClearIcon,
    EyeIcon,
    EyeOffIcon,
    LogoAnimated,
  },
  data() {
    return {
      query: '',

      selectedLicenses: [],

      showSnapshots: false,
      selectedVersions: [],

      selectedEnvironments: [],

      facets: [],
      orFacets: [],
      results: null,
      pages: [],
      currentPage: 1,

      projectType: 'mod',

      sortTypes: [
        { display: 'Relevance', name: 'relevance' },
        { display: 'Download count', name: 'downloads' },
        { display: 'Follow count', name: 'follows' },
        { display: 'Recently created', name: 'newest' },
        { display: 'Recently updated', name: 'updated' },
      ],
      sortType: { display: 'Relevance', name: 'relevance' },

      maxResults: 20,

      sidebarMenuOpen: false,
      showAllLoaders: false,

      skipLink: '#search-results',
    }
  },
  async fetch() {
    if (this.$route.query.q) this.query = this.$route.query.q
    if (this.$route.query.f) {
      const facets = this.$route.query.f.split(',')

      for (const facet of facets) await this.toggleFacet(facet, true)
    }
    if (this.$route.query.g) {
      const orFacets = this.$route.query.g.split(',')

      for (const orFacet of orFacets) await this.toggleOrFacet(orFacet, true)
    }
    if (this.$route.query.v)
      this.selectedVersions = this.$route.query.v.split(',')
    if (this.$route.query.l)
      this.selectedLicenses = this.$route.query.l.split(',')
    if (this.$route.query.h) this.showSnapshots = this.$route.query.h === 'true'
    if (this.$route.query.e)
      this.selectedEnvironments = this.$route.query.e.split(',')
    if (this.$route.query.s) {
      this.sortType.name = this.$route.query.s

      switch (this.sortType.name) {
        case 'relevance':
          this.sortType.display = 'Relevance'
          break
        case 'downloads':
          this.sortType.display = 'Downloads'
          break
        case 'newest':
          this.sortType.display = 'Recently created'
          break
        case 'updated':
          this.sortType.display = 'Recently updated'
          break
        case 'follows':
          this.sortType.display = 'Follow count'
          break
      }
    }
    if (this.$route.query.m) {
      this.maxResults = this.$route.query.m
    }
    if (this.$route.query.o)
      this.currentPage = Math.ceil(this.$route.query.o / this.maxResults) + 1

    this.projectType = this.$route.name.substring(
      0,
      this.$route.name.length - 1
    )

    await this.onSearchChange(this.currentPage)
  },
  watch: {
    '$route.path': {
      async handler() {
        this.projectType = this.$route.name.substring(
          0,
          this.$route.name.length - 1
        )

        this.results = null
        this.pages = []
        this.currentPage = 1
        this.query = ''
        this.maxResults = 20
        this.sortType = { display: 'Relevance', name: 'relevance' }

        await this.clearFilters()
      },
    },
  },
  created() {
    // This is currently using the global event bus as I couldn't figure out how to use the local one
    this.$nuxt.$emit('registerSkipLink', {
      id: '#search-results',
      text: 'Skip to Search Results',
    })
  },
  destroyed() {
    // Not sure about this
    this.$nuxt.$emit('registerSkipLink')
  },
  methods: {
    async clearFilters() {
      for (const facet of [...this.facets]) await this.toggleFacet(facet, true)
      for (const facet of [...this.orFacets])
        await this.toggleOrFacet(facet, true)

      this.selectedLicenses = []
      this.selectedVersions = []
      this.selectedEnvironments = []
      await this.onSearchChange(1)
    },
    async toggleFacet(elementName, doNotSendRequest) {
      const index = this.facets.indexOf(elementName)
      if (index !== -1) {
        this.facets.splice(index, 1)
      } else {
        this.facets.push(elementName)
      }

      if (!doNotSendRequest) await this.onSearchChange(1)
    },
    async toggleOrFacet(elementName, doNotSendRequest) {
      const index = this.orFacets.indexOf(elementName)
      if (index !== -1) {
        this.orFacets.splice(index, 1)
      } else {
        this.orFacets.push(elementName)
      }

      if (!doNotSendRequest) await this.onSearchChange(1)
    },
    async toggleEnv(environment, sendRequest) {
      const index = this.selectedEnvironments.indexOf(environment)
      if (index !== -1) {
        this.selectedEnvironments.splice(index, 1)
      } else {
        this.selectedEnvironments.push(environment)
      }

      if (!sendRequest) await this.onSearchChange(1)
    },
    async onSearchChangeToTop(newPageNumber) {
      if (process.client) window.scrollTo({ top: 0, behavior: 'smooth' })

      await this.onSearchChange(newPageNumber)
    },
    async onSearchChange(newPageNumber) {
      if (this.query === null) return

      try {
        const params = [
          `limit=${this.maxResults}`,
          `index=${this.sortType.name}`,
        ]

        if (this.query.length > 0) {
          params.push(`query=${this.query.replace(/ /g, '+')}`)
        }

        if (
          this.facets.length > 0 ||
          this.selectedVersions.length > 0 ||
          this.selectedEnvironments.length > 0 ||
          this.projectType
        ) {
          let formattedFacets = []
          for (const facet of this.facets) {
            formattedFacets.push([facet])
          }

          if (this.orFacets.length > 0) {
            formattedFacets.push(this.orFacets)
          }

          if (this.selectedVersions.length > 0) {
            const versionFacets = []
            for (const facet of this.selectedVersions) {
              versionFacets.push('versions:' + facet)
            }
            formattedFacets.push(versionFacets)
          }

          if (this.selectedLicenses.length > 0) {
            const licenseFacets = []
            for (const facet of this.selectedLicenses) {
              licenseFacets.push('license:' + facet.toLowerCase())
            }
            formattedFacets.push(licenseFacets)
          }

          if (this.selectedEnvironments.length > 0) {
            let environmentFacets = []

            const includesClient = this.selectedEnvironments.includes('client')
            const includesServer = this.selectedEnvironments.includes('server')
            if (includesClient && includesServer) {
              environmentFacets = [
                ['client_side:required'],
                ['server_side:required'],
              ]
            } else {
              if (includesClient) {
                environmentFacets = [
                  ['client_side:optional', 'client_side:required'],
                  ['server_side:optional', 'server_side:unsupported'],
                ]
              }
              if (includesServer) {
                environmentFacets = [
                  ['client_side:optional', 'client_side:unsupported'],
                  ['server_side:optional', 'server_side:required'],
                ]
              }
            }

            formattedFacets = [...formattedFacets, ...environmentFacets]
          }

          if (this.projectType)
            formattedFacets.push([`project_type:${this.projectType}`])

          params.push(`facets=${JSON.stringify(formattedFacets)}`)
        }

        const offset = (newPageNumber - 1) * this.maxResults
        if (newPageNumber !== 1) {
          params.push(`offset=${offset}`)
        }

        let url = 'search'

        if (params.length > 0) {
          for (let i = 0; i < params.length; i++) {
            url += i === 0 ? `?${params[i]}` : `&${params[i]}`
          }
        }

        const res = await this.$axios.get(url, this.$defaultHeaders())
        this.results = res.data.hits

        const pageAmount = Math.ceil(res.data.total_hits / res.data.limit)

        this.currentPage = newPageNumber
        if (pageAmount > 4) {
          if (this.currentPage + 3 >= pageAmount) {
            this.pages = [
              1,
              '-',
              pageAmount - 4,
              pageAmount - 3,
              pageAmount - 2,
              pageAmount - 1,
              pageAmount,
            ]
          } else if (this.currentPage > 4) {
            this.pages = [
              1,
              '-',
              this.currentPage - 1,
              this.currentPage,
              this.currentPage + 1,
              '-',
              pageAmount,
            ]
          } else {
            this.pages = [1, 2, 3, 4, 5, '-', pageAmount]
          }
        } else {
          this.pages = Array.from({ length: pageAmount }, (_, i) => i + 1)
        }

        if (process.client) {
          const queryItems = []

          if (this.query) queryItems.push(`q=${encodeURIComponent(this.query)}`)
          if (offset > 0) queryItems.push(`o=${offset}`)
          if (this.facets.length > 0)
            queryItems.push(`f=${encodeURIComponent(this.facets)}`)
          if (this.orFacets.length > 0)
            queryItems.push(`g=${encodeURIComponent(this.orFacets)}`)
          if (this.selectedVersions.length > 0)
            queryItems.push(`v=${encodeURIComponent(this.selectedVersions)}`)
          if (this.selectedLicenses.length > 0)
            queryItems.push(`l=${encodeURIComponent(this.selectedLicenses)}`)
          if (this.showSnapshots) url += `h=true`
          if (this.selectedEnvironments.length > 0)
            queryItems.push(
              `e=${encodeURIComponent(this.selectedEnvironments)}`
            )
          if (this.sortType.name !== 'relevance')
            queryItems.push(`s=${encodeURIComponent(this.sortType.name)}`)
          if (this.maxResults !== 20)
            queryItems.push(`m=${encodeURIComponent(this.maxResults)}`)

          url = `${this.$route.path}`

          if (queryItems.length > 0) {
            url += `?${queryItems[0]}`

            for (let i = 1; i < queryItems.length; i++) {
              url += `&${queryItems[i]}`
            }
          }

          await this.$router.replace({ path: url })
        }
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(err)
      }
    },
  },
}
</script>

<style lang="scss" scoped>
.filters-card {
  padding: var(--spacing-card-lg);
}

.sidebar-menu {
  display: none;
  margin-top: 1rem;
}

.sidebar-menu_open {
  display: block;
}

.sidebar-menu-heading {
  margin: 1.5rem 0 0.5rem 0;
}

.search-controls {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-card-md);
  flex-wrap: wrap;

  .sort-controls {
    width: 100%;
    display: flex;
    flex-direction: row;
    gap: var(--spacing-card-md);
    flex-wrap: wrap;

    .labeled-control {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      flex-wrap: wrap;
      gap: 0.5rem;

      .labeled-control__label {
        white-space: nowrap;
      }
    }
  }

  .iconified-input {
    flex: 1;

    input {
      min-width: 15rem;
      width: 100%;
    }
  }
}

.search-controls__sorting {
  min-width: 14rem;
}

.labeled-control__label,
.labeled-control__control {
  display: block;
}

.no-results {
  text-align: center;
}

@media screen and (min-width: 700px) {
  .search-controls {
    flex-wrap: nowrap;
    flex-direction: row;
  }

  .sort-controls {
    min-width: fit-content;
    max-width: fit-content;
    flex-wrap: nowrap;
  }

  .labeled-control {
    align-items: center;
    display: flex;
    flex-direction: column !important;
    flex-wrap: wrap;
    gap: 0.5rem;
    max-width: fit-content;
  }

  .labeled-control__label {
    flex-shrink: 0;
    margin-bottom: 0 !important;
  }
}

@media screen and (min-width: 860px) {
  .labeled-control {
    flex-wrap: nowrap !important;
    flex-direction: row !important;
  }
}

@media screen and (min-width: 1024px) {
  .sidebar-menu {
    display: block;
    margin-top: 0;
  }

  .sidebar-menu-close-button {
    display: none;
  }

  .labeled-control {
    flex-wrap: wrap !important;
    flex-direction: column !important;
  }
}

@media screen and (min-width: 1100px) {
  .labeled-control {
    flex-wrap: nowrap !important;
    flex-direction: row !important;
  }
}
</style>
