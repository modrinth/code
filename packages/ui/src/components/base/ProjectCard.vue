<template>
  <article class="project-card base-card" :aria-label="name" role="listitem">
    <router-link class="icon" tabindex="-1" :to="`/${projectTypeUrl}/${id}`">
      <Avatar :src="iconUrl" :alt="name" size="md" no-shadow loading="lazy" />
    </router-link>
    <router-link
      class="gallery"
      :class="{ 'no-image': !featuredImage }"
      tabindex="-1"
      :to="`/${projectTypeUrl}/${id}`"
      :style="color ? `background-color: ${toColor};` : ''"
    >
      <img v-if="featuredImage" :src="featuredImage" alt="gallery image" loading="lazy" />
    </router-link>
    <div class="title">
      <router-link :to="`/${projectTypeUrl}/${id}`">
        <h2 class="name">
          {{ name }}
        </h2>
      </router-link>
      <p v-if="author" class="author">
        by
        <router-link class="title-link" :to="'/user/' + author">{{ author }} </router-link>
      </p>
      <Badge v-if="status && status !== 'approved'" :type="status" class="status" />
    </div>
    <p class="description">
      {{ description }}
    </p>
    <Categories :categories="categories" :type="type" class="tags">
      <EnvironmentIndicator
        :type-only="moderation"
        :client-side="clientSide"
        :server-side="serverSide"
        :type="projectTypeDisplay"
        :search="search"
        :categories="categories"
      />
    </Categories>
    <div class="stats">
      <div v-if="downloads" class="stat">
        <DownloadIcon aria-hidden="true" />
        <p>
          <strong>{{ formatNumber(downloads) }}</strong
          ><span class="stat-label"> download<span v-if="downloads !== '1'">s</span></span>
        </p>
      </div>
      <div v-if="follows" class="stat">
        <HeartIcon aria-hidden="true" />
        <p>
          <strong>{{ formatNumber(follows) }}</strong
          ><span class="stat-label"> follower<span v-if="follows !== '1'">s</span></span>
        </p>
      </div>
      <div class="buttons">
        <slot />
      </div>
      <div v-if="showUpdatedDate" v-tooltip="updatedDate" class="stat date">
        <EditIcon aria-hidden="true" />
        <span class="date-label">Updated </span> {{ sinceUpdated }}
      </div>
      <div v-else v-tooltip="createdDate" class="stat date">
        <CalendarIcon aria-hidden="true" />
        <span class="date-label">Published </span>{{ sinceCreation }}
      </div>
    </div>
  </article>
</template>

<script setup>
import { HeartIcon, DownloadIcon, EditIcon, CalendarIcon } from '@modrinth/assets'
import { formatNumber } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime.js'
import { defineComponent } from 'vue'
import Categories from '../search/Categories.vue'
import Badge from './SimpleBadge.vue'
import Avatar from './Avatar.vue'
import EnvironmentIndicator from './EnvironmentIndicator.vue'
</script>

<script>
import { useRelativeTime } from '../../composables'

