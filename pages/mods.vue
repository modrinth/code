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
            @input="onSearchChange"
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
        <div class="pagination column-grow-1 columns paginates">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
          <p>1</p>
          <p>2</p>
          <p>3</p>
          <p>...</p>
          <p>10</p>
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
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
      <div>
        <!--<section class="filter-group">
          <h3>Categories</h3>
          <section>
            <svg
              enable-background="new 0 0 24 24"
              height="24"
              viewBox="0 0 24 24"
              width="24"
            >
              <g><rect fill="none" height="24" width="24" /></g>
              <g>
                <path
                  d="M22,12c0-1.1-0.9-2-2-2V7c0-1.1-0.9-2-2-2H6C4.9,5,4,5.9,4,7v3c-1.1,0-2,0.9-2,2v5h1.33L4,19h1l0.67-2h12.67L19,19h1 l0.67-2H22V12z M18,10h-5V7h5V10z M6,7h5v3H6V7z M4,12h16v3H4V12z"
                />
              </g>
            </svg>
            <span>Decoration</span>
          </section>
          <section>
            <span>A</span>
          </section>
          <section>
            <span>A</span>
          </section>
        </section>-->
      </div>
    </section>
  </div>
</template>

<script>
import axios from 'axios'
import SearchResult from '@/components/SearchResult'

export default {
  components: {
    SearchResult,
  },
  data() {
    return {
      query: '',
      results: [],
    }
  },
  async created() {
    const config = {
      headers: {
        Accept: 'application/json',
      },
    }

    try {
      const res = await axios.get('https://api.modrinth.com/api/v1/mod', config)

      this.results = res.data
    } catch (err) {
      console.error(err)
    }
  },
  methods: {
    async onSearchChange() {
      const config = {
        headers: {
          Accept: 'application/json',
        },
      }

      try {
        const res = await axios.get(
          `https://api.modrinth.com/api/v1/mod?query=${this.query}`,
          config
        )

        this.results = res.data
      } catch (err) {
        console.error(err)
      }
    },
    formatNumber(x) {
      return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    },
  },
}
</script>

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
      text-align: left !important;
      letter-spacing: 0.02rem;
      margin-bottom: 0.5rem;
      margin-top: 1.5rem;
      text-transform: uppercase;
    }
  }
}

.filter-group {
  text-align: center;
  margin-top: 2em;

  section {
    cursor: pointer;
    width: 100%;
    padding: 2px;
    border-left: 4px solid var(--color-grey-3);
    border-radius: 0 0.25rem 0.25rem 0;
    color: var(--color-grey-3);

    span {
      margin-left: 10px;
    }

    svg {
      height: 1rem;
      width: 1rem;
    }

    &:hover,
    &:focus {
      background-color: var(--color-grey-1);
      color: var(--color-text);
    }
  }
}
</style>
