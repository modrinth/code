<template>
  <article class="project-card card" :aria-label="name" role="listitem">
    <div class="columns">
      <div class="icon">
        <nuxt-link :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`">
          <Avatar :src="iconUrl" :alt="name" size="md" />
        </nuxt-link>
      </div>
      <div class="card-content">
        <div class="info">
          <div class="top">
            <h2 class="title">
              <nuxt-link
                :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`"
                >{{ name }}</nuxt-link
              >
            </h2>
            <p v-if="author" class="author">
              by
              <nuxt-link class="title-link" :to="'/user/' + author"
                >{{ author }}
              </nuxt-link>
            </p>
          </div>
          <div
            v-if="
              type !== 'resourcepack' &&
              !(projectTypeDisplay === 'plugin' && search)
            "
            class="side-type"
          >
            <div
              v-if="clientSide === 'optional' && serverSide === 'optional'"
              class="side-descriptor"
            >
              <InfoIcon aria-hidden="true" />
              Universal {{ projectTypeDisplay }}
            </div>
            <div
              v-else-if="
                (clientSide === 'optional' || clientSide === 'required') &&
                (serverSide === 'optional' || serverSide === 'unsupported')
              "
              class="side-descriptor"
            >
              <InfoIcon aria-hidden="true" />
              Client {{ projectTypeDisplay }}
            </div>
            <div
              v-else-if="
                (serverSide === 'optional' || serverSide === 'required') &&
                (clientSide === 'optional' || clientSide === 'unsupported')
              "
              class="side-descriptor"
            >
              <InfoIcon aria-hidden="true" />
              Server {{ projectTypeDisplay }}
            </div>
            <div v-else-if="moderation" class="side-descriptor">
              <InfoIcon aria-hidden="true" />
              A {{ projectTypeDisplay }}
            </div>
          </div>
          <div v-else-if="moderation" class="side-descriptor">
            <InfoIcon aria-hidden="true" />
            A {{ projectTypeDisplay }}
          </div>
          <p class="description">
            {{ description }}
          </p>
          <Categories
            :categories="categories"
            :type="type"
            class="right-categories"
          />
          <div class="dates">
            <div
              v-tooltip="
                $dayjs(createdAt).format('MMMM D, YYYY [at] h:mm:ss A')
              "
              class="date"
            >
              <CalendarIcon aria-hidden="true" />
              Created {{ $dayjs(createdAt).fromNow() }}
            </div>
            <div
              v-tooltip="
                $dayjs(updatedAt).format('MMMM D, YYYY [at] h:mm:ss A')
              "
              class="date"
            >
              <EditIcon aria-hidden="true" />
              Updated {{ $dayjs(updatedAt).fromNow() }}
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="right-side">
      <div v-if="downloads" class="stat">
        <DownloadIcon aria-hidden="true" />
        <p>
          <strong>{{ $formatNumber(downloads) }}</strong> download<span
            v-if="downloads !== '1'"
            >s</span
          >
        </p>
      </div>
      <div v-if="follows" class="stat">
        <HeartIcon aria-hidden="true" />
        <p>
          <strong>{{ $formatNumber(follows) }}</strong> follower<span
            v-if="follows !== '1'"
            >s</span
          >
        </p>
      </div>
      <div class="mobile-dates">
        <div class="date">
          <CalendarIcon aria-hidden="true" />
          Created {{ $dayjs(createdAt).fromNow() }}
        </div>
        <div class="date">
          <EditIcon aria-hidden="true" />
          Updated {{ $dayjs(updatedAt).fromNow() }}
        </div>
      </div>
      <div v-if="status" class="status">
        <Badge
          v-if="status === 'approved'"
          color="green custom-circle"
          :type="status"
        />
        <Badge
          v-else-if="status === 'processing' || status === 'archived'"
          color="yellow custom-circle"
          :type="status"
        />
        <Badge
          v-else-if="status === 'rejected'"
          color="red custom-circle"
          :type="status"
        />
        <Badge v-else color="gray custom-circle" :type="status" />
      </div>
      <div class="buttons">
        <slot />
      </div>
    </div>
  </article>
</template>

<script>
import Categories from '~/components/ui/search/Categories'
import Badge from '~/components/ui/Badge'

