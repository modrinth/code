<template>
  <div class="result rows">
    <img class="result-icon" :src="iconUrl" />
    <div class="column-grow-3">
      <div class="rows column-grow-2">
        <h2 class="mod-name">
          <a :href="pageUrl">{{ name }}</a>
        </h2>
        <p class="author">
          by <a :href="authorUrl">{{ author }}</a>
        </p>
      </div>
      <p class="column-grow-1">
        {{ description }}
      </p>
      <div class="column-grow-1 columns result-infos">
        <div class="result-image columns">
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#3cdb36"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
          <p>{{ downloads }}</p>
        </div>
        <div class="result-image columns">
          <svg viewBox="0 0 16 16" fill="#099fef">
            <path
              fill-rule="evenodd"
              d="M10.854 7.146a.5.5 0 0 1 0 .708l-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 9.793l2.646-2.647a.5.5 0 0 1 .708 0z"
            />
            <path
              fill-rule="evenodd"
              d="M1 4v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V4H1zm1-3a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V3a2 2 0 0 0-2-2H2z"
            />
            <path
              fill-rule="evenodd"
              d="M3.5 0a.5.5 0 0 1 .5.5V1a.5.5 0 0 1-1 0V.5a.5.5 0 0 1 .5-.5zm9 0a.5.5 0 0 1 .5.5V1a.5.5 0 0 1-1 0V.5a.5.5 0 0 1 .5-.5z"
            />
          </svg>
          <p>{{ createdAt }}</p>
        </div>
        <div class="result-image columns">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="#e88d0d"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
            ></path>
            <path
              d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
            ></path>
          </svg>
          <p>{{ updatedAt }}</p>
        </div>
        <div class="result-image columns">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="#e8200d"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
            ></path>
            <line x1="7" y1="7" x2="7.01" y2="7"></line>
          </svg>
          <p>{{ latestVersion }}</p>
        </div>

        <div class="loaders columns">
          <img
            v-if="categories.includes('fabric')"
            src="https://fabricmc.net/assets/logo.png"
            alt="fabric"
          />

          <svg v-if="categories.includes('forge')" viewBox="0 0 90 46">
            <defs>
              <path
                id="a"
                fill="#1e2d44"
                d="M85.8 49.1l-4.5-5.5q-1.65-.1-3.55-.55-3.8-.95-4.85-2.65Q68.75 34.25 74 27q5.45-7.65 17.55-10.3l-37.8-1.9H100v-3.7H47.75v14q0 .65-1.9-11.7h-4.1v13l-1.9-12.3h-27.9q12.85 10.8 19.9 14.3 2.45 1.2 6.05 1.65 2.1.25 6.35.35 2.1.1 3.1.35 1.65.4 2.7 1.45 1.75 1.7 2 4 .3 2.4-1.2 4.3-1.2 1.65-4.35 2.6l-2.95.6L39 49.1v6.4h10.25l.3-6.3 8.95-6.3q-2.85 2.4-6.25 7.7-.95 1.5-1.7 3.5 1.7-1.45 4.9-2.3 3.3-.9 7.3-.9 3.9 0 7.2.9 3.25.85 4.95 2.3-.6-1.75-1.7-3.5-3.4-5.2-6.2-7.7l8.9 6.3.3 6.3h9.6v-6.4z"
              />
            </defs>
            <use transform="translate(-10.95 -10.3)" xlink:href="#a" />
          </svg>
        </div>
      </div>
    </div>
    <div class="categories">
      <p v-if="categories.includes('technology')">
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
      <p v-if="categories.includes('adventure')">
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
      <p v-if="categories.includes('magic')">
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
      <p v-if="categories.includes('utility')">
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
      <p v-if="categories.includes('decoration')">
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
      <p v-if="categories.includes('library')">
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
      <p v-if="categories.includes('cursed')">
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
      <p v-if="categories.includes('worldgen')">
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
      <p v-if="categories.includes('storage')">
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
      <p v-if="categories.includes('food')">
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
      <p v-if="categories.includes('equipment')">
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
      <p v-if="categories.includes('misc')">
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
    </div>
  </div>
</template>

<script>
export default {
  name: 'SearchResult',
  props: {
    id: {
      type: String,
      default: 'modrinth-0',
    },
    name: {
      type: String,
      default: 'Mod Name',
    },
    author: {
      type: String,
      default: 'Author',
    },
    description: {
      type: String,
      default: 'A mod description',
    },
    pageUrl: {
      type: String,
      default: '#',
    },
    authorUrl: {
      type: String,
      default: '#',
    },
    iconUrl: {
      type: String,
      default: '#',
    },
    downloads: {
      type: String,
      default: '0',
    },
    createdAt: {
      type: String,
      default: '0000-00-00',
    },
    updatedAt: {
      type: String,
      default: '0000-00-00',
    },
    latestVersion: {
      type: String,
      default: '1.16.2',
    },
    categories: {
      type: Array,
      default() {
        return []
      },
    },
  },
}
</script>

<style lang="scss">
.results {
  margin-top: 20px;
  margin-bottom: 50px;
}

.result {
  align-items: center;
  margin-bottom: 10px;
  background: var(--color-bg);
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
}

.result * {
  object-fit: contain;
  margin-bottom: 0;
  margin-top: 0;
}

.result-icon {
  width: 90px;
  height: 90px;
  margin: 10px 20px 10px 10px !important;
  border-radius: 0.5rem;
}

.mod-name {
  align-self: flex-end;
}

.author {
  margin-left: 5px;
  margin-bottom: 2px !important;
  align-self: flex-end;
}

.result-infos {
  margin-top: 5px !important;
  align-self: flex-end;
  align-items: center;
}

.result-image svg {
  width: 15px;
  height: 15px;
  align-self: center;
}

.result-image p {
  margin-left: 5px;
  margin-right: 15px;
  font-size: 15px;
  align-self: center;
}

.categories {
  display: flex;
  flex-direction: row;
  margin: auto 20px 20px auto;
}

.categories p {
  display: flex;
  align-items: center;
  flex-direction: row;
  background-color: var(--color-grey-1);
  color: var(--color-text);
  margin: 0 5px;
  padding: 2px;
  font-size: 15px;

  svg {
    width: 15px;
    margin-right: 5px;
  }
}

.loaders {
  align-items: center;
  img {
    width: 15px;
  }

  svg {
    width: 20px;
    margin-top: 2px;
    margin-left: 5px;
  }
}
</style>
