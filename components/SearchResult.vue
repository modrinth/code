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
      </div>
    </div>
    <div class="categories">
      <div class="loaders rows">
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
        <svg v-if="categories.includes('fabric')" viewBox="0 0 16 16">
          <g fill="#38342a">
            <rect width="1" height="1" x="9" y="1" />
            <rect width="1" height="1" x="8" y="2" />
            <rect width="1" height="1" x="10" y="2" />
            <rect width="1" height="1" x="8" y="3" />
            <rect width="1" height="1" x="11" y="3" />
            <rect width="1" height="1" x="7" y="4" />
            <rect width="1" height="1" x="9" y="4" />
            <rect width="1" height="1" x="12" y="4" />
            <rect width="1" height="1" x="6" y="5" />
            <rect width="1" height="1" x="10" y="5" />
            <rect width="1" height="1" x="13" y="5" />
            <rect width="1" height="1" x="5" y="6" />
            <rect width="1" height="1" x="11" y="6" />
            <rect width="1" height="1" x="13" y="6" />
            <rect width="1" height="1" x="14" y="6" />
            <rect width="1" height="1" x="4" y="7" />
            <rect width="1" height="1" x="12" y="7" />
            <rect width="1" height="1" x="14" y="7" />
            <rect width="1" height="1" x="3" y="8" />
            <rect width="1" height="1" x="12" y="8" />
            <rect width="1" height="1" x="13" y="8" />
            <rect width="1" height="1" x="2" y="9" />
            <rect width="1" height="1" x="11" y="9" />
            <rect width="1" height="1" x="2" y="10" />
            <rect width="1" height="1" x="10" y="10" />
            <rect width="1" height="1" x="3" y="11" />
            <rect width="1" height="1" x="9" y="11" />
            <rect width="1" height="1" x="4" y="12" />
            <rect width="1" height="1" x="8" y="12" />
            <rect width="1" height="1" x="5" y="13" />
            <rect width="1" height="1" x="7" y="13" />
            <rect width="1" height="1" x="8" y="13" />
            <rect width="1" height="1" x="6" y="14" />
            <rect width="1" height="1" x="7" y="14" />
          </g>
          <rect fill="#807a6d" width="1" height="1" x="13" y="7" />
          <g fill="#9a927e">
            <rect width="1" height="1" x="3" y="10" />
            <rect width="1" height="1" x="4" y="11" />
            <rect width="1" height="1" x="5" y="12" />
            <rect width="1" height="1" x="6" y="13" />
          </g>
          <g fill="#aea694">
            <rect width="1" height="1" x="3" y="9" />
            <rect width="1" height="1" x="4" y="10" />
            <rect width="1" height="1" x="5" y="11" />
            <rect width="1" height="1" x="6" y="12" />
          </g>
          <g fill="#bcb29c">
            <rect width="1" height="1" x="10" y="4" />
            <rect width="1" height="1" x="11" y="5" />
            <rect width="1" height="1" x="12" y="6" />
            <rect width="1" height="1" x="11" y="7" />
            <rect width="1" height="1" x="10" y="8" />
            <rect width="1" height="1" x="11" y="8" />
            <rect width="1" height="1" x="10" y="9" />
            <rect width="1" height="1" x="8" y="10" />
            <rect width="1" height="1" x="9" y="10" />
            <rect width="1" height="1" x="8" y="11" />
            <rect width="1" height="1" x="7" y="12" />
          </g>
          <g fill="#c6bca5">
            <rect width="1" height="1" x="9" y="3" />
            <rect width="1" height="1" x="7" y="5" />
            <rect width="1" height="1" x="8" y="6" />
            <rect width="1" height="1" x="9" y="7" />
            <rect width="1" height="1" x="10" y="7" />
            <rect width="1" height="1" x="6" y="8" />
            <rect width="1" height="1" x="7" y="9" />
            <rect width="1" height="1" x="8" y="9" />
          </g>
          <g fill="#dbd0b4">
            <rect width="1" height="1" x="9" y="2" />
            <rect width="1" height="1" x="10" y="3" />
            <rect width="1" height="1" x="8" y="4" />
            <rect width="1" height="1" x="11" y="4" />
            <rect width="1" height="1" x="8" y="5" />
            <rect width="1" height="1" x="9" y="5" />
            <rect width="1" height="1" x="12" y="5" />
            <rect width="1" height="1" x="6" y="6" />
            <rect width="1" height="1" x="7" y="6" />
            <rect width="1" height="1" x="9" y="6" />
            <rect width="1" height="1" x="10" y="6" />
            <rect width="1" height="1" x="5" y="7" />
            <rect width="1" height="1" x="6" y="7" />
            <rect width="1" height="1" x="7" y="7" />
            <rect width="1" height="1" x="8" y="7" />
            <rect width="1" height="1" x="4" y="8" />
            <rect width="1" height="1" x="5" y="8" />
            <rect width="1" height="1" x="7" y="8" />
            <rect width="1" height="1" x="8" y="8" />
            <rect width="1" height="1" x="9" y="8" />
            <rect width="1" height="1" x="4" y="9" />
            <rect width="1" height="1" x="5" y="9" />
            <rect width="1" height="1" x="6" y="9" />
            <rect width="1" height="1" x="9" y="9" />
            <rect width="1" height="1" x="5" y="10" />
            <rect width="1" height="1" x="6" y="10" />
            <rect width="1" height="1" x="7" y="10" />
            <rect width="1" height="1" x="6" y="11" />
            <rect width="1" height="1" x="7" y="11" />
          </g>
        </svg>
      </div>
      <div class="category"></div>
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

<style>
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
  flex-direction: column;
  margin-left: auto;
  margin-right: 20px;
}

.loaders svg {
  height: 20px;
}
</style>
