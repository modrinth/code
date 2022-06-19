<template>
  <div class="content">
    <div v-if="currentMember" class="card header-buttons">
      <nuxt-link
        to="version/create"
        class="brand-button-colors iconified-button"
      >
        <PlusIcon />
        Create a version
      </nuxt-link>
    </div>
    <VersionFilterControl
      class="card"
      :versions="versions"
      @updateVersions="updateVersions"
    />
    <div v-if="versions.length > 0" class="card">
      <table>
        <thead>
          <tr>
            <th role="presentation"></th>
            <th>Version</th>
            <th>Supports</th>
            <th>Stats</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="version in filteredVersions" :key="version.id">
            <td>
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
              >
                <DownloadIcon aria-hidden="true" />
              </a>
            </td>
            <td>
              <div class="info">
                <div class="top title-link">
                  <nuxt-link
                    :to="`/${project.project_type}/${
                      project.slug ? project.slug : project.id
                    }/version/${encodeURIComponent(version.version_number)}`"
                  >
                    {{ version.name }}
                  </nuxt-link>
                </div>
                <div class="bottom">
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
                  <span class="version_number">{{
                    version.version_number
                  }}</span>
                </div>
                <div class="mobile-info">
                  <p>
                    {{
                      version.loaders
                        .map((x) =>
                          x.toLowerCase() === 'modloader'
                            ? 'ModLoader'
                            : x.charAt(0).toUpperCase() + x.slice(1)
                        )
                        .join(', ') +
                      ' ' +
                      $formatVersion(version.game_versions)
                    }}
                  </p>
                  <p></p>
                  <p>
                    <strong>{{ $formatNumber(version.downloads) }}</strong>
                    downloads
                  </p>
                  <p>
                    Published on
                    <strong>{{
                      $dayjs(version.date_published).format('MMM D, YYYY')
                    }}</strong>
                  </p>
                </div>
              </div>
            </td>
            <td>
              <p>
                {{
                  version.loaders
                    .map((x) =>
                      x.toLowerCase() === 'modloader'
                        ? 'ModLoader'
                        : x.charAt(0).toUpperCase() + x.slice(1)
                    )
                    .join(', ')
                }}
              </p>
              <p>{{ $formatVersion(version.game_versions) }}</p>
            </td>
            <td>
              <p>
                <span>{{ $formatNumber(version.downloads) }}</span>
                downloads
              </p>
              <p>
                Published on
                <span>{{
                  $dayjs(version.date_published).format('MMM D, YYYY')
                }}</span>
              </p>
            </td>
          </tr>
        </tbody>
      </table>
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
  methods: {
    updateVersions(updatedVersions) {
      this.filteredVersions = updatedVersions
    },
  },
}
</script>

<style lang="scss" scoped>
table {
  border-collapse: separate;
  border-spacing: 0 0.75rem;

  th {
    text-align: left;
    font-size: var(--font-size-md);

    &:nth-child(3),
    &:nth-child(4) {
      display: none;
    }
  }

  tr {
    td:nth-child(2) {
      padding-right: 2rem;
      min-width: 16rem;
      .top {
        font-weight: bold;
      }
      .bottom {
        display: flex;
        flex-direction: row;
        align-items: center;
        text-overflow: ellipsis;
        margin-top: 0.25rem;

        .divider {
          width: 0.25rem;
          height: 0.25rem;
          border-radius: 50%;
          display: inline-block;
          margin: 0 0.25rem;
          background-color: var(--color-text);
        }
      }

      .mobile-info {
        p {
          margin: 0.25rem 0 0;
        }
      }
    }
    td:nth-child(3) {
      display: none;
      width: 100%;
      p {
        margin: 0.25rem 0;
      }
    }
    td:nth-child(4) {
      display: none;
      min-width: 15rem;
      p {
        margin: 0.25rem 0;
        span {
          font-weight: bold;
        }
      }
    }
  }
}

@media screen and (min-width: 1024px) {
  table {
    tr {
      th:nth-child(3),
      td:nth-child(3),
      th:nth-child(4),
      td:nth-child(4) {
        display: table-cell;
      }
    }
  }

  .mobile-info {
    display: none;
  }
}

.header-buttons {
  display: flex;
  justify-content: right;
}
</style>
