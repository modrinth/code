<template>
  <div class="columns">
    <div class="content column-grow-4">
      <h2>Mods</h2>
      <section id="search-pagination">
        <div class="iconified-input column-grow-2">
          <input
            id="search"
            v-model="query"
            type="search"
            name="search"
            placeholder="Search mods"
            @input="onSearchChange(true)"
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
        <div
          v-if="pages.length > 1"
          class="pagination column-grow-1 columns paginates"
        >
          <svg
            :class="{ 'disabled-paginate': currentPage === 1 }"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            @click="currentPage !== 1 ? switchPage(currentPage - 1) : null"
          >
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
          <p
            v-for="(item, index) in pages"
            :key="'page-' + item"
            :class="{
              'active-page-number': currentPage !== item,
            }"
            @click="currentPage !== item ? switchPage(item) : null"
          >
            <span v-if="pages[index - 1] + 1 !== item && item !== 1">...</span>
            <span :class="{ 'disabled-page-number': currentPage === item }">{{
              item
            }}</span>
          </p>

          <svg
            :class="{
              'disabled-paginate': currentPage === pages[pages.length - 1],
            }"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            @click="
              currentPage !== pages[pages.length - 1]
                ? switchPage(currentPage + 1)
                : null
            "
          >
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </div>
      </section>
      <div class="results column-grow-4">
        <SearchResult
          v-for="result in results"
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
        />
      </div>
    </div>
    <section class="filters">
      <!--#region filters  -->
      <div>
        <section class="filter-group">
          <h3>Categories</h3>
          <p id="technology" @click="toggleFilter('technology')">
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
            Technology
          </p>
          <p id="adventure" @click="toggleFilter('adventure')">
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
            Adventure
          </p>
          <p id="magic" @click="toggleFilter('magic')">
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
            Magic
          </p>
          <p id="utility" @click="toggleFilter('utility')">
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
            Utility
          </p>
          <p id="decoration" @click="toggleFilter('decoration')">
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
            Decoration
          </p>
          <p id="library" @click="toggleFilter('library')">
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
            Library
          </p>
          <p id="cursed" @click="toggleFilter('cursed')">
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
            Cursed
          </p>
          <p id="worldgen" @click="toggleFilter('worldgen')">
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
            Worldgen
          </p>
          <p id="storage" @click="toggleFilter('storage')">
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
            Storage
          </p>
          <p id="food" @click="toggleFilter('food')">
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
            Food
          </p>
          <p id="equipment" @click="toggleFilter('equipment')">
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
            Equipment
          </p>
          <p id="misc" @click="toggleFilter('misc')">
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
            Misc
          </p>
          <h3>Loaders</h3>
          <p id="forge" @click="toggleFilter('forge')">Forge</p>
          <p id="fabric" @click="toggleFilter('fabric')">Fabric</p>
          <h3>Versions</h3>
          <p>WIP</p>
        </section>
      </div>
      <!--#endregion -->
    </section>
  </div>
</template>

<script>
import axios from 'axios'
import SearchResult from '@/components/ModResult'

export default {
  components: {
    SearchResult,
  },
  data() {
    return {
      query: '',
      filters: [],
      results: [],
      pages: [],
      currentPage: 1,
    }
  },
  async created() {
    await this.onSearchChange()
  },
  methods: {
    async toggleFilter(elementName) {
      const element = document.getElementById(elementName)
      const index = this.filters.indexOf(element.id)

      if (index !== -1) {
        element.classList.remove('filter-active')
        this.filters.splice(index, 1)
      } else {
        element.classList.add('filter-active')
        this.filters.push(element.id)
      }

      await this.onSearchChange(true)
    },
    async switchPage(pageNumber) {
      this.currentPage = pageNumber

      await this.onSearchChange(false)
    },
    async onSearchChange(resetPageNumber) {
      if (resetPageNumber) this.currentPage = 1

      const config = {
        headers: {
          Accept: 'application/json',
        },
      }

      try {
        let url = 'https://api.modrinth.com/api/v1/mod'
        const params = []

        if (this.query.length > 0) {
          params.push(`query=${this.query.replace(/ /g, '+')}`)
        }

        if (this.filters.length > 0) {
          const facets = []

          for (const facet of this.filters) {
            facets.push(['categories:' + facet])
          }

          params.push(`facets=${JSON.stringify(facets)}`)
        }

        if (this.currentPage !== 1) {
          params.push(`offset=${(this.currentPage - 1) * 10}`)
        }

        if (params.length > 0) {
          for (let i = 0; i < params.length; i++) {
            url += i === 0 ? `?${params[i]}` : `&${params[i]}`
          }
        }

        const res = await axios.get(url, config)
        this.results = res.data.hits

        const pageAmount = Math.ceil(res.data.nb_hits / res.data.limit)

        if (pageAmount > 4) {
          if (this.currentPage > 4) {
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
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(err)
      }
    },
    formatNumber(x) {
      return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    },
  },
}
</script>

<style src="vue-multiselect/dist/vue-multiselect.min.css"></style>
<style lang="scss">
#search-pagination {
  align-items: center;
  display: flex;
  justify-content: space-between;
}

.paginates {
  align-items: center;
}

.paginates p {
  margin-left: 5px;
  margin-right: 5px;
}

.content {
  min-height: 95vh;
}

.filters {
  background-color: var(--color-bg);
  border-left: 1px solid var(--color-grey-2);
  top: 0;
  position: -webkit-sticky;
  position: sticky;
  max-height: 100vh;
  min-width: 15%;

  div {
    padding: 0 1.5rem;

    h3 {
      color: #718096;
      font-size: 0.8rem;
      letter-spacing: 0.02rem;
      margin-bottom: 0.5rem;
      margin-top: 1.5rem;
      text-transform: uppercase;
    }
  }
}

.filter-group {
  margin-top: 2em;

  p {
    display: flex;
    align-items: center;
    cursor: pointer;
    padding: 2px 2px 2px 20px;
    margin: 0 0 0 5px;
    border-left: 4px solid var(--color-grey-3);
    border-radius: 0 0.25rem 0.25rem 0;
    color: var(--color-grey-5);
    font-size: 1rem;
    letter-spacing: 0.02rem;

    svg {
      margin-right: 5px;
      height: 1rem;
    }

    &:hover,
    &:focus {
      background-color: var(--color-grey-1);
      color: var(--color-text);
    }
  }

  .filter-active {
    background-color: var(--color-grey-1);
    color: var(--color-text);
    border-left: 4px solid var(--color-brand);
  }
}

.disabled-paginate {
  cursor: default;
  color: var(--color-grey-3);
}

.active-page-number {
  cursor: pointer;
}

.disabled-page-number {
  cursor: default;
  padding: 2px 3px;
  border-radius: 3px;
  background-color: var(--color-grey-3);
}
</style>
