<template>
  <div class="content">
    <VersionFilterControl
      class="card"
      :versions="versions"
      @updateVersions="updateVersions"
    />
    <div class="card">
      <div
        v-for="version in filteredVersions"
        :key="version.id"
        class="changelog-item"
      >
        <div
          :class="`changelog-bar ${version.version_type} ${
            version.duplicate ? 'duplicate' : ''
          }`"
        ></div>
        <div class="version-wrapper">
          <div class="version-header">
            <div class="version-header-text">
              <h2 class="name">
                <nuxt-link
                  :to="`/${project.project_type}/${
                    project.slug ? project.slug : project.id
                  }/version/${encodeURI(version.displayUrlEnding)}`"
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
            v-if="version.changelog && !version.duplicate"
            v-highlightjs
            class="markdown-body"
            v-html="$xss($md.render(version.changelog))"
          />
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import VersionFilterControl from '~/components/ui/VersionFilterControl'

export default {
  components: {
    VersionFilterControl,
    DownloadIcon,
  },
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
  data() {
    return {
      filteredVersions: this.$calculateDuplicates(this.versions),
      currentPage: 1,
    }
  },
  fetch() {
    if (this.$route.query.page) {
      this.currentPage = parseInt(this.$route.query.page)
    }
  },
  head() {
    const title = `${this.project.title} - Changelog`
    const description = `Explore the changelog of ${this.project.title}'s ${this.versions.length} versions.`

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
    async switchPage(page, toTop) {
      this.currentPage = page
      await this.$router.replace(this.getPageLink(page))

      if (toTop) {
        setTimeout(() => window.scrollTo({ top: 0, behavior: 'smooth' }), 50)
        setTimeout(() => window.scrollTo({ top: 0, behavior: 'smooth' }), 50)
      }
    },
    getPageLink(page) {
      if (page === 1) {
        return this.$route.path
      } else {
        return `${this.$route.path}?page=${this.currentPage}`
      }
    },
    updateVersions(updatedVersions) {
      this.filteredVersions = this.$calculateDuplicates(updatedVersions)
    },
  },
  auth: false,
}
</script>

<style lang="scss" scoped>
.changelog-item {
  display: block;
  margin-bottom: 1rem;
  position: relative;
  padding-left: 1.8rem;

  .changelog-bar {
    --color: var(--color-special-green);

    &.alpha {
      --color: var(--color-special-red);
    }

    &.release {
      --color: var(--color-special-green);
    }

    &.beta {
      --color: var(--color-special-orange);
    }

    left: 0;
    top: 0.5rem;
    width: 0.2rem;
    min-width: 0.2rem;
    position: absolute;
    margin: 0 0.4rem;
    border-radius: var(--size-rounded-max);
    min-height: 100%;
    background-color: var(--color);

    &:before {
      content: '';
      width: 1rem;
      height: 1rem;
      position: absolute;
      top: 0;
      left: -0.4rem;
      border-radius: var(--size-rounded-max);
      background-color: var(--color);
    }

    &.duplicate {
      background: linear-gradient(
        to bottom,
        transparent,
        transparent 30%,
        var(--color) 30%,
        var(--color)
      );
      background-size: 100% 10px;
    }

    &.duplicate {
      height: calc(100% + 1.5rem);
    }
  }
}

.version-header {
  display: flex;
  align-items: center;
  margin-top: 0.2rem;

  .circle {
    min-width: 0.75rem;
    min-height: 0.75rem;
    border-radius: 50%;
    display: inline-block;
    margin-right: 0.25rem;
  }

  .version-header-text {
    display: flex;
    align-items: baseline;
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
  margin: 0.5rem 0.5rem 0 0;
}
</style>
