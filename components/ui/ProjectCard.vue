<template>
  <article class="project-card">
    <div class="columns">
      <div class="icon">
        <nuxt-link v-if="isModrinth" :to="'/mod/' + id">
          <img
            :src="iconUrl || 'https://cdn.modrinth.com/placeholder.svg?inline'"
            :alt="name"
            loading="lazy"
          />
        </nuxt-link>
      </div>
      <div class="info">
        <div class="top">
          <h2 class="title">
            <nuxt-link v-if="isModrinth" :to="'/mod/' + id">{{
              name
            }}</nuxt-link>
            <a v-else :href="pageUrl">{{ name }}</a>
          </h2>
          <p v-if="author" class="author">
            by <nuxt-link :to="'/user/' + author">{{ author }}</nuxt-link>
          </p>
        </div>
        <p class="description">
          {{ description }}
        </p>
        <div :class="{ vertical: editMode }" class="bottom">
          <div class="stats">
            <div v-if="status !== null" class="stat">
              <div class="info">
                <h4>Status</h4>
                <span v-if="status === 'approved'" class="badge green">
                  Approved
                </span>
                <span v-if="status === 'rejected'" class="badge red">
                  Rejected
                </span>
                <span v-if="status === 'draft'" class="badge yellow"
                  >Draft</span
                >
                <span v-if="status === 'processing'" class="badge yellow">
                  Processing
                </span>
                <span v-if="status === 'unlisted'" class="badge gray">
                  Unlisted
                </span>
                <span v-if="status === 'unknown'" class="badge gray">
                  Unknown
                </span>
              </div>
            </div>
            <div class="stat">
              <DownloadIcon aria-hidden="true" />
              <div class="info">
                <h4>Downloads</h4>
                <p class="value">{{ formatNumber(downloads) }}</p>
              </div>
            </div>
            <div class="stat">
              <CalendarIcon aria-hidden="true" />
              <div class="info">
                <h4>Created</h4>
                <p
                  v-tooltip="
                    $dayjs(createdAt).format(
                      '[Created on] YYYY-MM-DD [at] HH:mm A'
                    )
                  "
                  class="value"
                >
                  {{ $dayjs(createdAt).fromNow() }}
                </p>
              </div>
            </div>
            <div class="stat">
              <EditIcon aria-hidden="true" />
              <div class="info">
                <h4>Updated</h4>
                <p
                  v-tooltip="
                    $dayjs(updatedAt).format(
                      '[Updated on] YYYY-MM-DD [at] HH:mm A'
                    )
                  "
                  class="value"
                >
                  {{ $dayjs(updatedAt).fromNow() }}
                </p>
              </div>
            </div>
            <div v-if="latestVersion" class="stat">
              <TagIcon aria-hidden="true" />
              <div class="info">
                <h4>Available For</h4>
                <p class="value">
                  {{ latestVersion }}
                </p>
              </div>
            </div>
          </div>
          <Categories :categories="categories" />
        </div>
      </div>
    </div>
    <div v-if="editMode" class="buttons">
      <slot />
    </div>
  </article>
</template>

<script>
import Categories from '~/components/ui/search/Categories'

import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import TagIcon from '~/assets/images/utils/tag.svg?inline'

export default {
  name: 'ProjectCard',
  components: {
    Categories,
    CalendarIcon,
    DownloadIcon,
    EditIcon,
    TagIcon,
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
      default: null,
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
      default: null,
    },
    latestVersion: {
      type: String,
      default: null,
    },
    categories: {
      type: Array,
      default() {
        return []
      },
    },
    editMode: {
      type: Boolean,
      default: false,
    },
    status: {
      type: String,
      default: null,
    },
    isModrinth: {
      type: Boolean,
      default: false,
    },
  },
  methods: {
    formatNumber(x) {
      return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    },
  },
}
</script>

<style lang="scss" scoped>
.project-card {
  @extend %row;
  @extend %card-spaced-b;
  width: 100%;
  flex-direction: column;

  @media screen and (min-width: 1024px) {
    flex-direction: row;
  }

  .icon {
    margin: auto 0;
    img {
      width: 6rem;
      height: 6rem;
      margin: var(--spacing-card-md);
      border-radius: var(--size-rounded-icon);
      object-fit: contain;
    }
  }
  .info {
    @extend %column;
    flex-grow: 1;
    .top {
      @extend %row;
      flex-wrap: wrap;
      flex-shrink: 0;
      margin-top: var(--spacing-card-md);
      margin-right: var(--spacing-card-md);
      .title {
        margin: 0;
        color: var(--color-text-dark);
        font-size: var(--font-size-lg);
      }
      .author {
        margin: auto 0 0 0.5rem;
        color: var(--color-text);
      }
    }
    .description {
      margin: var(--spacing-card-sm) var(--spacing-card-md) 0 0;
      height: 100%;
      color: var(--color-text-dark);
    }
    .bottom {
      @extend %column;
      flex-shrink: 0;
      margin-top: var(--spacing-card-sm);
      margin-right: var(--spacing-card-md);
      margin-bottom: var(--spacing-card-md);

      @media screen and (min-width: 1024px) {
        flex-direction: row;
        &.vertical {
          flex-direction: column;
          .categories {
            margin-top: var(--spacing-card-sm);
          }
        }
      }

      .stats {
        @extend %row;
        flex-wrap: wrap;

        @media screen and (min-width: 900px) {
          flex-wrap: nowrap;
        }

        .stat {
          @extend %stat;
        }
      }
      .categories {
        @media screen and (min-width: 1024px) {
          flex-direction: row;
          margin: auto 0;
        }
      }
    }
  }
  .buttons {
    @extend %column;
    margin-bottom: 1rem;

    @media screen and (min-width: 1024px) {
      margin-bottom: 0;
    }
  }
}
</style>
