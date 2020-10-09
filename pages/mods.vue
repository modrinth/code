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
          :created-at="result.date_created"
          :updated-at="result.date_modified"
          :downloads="formatNumber(result.downloads)"
          :icon-url="result.icon_url"
          :author-url="result.author_url"
          :page-url="result.page_url"
          :categories="result.categories"
          :is-ad="index === -1"
        />
        <div v-if="results.length === 0" class="no-results">
          <p>No results found for your query!</p>
        </div>
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
            <TechCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Adventure"
            facet-name="categories:adventure"
            @toggle="toggleFacet"
          >
            <AdventureCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Magic"
            facet-name="categories:magic"
            @toggle="toggleFacet"
          >
            <MagicCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Utility"
            facet-name="categories:utility"
            @toggle="toggleFacet"
          >
            <UtilityCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Decoration"
            facet-name="categories:decoration"
            @toggle="toggleFacet"
          >
            <DecorationCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Library"
            facet-name="categories:library"
            @toggle="toggleFacet"
          >
            <LibraryCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Cursed"
            facet-name="categories:cursed"
            @toggle="toggleFacet"
          >
            <CursedCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Worldgen"
            facet-name="categories:worldgen"
            @toggle="toggleFacet"
          >
            <WorldGenCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Storage"
            facet-name="categories:storage"
            @toggle="toggleFacet"
          >
            <StorageCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Food"
            facet-name="categories:food"
            @toggle="toggleFacet"
          >
            <FoodCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Equipment"
            facet-name="categories:equipment"
            @toggle="toggleFacet"
          >
            <EquipmentCategory />
          </SearchFilter>
          <SearchFilter
            :active-filters="facets"
            display-name="Misc"
            facet-name="categories:misc"
            @toggle="toggleFacet"
          >
            <MiscCategory />
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

import TechCategory from '~/assets/images/categories/tech.svg?inline'
import AdventureCategory from '~/assets/images/categories/adventure.svg?inline'
import CursedCategory from '~/assets/images/categories/cursed.svg?inline'
import DecorationCategory from '~/assets/images/categories/decoration.svg?inline'
import EquipmentCategory from '~/assets/images/categories/equipment.svg?inline'
import FoodCategory from '~/assets/images/categories/food.svg?inline'
import LibraryCategory from '~/assets/images/categories/library.svg?inline'
import MagicCategory from '~/assets/images/categories/magic.svg?inline'
import MiscCategory from '~/assets/images/categories/misc.svg?inline'
import StorageCategory from '~/assets/images/categories/storage.svg?inline'
import UtilityCategory from '~/assets/images/categories/utility.svg?inline'
import WorldGenCategory from '~/assets/images/categories/worldgen.svg?inline'

export default {
  auth: false,
  components: {
    SearchResult,
    Pagination,
    Multiselect,
    SearchFilter,
    TechCategory,
    AdventureCategory,
    CursedCategory,
    DecorationCategory,
    EquipmentCategory,
    FoodCategory,
    LibraryCategory,
    MagicCategory,
    MiscCategory,
    StorageCategory,
    UtilityCategory,
    WorldGenCategory,
  },
  async fetch() {
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
  methods: {
    async fillInitialVersions() {
      try {
        const res = await axios.get(
          'https://launchermeta.mojang.com/mc/game/version_manifest.json'
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

        const res = await axios.get(url)
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
    cursor: pointer;
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

.no-results {
  text-align: center;
  padding: 20px 0;
  font-size: 30px;
  color: var(--color-text);
  background-color: var(--color-grey-1);
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
  cursor: pointer;
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