dayjs.extend(relativeTime)
export default defineComponent({
  props: {
    id: {
      type: String,
      default: 'modrinth-0',
    },
    type: {
      type: String,
      default: 'mod',
    },
    name: {
      type: String,
      default: 'Project Name',
    },
    author: {
      type: String,
      default: null,
    },
    description: {
      type: String,
      default: 'A _type description',
    },
    iconUrl: {
      type: String,
      default: '#',
      required: false,
    },
    downloads: {
      type: String,
      default: null,
      required: false,
    },
    follows: {
      type: String,
      default: null,
      required: false,
    },
    createdAt: {
      type: String,
      default: '0000-00-00',
    },
    updatedAt: {
      type: String,
      default: null,
    },
    categories: {
      type: Array,
      default() {
        return []
      },
    },
    filteredCategories: {
      type: Array,
      default() {
        return []
      },
    },
    projectTypeDisplay: {
      type: String,
      default: null,
    },
    projectTypeUrl: {
      type: String,
      default: null,
    },
    status: {
      type: String,
      default: null,
    },
    serverSide: {
      type: String,
      required: false,
      default: '',
    },
    clientSide: {
      type: String,
      required: false,
      default: '',
    },
    moderation: {
      type: Boolean,
      required: false,
      default: false,
    },
    search: {
      type: Boolean,
      required: false,
      default: false,
    },
    featuredImage: {
      type: String,
      required: false,
      default: null,
    },
    showUpdatedDate: {
      type: Boolean,
      required: false,
      default: true,
    },
    hideLoaders: {
      type: Boolean,
      required: false,
      default: false,
    },
    color: {
      type: Number,
      required: false,
      default: null,
    },
  },
  setup(_) {
    const formatRelativeTime = useRelativeTime()
    return { formatRelativeTime }
  },
  computed: {
    toColor() {
      let color = this.color

      color >>>= 0
      const b = color & 0xff
      const g = (color & 0xff00) >>> 8
      const r = (color & 0xff0000) >>> 16
      return `rgba(${[r, g, b, 1].join(',')})`
    },
    createdDate() {
      return dayjs(this.createdAt).format('MMMM D, YYYY [at] h:mm:ss A')
    },
    sinceCreation() {
      return this.formatRelativeTime(this.createdAt)
    },
    updatedDate() {
      return dayjs(this.updatedAt).format('MMMM D, YYYY [at] h:mm:ss A')
    },
    sinceUpdated() {
      return this.formatRelativeTime(this.updatedAt)
    },
  },
  methods: {
    formatNumber,
  },
})
</script>

<style lang="scss" scoped>
.project-card {
  display: inline-grid;
  box-sizing: border-box;
  overflow: hidden;
  margin: 0;
  line-height: 1;
}

.display-mode--list .project-card {
  grid-template:
    'icon title stats'
    'icon description stats'
    'icon tags stats';
  grid-template-columns: min-content 1fr auto;
  grid-template-rows: min-content 1fr min-content;
  column-gap: var(--gap-md);
  row-gap: var(--gap-sm);
  width: 100%;

  @media screen and (max-width: 750px) {
    grid-template:
      'icon title'
      'icon description'
      'icon tags'
      'stats stats';
    grid-template-columns: min-content auto;
    grid-template-rows: min-content 1fr min-content min-content;
  }

  @media screen and (max-width: 550px) {
    grid-template:
      'icon title'
      'icon description'
      'tags tags'
      'stats stats';
    grid-template-columns: min-content auto;
    grid-template-rows: min-content 1fr min-content min-content;
  }

  h2 {
    margin: 0;
    font-size: 1.5rem;
  }
}

.display-mode--gallery .project-card,
.display-mode--grid .project-card {
  padding: 0 0 1rem 0;
  grid-template: 'gallery gallery' 'icon title' 'description  description' 'tags tags' 'stats stats';
  grid-template-columns: min-content 1fr;
  grid-template-rows: min-content min-content 1fr min-content min-content;
  row-gap: var(--gap-sm);

  .gallery {
    display: inline-block;
    width: 100%;
    height: 10rem;
    background-color: var(--color-button-bg);

    &.no-image {
      filter: brightness(0.7);
    }

    img {
      box-shadow: none;
      width: 100%;
      height: 10rem;
      object-fit: cover;
    }
  }

  .icon {
    margin-left: var(--gap-lg);
    margin-top: -3rem;
    z-index: 1;

    img,
    svg {
      border-radius: var(--radius-lg);
      box-shadow:
        -2px -2px 0 2px var(--color-raised-bg),
        2px -2px 0 2px var(--color-raised-bg);
    }
  }

  .title {
    margin-left: var(--gap-md);
    margin-right: var(--gap-md);
    flex-direction: column;

    .name {
      font-size: 1.25rem;
    }

    .status {
      margin-top: var(--gap-xs);
    }
  }

  .description {
    margin-inline: var(--gap-lg);
  }

  .tags {
    margin-inline: var(--gap-lg);
  }

  .stats {
    margin-inline: var(--gap-lg);
    flex-direction: row;
    align-items: center;

    .stat-label {
      display: none;
    }

    .buttons {
      flex-direction: row;
      gap: var(--gap-sm);
      align-items: center;

      > :first-child {
        margin-left: auto;
      }

      &:first-child > :last-child {
        margin-right: auto;
      }
    }

    .buttons:not(:empty) + .date {
      flex-basis: 100%;
    }
  }
}

