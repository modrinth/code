<template>
  <div class="content">
    <div v-if="currentMember" class="card header-buttons">
      <nuxt-link to="version/create" class="brand-button iconified-button">
        <PlusIcon />
        Create a version
      </nuxt-link>
    </div>
    <VersionFilterControl
      class="card"
      :versions="versions"
      @updateVersions="updateVersions"
    />
    <div v-if="versions.length > 0" class="universal-card all-versions">
      <div class="header">
        <div></div>
        <div>Version</div>
        <div>Supports</div>
        <div>Stats</div>
      </div>
      <div
        v-for="version in filteredVersions"
        :key="version.id + '-new'"
        class="version-button button-transparent"
        @click="
          $router.push(
            `/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/version/${encodeURI(version.displayUrlEnding)}`
          )
        "
      >
        <a
          v-tooltip="
            $parent.findPrimary(version).filename +
            ' (' +
            $formatBytes($parent.findPrimary(version).size) +
            ')'
          "
          :href="$parent.findPrimary(version).url"
          class="download-button"
          :class="version.version_type"
          :title="`Download ${version.name}`"
          @click.stop="(event) => event.stopPropagation()"
        >
          <DownloadIcon aria-hidden="true" />
        </a>
        <span class="version__title">{{ version.name }}</span>
        <div class="version__metadata">
          <VersionBadge
            v-if="version.version_type === 'release'"
            type="release"
            color="green"
          />
          <VersionBadge
            v-else-if="version.version_type === 'beta'"
            type="beta"
            color="yellow"
          />
          <VersionBadge
            v-else-if="version.version_type === 'alpha'"
            type="alpha"
            color="red"
          />
          <span class="divider" />
          <span class="version_number">{{ version.version_number }}</span>
        </div>
        <div class="version__supports">
          <span>
            {{ version.loaders.map((x) => $formatCategory(x)).join(', ') }}
          </span>
          <span>{{ $formatVersion(version.game_versions) }}</span>
        </div>
        <div class="version__stats">
          <span>
            <strong>{{ $formatNumber(version.downloads) }}</strong>
            downloads
          </span>
          <span>
            Published on
            <strong>{{
              $dayjs(version.date_published).format('MMM D, YYYY')
            }}</strong>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import VersionBadge from '~/components/ui/Badge'
import VersionFilterControl from '~/components/ui/VersionFilterControl'

export default {
  components: {
    PlusIcon,
    DownloadIcon,
    VersionBadge,
    VersionFilterControl,
  },
  auth: false,
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
    versions: {
      type: Array,
      default() {
        return []
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
  },
  data() {
    return {
      filteredVersions: this.versions,
    }
  },
  fetch() {
    if (this.$route.query.page)
      this.currentPage = parseInt(this.$route.query.page)
  },
  head() {
    const title = `${this.project.title} - Versions`
    const description = `Download and browse ${this.versions.length} ${
      this.project.title
    } versions. ${this.$formatNumber(
      this.project.downloads
    )} total downloads. Last updated ${this.$dayjs(
      this.versions[0] ? this.versions[0].date_published : null
    ).format('MMM D, YYYY')}.`

    return {
      title,
      meta: [
        {
          hid: 'og:title',
          name: 'og:title',
          content: title,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: title,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: description,
        },
        {
          hid: 'description',
          name: 'description',
          content: description,
        },
      ],
    }
  },
  methods: {
    updateVersions(updatedVersions) {
      this.filteredVersions = updatedVersions
    },
  },
}
</script>

<style lang="scss" scoped>
.header-buttons {
  display: flex;
  justify-content: right;
}

.all-versions {
  display: flex;
  flex-direction: column;

  .header {
    display: grid;
    grid-template: 'download title supports stats';
    grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1fr 1fr 1fr;
    color: var(--color-text-dark);
    font-size: var(--font-size-md);
    font-weight: bold;
    justify-content: left;
    margin-inline: var(--spacing-card-md);
    margin-bottom: var(--spacing-card-sm);
    column-gap: var(--spacing-card-sm);

    div:first-child {
      grid-area: download;
    }

    div:nth-child(2) {
      grid-area: title;
    }

    div:nth-child(3) {
      grid-area: supports;
    }

    div:nth-child(4) {
      grid-area: stats;
    }
  }

  .version-button {
    display: grid;
    grid-template: 'download title supports stats' 'download metadata supports stats';
    grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1fr 1fr 1fr;
    column-gap: var(--spacing-card-sm);
    justify-content: left;
    padding: var(--spacing-card-md);

    .download-button {
      grid-area: download;
    }
    .version__title {
      grid-area: title;
      font-weight: bold;
    }
    .version__metadata {
      grid-area: metadata;
      display: flex;
      flex-direction: row;
      flex-wrap: wrap;
      gap: var(--spacing-card-xs);
      margin-top: var(--spacing-card-xs);
    }
    .version__supports {
      grid-area: supports;
      display: flex;
      flex-direction: column;
      gap: var(--spacing-card-xs);
    }
    .version__stats {
      grid-area: stats;
      display: flex;
      flex-direction: column;
      gap: var(--spacing-card-xs);
    }

    &:active:not(&:disabled) {
      transform: scale(0.99) !important;
    }
  }
}

@media screen and (max-width: 1024px) {
  .all-versions {
    .header {
      grid-template: 'download title';
      grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1fr;

      div:nth-child(3) {
        display: none;
      }

      div:nth-child(4) {
        display: none;
      }
    }

    .version-button {
      grid-template: 'download title' 'download metadata' 'download supports' 'download stats';
      grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1fr;
      row-gap: var(--spacing-card-xs);

      .version__supports {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        column-gap: var(--spacing-card-xs);
      }
      .version__metadata {
        margin: 0;
      }
    }
  }
}
</style>