import InfoIcon from '~/assets/images/utils/info.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import EditIcon from '~/assets/images/utils/updated.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import HeartIcon from '~/assets/images/utils/heart.svg?inline'
import Avatar from '~/components/ui/Avatar'

export default {
  name: 'ProjectCard',
  components: {
    Avatar,
    Categories,
    Badge,
    InfoIcon,
    CalendarIcon,
    EditIcon,
    DownloadIcon,
    HeartIcon,
  },
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
  },
  computed: {
    projectTypeDisplay() {
      return this.$getProjectTypeForDisplay(this.type, this.categories)
    },
  },
}
</script>

<style lang="scss" scoped>
.columns {
  width: 100%;
}

.project-card {
  display: flex;
  flex-direction: row;
  padding: var(--spacing-card-bg);
  width: calc(100% - 2 * var(--spacing-card-bg));
  overflow: hidden;

  @media screen and (min-width: 1024px) {
    flex-direction: row;
    justify-content: space-between;
  }

  .icon {
    margin: 0 var(--spacing-card-md) var(--spacing-card-md) 0;
  }

  .card-content {
    display: flex;
    justify-content: space-between;
    flex-grow: 1;
    overflow: hidden;

    .info {
      display: flex;
      flex-direction: column;

      .top {
        align-items: baseline;
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        flex-shrink: 0;
        margin-right: var(--spacing-card-md);

        .title {
          margin: 0 0.5rem 0 0;
          overflow-wrap: anywhere;
          color: var(--color-text-dark);
          font-size: var(--font-size-xl);
          word-wrap: break-word;
        }

        .author {
          margin: auto 0 0 0;
          color: var(--color-text);
          line-break: anywhere;
        }
      }

      .side-descriptor {
        display: flex;
        align-items: center;
        font-weight: bolder;
        font-size: var(--font-size-sm);

        margin: 0.125rem 0;

        svg {
          width: auto;
          height: 1rem;
          margin-right: 0.125rem;
        }
      }

      .description {
        margin: var(--spacing-card-sm) var(--spacing-card-md)
          var(--spacing-card-sm) 0;
      }

      .right-categories {
        margin-bottom: var(--spacing-card-sm);
      }

      .dates {
        display: flex;
        flex-wrap: wrap;

        .date {
          display: flex;
          align-items: center;
          margin-right: 2rem;
          cursor: default;

          svg {
            width: 1.25rem;
            height: 1.25rem;
            margin-right: 0.25rem;
          }
        }
      }
    }
  }

  .right-side {
    min-width: fit-content;

    .stat {
      display: flex;
      align-items: center;
      margin-bottom: 0.5rem;

      svg {
        width: auto;
        height: 1.25rem;

        margin-left: auto;
        margin-right: 0.25rem;
      }

      p {
        margin: 0;

        strong {
          font-weight: bolder;
          font-size: var(--font-size-lg);
        }
      }
    }

    .status {
      margin-bottom: 0.5rem;
    }

    .buttons {
      display: flex;
      flex-direction: column;

      button,
      a {
        margin-right: 0;
        margin-left: auto;
        margin-bottom: 0.5rem;

        &:last-child {
          margin-bottom: 0;
        }
      }
    }

    .mobile-dates {
      display: none;
    }
  }

  @media screen and (max-width: 800px) {
    flex-wrap: wrap;

    .card-content {
      flex-direction: column;

      .info {
        .top {
          flex-direction: column;
        }

        .dates {
          display: none;
        }
      }
    }

    .right-side {
      display: flex;
      gap: 0.5rem;
      flex-wrap: wrap;
      align-items: center;

      text-align: left;

      .stat {
        margin-bottom: 0;
      }

      .stat svg {
        margin-left: 0;
      }

      .buttons {
        flex: 1 1 100%;
      }

      .buttons button,
      a {
        margin-left: unset;
        margin-right: unset;
      }

      .status {
        margin-bottom: 0;
      }

      .mobile-dates {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem 0.5rem;
        color: var(--color-icon);
        font-size: var(--font-size-nm);

        .date {
          display: flex;
          align-items: center;
          cursor: default;

          svg {
            width: 1rem;
            height: 1rem;
            margin-right: 0.25rem;
          }
        }
      }
    }
  }
}
</style>
