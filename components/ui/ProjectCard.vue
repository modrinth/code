<template>
  <article
    class="project-card base-card padding-bg"
    :aria-label="name"
    role="listitem"
  >
    <nuxt-link
      class="icon"
      tabindex="-1"
      :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`"
    >
      <Avatar :src="iconUrl" :alt="name" size="md" />
    </nuxt-link>
    <nuxt-link
      class="gallery"
      tabindex="-1"
      :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`"
    >
      <img
        v-if="galleryImages.length > 0"
        :src="galleryImages[0]"
        alt="Gallery image TODO: improve this lol"
      />
    </nuxt-link>
    <div class="title">
      <nuxt-link :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`">
        <h2 class="name">
          <IssuesIcon
            v-if="hasModMessage"
            v-tooltip="
              'Project has a message from the moderators. View the project to see more.'
            "
            aria-label="Project has a message from the moderators. View the project to see more."
          />
          {{ name }}
        </h2>
      </nuxt-link>
      <p v-if="author" class="author">
        by
        <nuxt-link class="title-link" :to="'/user/' + author"
          >{{ author }}
        </nuxt-link>
      </p>
      <Badge
        v-if="status && status !== 'approved'"
        :type="status"
        class="status"
      />
    </div>
    <p class="description">
      {{ description }}
    </p>
    <Categories
      :categories="
        categories.filter(
          (x) => !hideLoaders || !$tag.loaders.find((y) => y.name === x)
        )
      "
      :type="type"
      class="tags"
    >
      <span v-if="moderation" class="environment">
        <InfoIcon aria-hidden="true" />
        A {{ projectTypeDisplay }}
      </span>
      <span
        v-else-if="
          !['resourcepack', 'shader'].includes(type) &&
          !(projectTypeDisplay === 'plugin' && search) &&
          !categories.some((x) => $tag.loaderData.dataPackLoaders.includes(x))
        "
        class="environment"
      >
        <template v-if="clientSide === 'optional' && serverSide === 'optional'">
          <GlobeIcon aria-hidden="true" />
          Client or server
        </template>
        <template
          v-else-if="clientSide === 'required' && serverSide === 'required'"
        >
          <GlobeIcon aria-hidden="true" />
          Client and server
        </template>
        <template
          v-else-if="
            (clientSide === 'optional' || clientSide === 'required') &&
            (serverSide === 'optional' || serverSide === 'unsupported')
          "
        >
          <ClientIcon aria-hidden="true" />
          Client
        </template>
        <template
          v-else-if="
            (serverSide === 'optional' || serverSide === 'required') &&
            (clientSide === 'optional' || clientSide === 'unsupported')
          "
        >
          <ServerIcon aria-hidden="true" />
          Server
        </template>
        <template v-else-if="moderation">
          <InfoIcon aria-hidden="true" />
          A {{ projectTypeDisplay }}
        </template>
      </span>
    </Categories>
    <div class="stats">
      <div v-if="downloads" class="stat">
        <DownloadIcon aria-hidden="true" />
        <p>
          <strong>{{ $formatNumber(downloads) }}</strong
          ><span class="stat-label">
            download<span v-if="downloads !== '1'">s</span></span
          >
        </p>
      </div>
      <div v-if="follows" class="stat">
        <HeartIcon aria-hidden="true" />
        <p>
          <strong>{{ $formatNumber(follows) }}</strong
          ><span class="stat-label">
            follower<span v-if="follows !== '1'">s</span></span
          >
        </p>
      </div>
      <div class="buttons">
        <slot />
      </div>
      <div
        v-if="showUpdatedDate"
        v-tooltip="$dayjs(updatedAt).format('MMMM D, YYYY [at] h:mm:ss A')"
        class="stat date"
      >
        <EditIcon aria-hidden="true" />
        <span class="date-label">Updated </span
        >{{ $dayjs(updatedAt).fromNow() }}
      </div>
      <div
        v-else
        v-tooltip="$dayjs(createdAt).format('MMMM D, YYYY [at] h:mm:ss A')"
        class="stat date"
      >
        <CalendarIcon aria-hidden="true" />
        <span class="date-label">Published </span
        >{{ $dayjs(createdAt).fromNow() }}
      </div>
    </div>
  </article>
</template>

<script>
import Categories from '~/components/ui/search/Categories'
import Badge from '~/components/ui/Badge'

import InfoIcon from '~/assets/images/utils/info.svg?inline'
import ClientIcon from '~/assets/images/utils/client.svg?inline'
import GlobeIcon from '~/assets/images/utils/globe.svg?inline'
import ServerIcon from '~/assets/images/utils/server.svg?inline'
import IssuesIcon from '~/assets/images/utils/issues.svg?inline'
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
    ClientIcon,
    ServerIcon,
    GlobeIcon,
    IssuesIcon,
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
    hasModMessage: {
      type: Boolean,
      default: false,
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
    galleryImages: {
      type: Array,
      required: false,
      default() {
        return []
      },
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
  },
  computed: {
    projectTypeDisplay() {
      return this.$getProjectTypeForDisplay(this.type, this.categories)
    },
  },
}
</script>

<style lang="scss" scoped>
.project-card {
  display: inline-grid;
  box-sizing: border-box;
  overflow: hidden;
  margin: 0;
}

.display-mode--list .project-card {
  grid-template:
    'icon title stats'
    'icon description stats'
    'icon tags stats';
  grid-template-columns: min-content 1fr auto;
  grid-template-rows: min-content 1fr min-content;
  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-sm);
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
}

.display-mode--gallery .project-card,
.display-mode--grid .project-card {
  padding: 0 0 var(--spacing-card-bg) 0;
  grid-template: 'gallery gallery' 'icon title' 'description  description' 'tags tags' 'stats stats';
  grid-template-columns: min-content 1fr;
  grid-template-rows: min-content min-content 1fr min-content min-content;
  row-gap: var(--spacing-card-sm);

  .gallery {
    display: inline-block;
    width: 100%;
    height: 10rem;
    background-color: var(--color-button-bg-active);

    img {
      width: 100%;
      height: 10rem;
      object-fit: cover;
    }
  }

  .icon {
    margin-left: var(--spacing-card-bg);
    margin-top: -3rem;
    z-index: 1;
  }

  .title {
    margin-left: var(--spacing-card-md);
    margin-right: var(--spacing-card-bg);
    flex-direction: column;

    .name {
      font-size: 1.25rem;
    }

    .status {
      margin-top: var(--spacing-card-xs);
    }
  }

  .description {
    margin-inline: var(--spacing-card-bg);
  }

  .tags {
    margin-inline: var(--spacing-card-bg);
  }

  .stats {
    margin-inline: var(--spacing-card-bg);
    flex-direction: row;
    align-items: center;

    .stat-label {
      display: none;
    }

    .buttons {
      flex-direction: row;
      gap: var(--spacing-card-sm);
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
    margin-top: calc(var(--spacing-card-bg) - var(--spacing-card-sm));
  }

  .title {
    margin-top: calc(var(--spacing-card-bg) - var(--spacing-card-sm));
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
  column-gap: var(--spacing-card-sm);
  row-gap: 0;
  word-wrap: anywhere;

  h2,
  p {
    margin: 0;
  }

  svg {
    width: auto;
    color: var(--color-special-orange);
    height: 1.5rem;
    margin-bottom: -0.25rem;
  }
}

.stats {
  grid-area: stats;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: var(--spacing-card-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--spacing-card-xs);
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
    column-gap: var(--spacing-card-md);
    margin-top: var(--spacing-card-xs);
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
    margin-top: var(--spacing-card-xs);
  }
}

.buttons {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);
  align-items: flex-end;
  flex-grow: 1;
}
</style>
