<template>
  <div class="content">
    <VersionFilterControl
      class="card"
      :versions="versions"
      @updateVersions="updateVersions"
    />
    <div class="card">
      <div v-for="version in filteredVersions" :key="version.id">
        <div class="version-header">
          <span :class="'circle ' + version.version_type" />
          <div class="version-header-text">
            <h2 class="name title-link">
              <nuxt-link
                :to="`/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/version/${encodeURIComponent(version.version_number)}`"
                >{{ version.name }}</nuxt-link
              >
            </h2>
            <span v-if="members.find((x) => x.user.id === version.author_id)">
              by
              <nuxt-link
                class="text-link"
                :to="
                  '/user/' +
                  members.find((x) => x.user.id === version.author_id).user
                    .username
                "
                >{{
                  members.find((x) => x.user.id === version.author_id).user
                    .username
                }}</nuxt-link
              >
            </span>
            <span>
              on
              {{ $dayjs(version.date_published).format('MMM D, YYYY') }}</span
            >
          </div>
          <a
            :href="$parent.findPrimary(version).url"
            class="iconified-button download"
            :title="`Download ${version.name}`"
          >
            <DownloadIcon aria-hidden="true" />
            Download
          </a>
        </div>
        <div
          v-highlightjs
          :class="'markdown-body ' + version.version_type"
          v-html="
            version.changelog
              ? $xss($md.render(version.changelog))
              : 'No changelog specified.'
          "
        />
      </div>
    </div>
  </div>
</template>
<script>
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import VersionFilterControl from '~/components/ui/VersionFilterControl'

export default {
  components: {
    DownloadIcon,
    VersionFilterControl,
  },
  data() {
    return {
      filteredVersions: this.versions,
    }
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
    members: {
      type: Array,
      default() {
        return []
      },
    },
  },
  methods: {
    updateVersions(updatedVersions) {
      this.filteredVersions = updatedVersions
    },
  },
}
</script>

<style lang="scss" scoped>
.version-header {
  display: flex;
  align-items: center;

  .circle {
    min-width: 0.75rem;
    min-height: 0.75rem;
    border-radius: 50%;
    display: inline-block;
    margin-right: 0.25rem;

    &.alpha {
      background-color: var(--color-badge-red-bg);
    }

    &.release {
      background-color: var(--color-badge-green-bg);
    }

    &.beta {
      background-color: var(--color-badge-yellow-bg);
    }
  }

  .version-header-text {
    display: flex;
    align-items: baseline;
    margin: 0 0.75rem;
    flex-wrap: wrap;

    h2 {
      margin: 0;
      font-size: var(--font-size-lg);
    }

    h2,
    span {
      padding-right: 0.25rem;
    }
  }

  .download {
    display: none;

    @media screen and (min-width: 800px) {
      display: flex;
    }
  }
}

.markdown-body {
  margin: 0.5rem 0.5rem 1rem calc(0.375rem - 1px);
  padding-left: 1.275rem;
  border-left: 2px solid var(--color-text);

  &.alpha {
    border-left-color: var(--color-badge-red-bg);
  }

  &.release {
    border-left-color: var(--color-badge-green-bg);
  }

  &.beta {
    border-left-color: var(--color-badge-yellow-bg);
  }
}
</style>
