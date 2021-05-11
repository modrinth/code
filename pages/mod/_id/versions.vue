<template>
  <div>
    <table>
      <thead>
        <tr>
          <th></th>
          <th>Name</th>
          <th>Version</th>
          <th>Mod Loader</th>
          <th>Minecraft Version</th>
          <th>Status</th>
          <th>Downloads</th>
          <th>Date Published</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="version in versions" :key="version.id">
          <td>
            <a
              :href="$parent.findPrimary(version).url"
              class="download"
              @click.prevent="
                $parent.downloadFile(
                  $parent.findPrimary(version).hashes.sha1,
                  $parent.findPrimary(version).url
                )
              "
            >
              <DownloadIcon />
            </a>
          </td>
          <td>
            <nuxt-link
              :to="
                '/mod/' +
                (mod.slug ? mod.slug : mod.id) +
                '/version/' +
                version.id
              "
            >
              {{ version.name ? version.name : version.version_number }}
            </nuxt-link>
          </td>
          <td>
            <nuxt-link
              :to="
                '/mod/' +
                (mod.slug ? mod.slug : mod.id) +
                '/version/' +
                version.id
              "
            >
              {{ version.version_number }}
            </nuxt-link>
          </td>
          <td>
            <FabricIcon v-if="version.loaders.includes('fabric')" />
            <ForgeIcon v-if="version.loaders.includes('forge')" />
          </td>
          <td>{{ version.game_versions.join(', ') }}</td>
          <td>
            <span v-if="version.version_type === 'release'" class="badge green">
              Release
            </span>
            <span v-if="version.version_type === 'beta'" class="badge yellow">
              Beta
            </span>
            <span v-if="version.version_type === 'alpha'" class="badge red">
              Alpha
            </span>
          </td>
          <td>{{ version.downloads }}</td>
          <td>{{ $dayjs(version.date_published).format('YYYY-MM-DD') }}</td>
        </tr>
      </tbody>
    </table>
    <div class="new-version">
      <nuxt-link v-if="currentMember" to="newversion" class="button">
        New Version
      </nuxt-link>
    </div>
  </div>
</template>
<script>
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'

export default {
  components: {
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
  },
  auth: false,
  props: {
    mod: {
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
  created() {
    this.$emit('update:link-bar', [['Versions', 'versions']])
  },
}
</script>

<style lang="scss" scoped>
table {
  border-collapse: collapse;
  margin-bottom: var(--spacing-card-md);
  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);
  table-layout: fixed;
  width: 100%;

  * {
    text-align: left;
  }

  tr:not(:last-child),
  tr:first-child {
    th,
    td {
      border-bottom: 1px solid var(--color-divider);
    }
  }

  th,
  td {
    &:first-child {
      text-align: center;
      width: 7%;

      svg {
        color: var(--color-text);

        &:hover,
        &:focus {
          color: var(--color-text-hover);
        }
      }
    }

    &:nth-child(2),
    &:nth-child(5) {
      padding-left: 0;
      width: 12%;
    }
  }

  th {
    color: var(--color-heading);
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    padding: 0.75rem 1rem;
    text-transform: uppercase;
  }

  td {
    overflow: hidden;
    padding: 0.75rem 1rem;

    img {
      height: 3rem;
      width: 3rem;
    }
  }
}

.new-version {
  width: 100%;
  text-align: right;
  margin-bottom: var(--spacing-card-md);
}

@media screen and (max-width: 400px) {
  th,
  td {
    &:nth-child(7) {
      display: none;
    }
  }
}

@media screen and (max-width: 600px) {
  th,
  td {
    &:nth-child(8) {
      display: none;
    }
  }
}

@media screen and (max-width: 800px) {
  th,
  td {
    &:nth-child(5) {
      display: none;
    }
  }
}

@media screen and (max-width: 1000px) {
  th,
  td {
    &:nth-child(2) {
      display: none;
    }
  }
}
</style>
