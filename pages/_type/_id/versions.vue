<template>
  <div class="content card">
    <nuxt-link
      v-if="currentMember"
      to="version/create"
      class="iconified-button new-version"
    >
      <UploadIcon />
      Upload
    </nuxt-link>
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
        <tr v-for="version in versions" :key="version.id">
          <td>
            <a
              :href="$parent.findPrimary(version).url"
              class="download-button"
              :title="`Download ${version.name}`"
              @click.prevent="
                $parent.downloadFile(
                  $parent.findPrimary(version).hashes.sha1,
                  $parent.findPrimary(version).url
                )
              "
            >
              <DownloadIcon aria-hidden="true" />
            </a>
          </td>
          <td>
            <div class="info">
              <div class="top">
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
                <span class="version_number">{{ version.version_number }}</span>
              </div>
              <div class="mobile-info">
                <p>
                  {{
                    version.loaders
                      .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                      .join(', ') +
                    ' ' +
                    $formatVersion(version.game_versions)
                  }}
                </p>
                <p></p>
                <p>
                  <strong>{{ $parent.formatNumber(version.downloads) }}</strong>
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
                  .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                  .join(', ')
              }}
            </p>
            <p>{{ $formatVersion(version.game_versions) }}</p>
          </td>
          <td>
            <p>
              <span>{{ $parent.formatNumber(version.downloads) }}</span>
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
</template>
<script>
import UploadIcon from '~/assets/images/utils/upload.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import VersionBadge from '~/components/ui/Badge'

export default {
  components: {
    UploadIcon,
    DownloadIcon,
    VersionBadge,
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
}
</script>

<style lang="scss" scoped>
.content {
  max-width: calc(100% - (2 * var(--spacing-card-lg)));
}

.new-version {
  max-width: 5.25rem;
}

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
      min-width: 13.875rem;
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
</style>