.display-mode--grid .project-card {
  .gallery {
    display: none;
  }

  .icon {
    margin-top: calc(var(--gap-lg) - var(--gap-sm));

    img,
    svg {
      border: none;
    }
  }

  .title {
    margin-top: calc(var(--gap-lg) - var(--gap-sm));
  }
}

.icon {
  grid-area: icon;
  display: flex;
  align-items: center;
}

.gallery {
  display: none;
  height: 10rem;
  grid-area: gallery;
}

.title {
  grid-area: title;
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: baseline;
  column-gap: var(--gap-sm);
  row-gap: 0;
  word-wrap: anywhere;

  h2 {
    font-weight: bolder;
    color: var(--color-contrast);
  }

  h2,
  p {
    margin: 0;
    overflow-wrap: anywhere;
  }

  svg {
    width: auto;
    color: var(--color-special-orange);
    height: 1.5rem;
    margin-bottom: -0.25rem;
  }

  .title-link {
    text-decoration: underline;

    &:focus-visible,
    &:hover {
      color: var(--color-heading);
    }

    &:active {
      color: var(--color-text-dark);
    }
  }
}

.stats {
  grid-area: stats;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: var(--gap-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--gap-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      height: var(--stat-strong-size);
      width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }

  @media screen and (max-width: 750px) {
    flex-direction: row;
    column-gap: var(--gap-md);
    margin-top: var(--gap-xs);
  }

  @media screen and (max-width: 600px) {
    margin-top: 0;

    .stat-label {
      display: none;
    }
  }
}

.environment {
  color: var(--color-text) !important;
  font-weight: bold;
}

.description {
  grid-area: description;
  margin-block: 0;
  display: flex;
  justify-content: flex-start;
}

.tags {
  grid-area: tags;
  display: flex;
  flex-direction: row;

  @media screen and (max-width: 550px) {
    margin-top: var(--gap-xs);
  }
}

.buttons {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  align-items: flex-end;
  flex-grow: 1;
}

.small-mode {
  @media screen and (min-width: 750px) {
    grid-template:
      'icon title'
      'icon description'
      'icon tags'
      'stats stats' !important;
    grid-template-columns: min-content auto !important;
    grid-template-rows: min-content 1fr min-content min-content !important;

    .tags {
      margin-top: var(--gap-xs) !important;
    }

    .stats {
      flex-direction: row;
      column-gap: var(--gap-md) !important;
      margin-top: var(--gap-xs) !important;

      .stat-label {
        display: none !important;
      }
    }
  }
}

.base-card {
  padding: var(--gap-lg);

  position: relative;
  min-height: 2rem;

  background-color: var(--color-raised-bg);
  border-radius: var(--radius-lg);

  outline: 2px solid transparent;

  box-shadow: var(--shadow-card);

  .card__overlay {
    position: absolute;
    top: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    grid-gap: 0.5rem;
    z-index: 2;
  }

  &.warning {
    border-left: 0.5rem solid var(--color-banner-side);
    padding: 1.5rem;
    line-height: 1.5;
    background-color: var(--color-banner-bg);
    color: var(--color-banner-text);
    min-height: 0;

    a {
      /* Uses active color to increase contrast */
      color: var(--color-blue);
      text-decoration: underline;
    }
  }

  &.moderation-card {
    background-color: var(--color-banner-bg);
  }
}
</style>
