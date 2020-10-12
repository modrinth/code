<template>
  <div class="result rows">
    <img class="result-icon" :src="iconUrl" />
    <div class="rows result-name-author">
      <h2 class="mod-name">
        <a :href="pageUrl">{{ name }}</a>
      </h2>
      <p class="author">
        by <a :href="authorUrl">{{ author }}</a>
      </p>
    </div>
    <p class="result-summary">
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
        <p>{{ $dayjs(createdAt).fromNow() }}</p>
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
        <p>{{ $dayjs(updatedAt).fromNow() }}</p>
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
    <div class="categories">
      <p v-if="categories.includes('technology')">
        <TechCategory />
        Technology
      </p>
      <p v-if="categories.includes('adventure')">
        <AdventureCategory />
        Adventure
      </p>
      <p v-if="categories.includes('magic')">
        <MagicCategory />
        Magic
      </p>
      <p v-if="categories.includes('utility')">
        <UtilityCategory />
        Utility
      </p>
      <p v-if="categories.includes('decoration')">
        <DecorationCategory />
        Decoration
      </p>
      <p v-if="categories.includes('library')">
        <LibraryCategory />
        Library
      </p>
      <p v-if="categories.includes('cursed')">
        <CursedCategory />
        Cursed
      </p>
      <p v-if="categories.includes('worldgen')">
        <WorldGenCategory />
        Worldgen
      </p>
      <p v-if="categories.includes('storage')">
        <StorageCategory />
        Storage
      </p>
      <p v-if="categories.includes('food')">
        <FoodCategory />
        Food
      </p>
      <p v-if="categories.includes('equipment')">
        <EquipmentCategory />
        Equipment
      </p>
      <p v-if="categories.includes('misc')">
        <MiscCategory />
        Misc
      </p>
    </div>
  </div>
</template>

<script>
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
  name: 'ModResult',
  components: {
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
    isAd: {
      type: Boolean,
      default: false,
    },
  },
}
</script>

<style lang="scss">
.results {
  margin-top: 10px;
}

.result {
  display: grid;
  grid-template-columns: 80px auto;
  grid-template-rows: auto auto auto 30px;
  max-width: 100vw;
  margin-bottom: 10px;
  background: var(--color-bg);
  box-shadow: 0 2px 3px 1px var(--color-grey-2);

  // Columns are larger to accomodate larger screens
  @media screen and (min-width: 1375px) {
    grid-template-columns: 120px auto;
  }
}

.result * {
  object-fit: contain;
  margin-bottom: 0;
  margin-top: 0;
}

.result-icon {
  width: 60px;
  height: 60px;
  margin: 5px !important;
  border-radius: 0.5rem;
  grid-row-start: 1;
  grid-row-end: 4;
  grid-column-start: 1;

  @media screen and (min-width: 900px) {
    margin: 10px 20px 10px 10px !important;
    width: 70px;
    height: 70px;
  }

  // Larger screen, larger icon
  @media screen and (min-width: 1375px) {
    width: 90px;
    height: 90px;
  }
}

.result-name-author {
  display: block;
  margin-top: 10px;
  min-height: 38px;
}

.result-summary {
  grid-row: 2;
  grid-column: 2;
  max-height: 150px;
  font-size: 11pt;
  margin: auto 0;
}

.mod-name {
  align-self: flex-end;
  font-size: 13pt;
}

.author {
  margin-bottom: 2px !important;
  align-self: flex-end;
  font-size: 12pt;
}

.result-infos {
  display: grid;
  grid-template-columns: 115px 115px auto;
  grid-template-rows: 20px 20px;
  margin-top: 5px;
  grid-column: 2;
  align-items: flex-start;
  align-self: flex-start;

  .columns:nth-child(2) {
    grid-column: 1;
  }

  .columns:nth-child(3) {
    grid-column: 2;
    grid-row: 1;
  }

  .result-image {
    p {
      font-size: 10pt;
    }

    svg {
      width: 14px;
      height: 14px;
    }
  }
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
  grid-column: 1;
  margin: 0 0 auto;
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
  grid-column: 1;
  grid-row: 3;
  img {
    width: 15px;
  }

  svg {
    width: 20px;
    margin-top: 2px;
    margin-left: 5px;
  }
}

// Larger tablet-sized screens
@media screen and (min-width: 900px) {
  .result {
    grid-template-columns: 90px auto;
    grid-template-rows: auto auto 35px;
  }

  .result-infos {
    display: flex;
    align-items: center;
    margin-top: 0;
    .result-image {
      p {
        font-size: 15px;
      }

      svg {
        width: 15px;
        height: 15px;
      }
    }
  }

  .loaders {
    svg {
      width: 20px;
    }
  }

  .categories {
    margin: 0 0 10px 0;
  }

  .result-name-author {
    display: flex;
    margin-top: 0;
    .author {
      margin-left: 5px;
    }
  }

  .mod-name {
    font-size: 18pt;
  }

  .result-summary {
    max-height: 100px;
    font-size: 13pt;
  }
}

// Larger screens
@media screen and (min-width: 1375px) {
  .result {
    margin: 5px 0 5px 15px;
    grid-column: 1;
    grid-template-columns: 110px auto;
  }

  .categories {
    margin: 0 5px 10px auto;
    grid-row: 3;
    grid-column: 3;
  }

  .mod-name {
    font-size: 20pt;
  }
}

// Desktop
@media screen and (min-width: 1500px) {
  .result-name-author {
    display: flex;
    min-height: 38px;
    grid-column: 2;
  }
}
</style>
