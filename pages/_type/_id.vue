<template>
  <div class="page-container">
    <div
      :class="{
        'page-contents': true,
        'alt-layout': $store.state.cosmetics.projectLayout,
      }"
    >
      <div class="header card">
        <nuxt-link
          :to="
            '/' +
            project.project_type +
            '/' +
            (project.slug ? project.slug : project.id)
          "
        >
          <img
            class="icon"
            :src="
              project.icon_url
                ? project.icon_url
                : 'https://cdn.modrinth.com/placeholder.svg?inline'
            "
            alt="project - icon"
        /></nuxt-link>
        <nuxt-link
          :to="
            '/' +
            project.project_type +
            '/' +
            (project.slug ? project.slug : project.id)
          "
        >
          <h1 class="title">{{ project.title }}</h1>
        </nuxt-link>
        <div
          v-if="
            project.client_side === 'optional' &&
            project.server_side === 'optional'
          "
          class="side-descriptor"
        >
          <InfoIcon aria-hidden="true" />
          Universal {{ project.project_type }}
        </div>
        <div
          v-else-if="
            (project.client_side === 'optional' ||
              project.client_side === 'required') &&
            (project.server_side === 'optional' ||
              project.server_side === 'unsupported')
          "
          class="side-descriptor"
        >
          <InfoIcon aria-hidden="true" />
          Client {{ project.project_type }}
        </div>
        <div
          v-else-if="
            (project.server_side === 'optional' ||
              project.server_side === 'required') &&
            (project.client_side === 'optional' ||
              project.client_side === 'unsupported')
          "
          class="side-descriptor"
        >
          <InfoIcon aria-hidden="true" />
          Server {{ project.project_type }}
        </div>
        <p class="description">
          {{ project.description }}
        </p>
        <Categories
          :categories="project.categories"
          :type="project.project_type"
          class="categories"
        />
        <hr class="card-divider" />
        <div class="stats">
          <span class="stat">{{ $formatNumber(project.downloads) }}</span>
          <span class="label"
            >download<span v-if="project.downloads !== 1">s</span></span
          >
          <span class="stat">{{ $formatNumber(project.followers) }}</span>
          <span class="label"
            >follower<span v-if="project.followers !== 1">s</span></span
          >
        </div>
        <div class="dates">
          <div
            v-tooltip="$dayjs(createdAt).format('MMMM D, YYYY [at] h:mm:ss A')"
            class="date"
          >
            <CalendarIcon aria-hidden="true" />
            <span class="label">Created</span>
            <span class="value">{{ $dayjs(project.published).fromNow() }}</span>
          </div>
          <div
            v-tooltip="$dayjs(updatedAt).format('MMMM D, YYYY [at] h:mm:ss A')"
            class="date"
          >
            <UpdateIcon aria-hidden="true" />
            <span class="label">Updated</span>
            <span class="value">{{ $dayjs(project.updated).fromNow() }}</span>
          </div>
        </div>
        <hr v-if="$auth.user" class="card-divider" />
        <div class="buttons">
          <nuxt-link
            v-if="$auth.user"
            :to="`/create/report?id=${project.id}&t=project`"
            class="iconified-button"
          >
            <ReportIcon aria-hidden="true" />
            Report
          </nuxt-link>
          <button
            v-if="$auth.user && !$user.follows.find((x) => x.id === project.id)"
            class="iconified-button"
            @click="$store.dispatch('user/followProject', project)"
          >
            <FollowIcon aria-hidden="true" />
            Follow
          </button>
          <button
            v-if="$auth.user && $user.follows.find((x) => x.id === project.id)"
            class="iconified-button"
            @click="$store.dispatch('user/unfollowProject', project)"
          >
            <FollowIcon fill="currentColor" aria-hidden="true" />
            Unfollow
          </button>
        </div>
      </div>
      <div
        v-if="
          currentMember &&
          (project.status === 'processing' ||
            (project.moderator_message &&
              (project.moderator_message.message ||
                project.moderator_message.body)))
        "
        class="project-status card"
      >
        <h3 class="card-header">Project status</h3>
        <div class="status-info"></div>
        <p>
          Your project is currently:
          <VersionBadge
            v-if="project.status === 'approved'"
            color="green"
            :type="project.status"
          />
          <VersionBadge
            v-else-if="
              project.status === 'processing' || project.status === 'archived'
            "
            color="yellow"
            :type="project.status"
          />
          <VersionBadge
            v-else-if="project.status === 'rejected'"
            color="red"
            :type="project.status"
          />
          <VersionBadge v-else color="gray" :type="project.status" />
        </p>
        <div class="message">
          <p v-if="project.status === 'processing'">
            Your project is currently not viewable by people who are not part of
            your team. Please wait for our moderators to manually review your
            project to see if it abides by our project rules!
          </p>
          <p v-if="project.status === 'draft'">
            Your project is currently not viewable by people who are not part of
            your team. If your project is ready for review, click the button
            below to make your mod public!
          </p>
          <p v-if="project.moderator_message">
            {{ project.moderator_message.message }}
          </p>
          <div
            v-if="project.moderator_message && project.moderator_message.body"
            v-highlightjs
            class="markdown-body"
            v-html="$xss($md.render(project.moderator_message.body))"
          ></div>
        </div>
        <div class="buttons">
          <button
            v-if="
              project.status !== 'processing' && project.status !== 'approved'
            "
            class="iconified-button"
            @click="submitForReview"
          >
            Resubmit for approval
          </button>
          <button
            v-if="project.status === 'approved'"
            class="iconified-button"
            @click="clearMessage"
          >
            Clear message
          </button>
        </div>
      </div>
      <div class="extra-info card">
        <template
          v-if="
            project.issues_url ||
            project.source_url ||
            project.wiki_url ||
            project.discord_url
          "
        >
          <h3 class="card-header">External resources</h3>
          <div class="links">
            <a
              v-if="project.issues_url"
              :href="project.issues_url"
              class="title"
              target="_blank"
            >
              <IssuesIcon aria-hidden="true" />
              <span>Issues</span>
            </a>
            <a
              v-if="project.source_url"
              :href="project.source_url"
              class="title"
              target="_blank"
            >
              <CodeIcon aria-hidden="true" />
              <span>Source</span>
            </a>
            <a
              v-if="project.wiki_url"
              :href="project.wiki_url"
              class="title"
              target="_blank"
            >
              <WikiIcon aria-hidden="true" />
              <span>Wiki</span>
            </a>
            <a
              v-if="project.discord_url"
              :href="project.discord_url"
              target="_blank"
            >
              <DiscordIcon
                v-if="$colorMode.value === 'light'"
                class="shrink"
                aria-hidden="true"
              />
              <DiscordIconWhite v-else class="shrink" aria-hidden="true" />
              <span>Discord</span>
            </a>
            <a
              v-for="(donation, index) in project.donation_urls"
              :key="index"
              :href="donation.url"
              target="_blank"
            >
              <BuyMeACoffeeLogo
                v-if="donation.id === 'bmac' && $colorMode.value === 'light'"
                aria-hidden="true"
              />
              <BuyMeACoffeeLogoWhite
                v-else-if="
                  donation.id === 'bmac' && $colorMode.value === 'dark'
                "
                aria-hidden="true"
              />
              <img
                v-else-if="
                  donation.id === 'patreon' && $colorMode.value === 'light'
                "
                class="shrink"
                alt=""
                src="~/assets/images/external/patreon.png"
              />
              <img
                v-else-if="
                  donation.id === 'patreon' && $colorMode.value === 'dark'
                "
                class="shrink"
                alt=""
                src="~/assets/images/external/patreon-white.png"
              />
              <img
                v-else-if="
                  donation.id === 'paypal' && $colorMode.value === 'light'
                "
                class="shrink"
                alt=""
                src="~/assets/images/external/paypal.png"
              />
              <img
                v-else-if="
                  donation.id === 'paypal' && $colorMode.value === 'dark'
                "
                class="shrink"
                alt=""
                src="~/assets/images/external/paypal-white.png"
              />
              <img
                v-else-if="
                  donation.id === 'ko-fi' && $colorMode.value === 'light'
                "
                alt=""
                src="~/assets/images/external/kofi.png"
              />
              <img
                v-else-if="
                  donation.id === 'ko-fi' && $colorMode.value === 'dark'
                "
                alt=""
                src="~/assets/images/external/kofi-white.png"
              />
              <FollowIcon v-else-if="donation.id === 'github'" />
              <UnknownIcon v-else />
              <span v-if="donation.id === 'bmac'">Buy Me a Coffee</span>
              <span v-else-if="donation.id === 'patreon'">Patreon</span>
              <span v-else-if="donation.id === 'paypal'">PayPal</span>
              <span v-else-if="donation.id === 'ko-fi'">Ko-fi</span>
              <span v-else-if="donation.id === 'github'">GitHub Sponsors</span>
              <span v-else>Donate</span>
            </a>
          </div>
          <hr class="card-divider" />
        </template>
        <template v-if="featuredVersions.length > 0">
          <h3 class="card-header">Featured versions</h3>
          <div
            v-for="version in featuredVersions"
            :key="version.id"
            class="featured-version"
          >
            <a
              :href="findPrimary(version).url"
              class="download"
              :title="`Download ${version.name}`"
            >
              <DownloadIcon aria-hidden="true" />
            </a>
            <div class="info">
              <nuxt-link
                :to="`/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/version/${encodeURIComponent(version.version_number)}`"
                class="top title-link"
              >
                {{ version.name }}
              </nuxt-link>
              <div
                v-if="version.game_versions.length > 0"
                class="game-version item"
              >
                {{
                  version.loaders
                    .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                    .join(', ')
                }}
                {{ $formatVersion(version.game_versions) }}
              </div>
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
            </div>
          </div>
          <hr class="card-divider" />
        </template>
        <h3 class="card-header">Project members</h3>
        <div
          v-for="member in members"
          :key="member.user.id"
          class="team-member columns"
        >
          <nuxt-link :to="'/user/' + member.user.username" class="name">
            <img :src="member.avatar_url" alt="profile-picture" />
          </nuxt-link>
          <div class="member-info">
            <nuxt-link :to="'/user/' + member.user.username" class="name">
              <p class="title-link">{{ member.name }}</p>
            </nuxt-link>
            <p class="role">{{ member.role }}</p>
          </div>
        </div>
        <hr class="card-divider" />
        <h3 class="card-header">Technical information</h3>
        <div class="infos">
          <div class="info">
            <div class="key">License</div>
            <div class="value uppercase">
              <a class="text-link" :href="project.license.url || null">{{
                project.license.id
              }}</a>
            </div>
          </div>
          <div class="info">
            <div class="key">Client side</div>
            <div class="value">
              {{ project.client_side }}
            </div>
          </div>
          <div class="info">
            <div class="key">Server side</div>
            <div class="value">
              {{ project.server_side }}
            </div>
          </div>
          <div class="info">
            <div class="key">Project ID</div>
            <div class="value lowercase">
              {{ project.id }}
            </div>
          </div>
        </div>
      </div>
      <Advertisement
        v-if="project.status === 'approved' || project.status === 'unlisted'"
        class="small-advertisement"
        type="square"
        small-screen="destroy"
      />
      <div class="content">
        <div class="project-main">
          <div v-if="project.project_type === 'modpack'" class="card warning">
            To install this modpack, visit
            <a
              href="https://docs.modrinth.com/docs/modpacks/playing_modpacks/"
              target="_blank"
              >our documentation</a
            >
            which provides instructions on using
            <a href="https://atlauncher.com/about" target="_blank">ATLauncher</a
            >, <a href="https://multimc.org/" target="_blank">MultiMC</a> and
            <a href="https://polymc.org/" target="_blank">PolyMC</a>.
          </div>
          <div class="card styled-tabs">
            <nuxt-link
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }`"
              class="tab"
              exact
            >
              <span>Description</span>
            </nuxt-link>
            <nuxt-link
              v-if="project.gallery.length > 0 || currentMember"
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/gallery`"
              class="tab"
            >
              <span>Gallery</span>
            </nuxt-link>
            <nuxt-link
              v-if="project.versions.length > 0"
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/changelog`"
              class="tab"
            >
              <span>Changelog</span>
            </nuxt-link>
            <nuxt-link
              v-if="project.versions.length > 0"
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/versions`"
              class="tab"
            >
              <span>Versions</span>
            </nuxt-link>
            <nuxt-link
              v-if="currentMember"
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/settings`"
              class="tab"
            >
              <span>Settings</span>
            </nuxt-link>
          </div>
          <Advertisement
            v-if="
              project.status === 'approved' || project.status === 'unlisted'
            "
            type="banner"
            small-screen="square"
            ethical-ads-small
            ethical-ads-big
          />
          <div class="project-content">
            <NuxtChild
              :project.sync="project"
              :versions.sync="versions"
              :featured-versions.sync="featuredVersions"
              :members.sync="members"
              :current-member="currentMember"
              :all-members.sync="allMembers"
              :dependencies.sync="dependencies"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import UpdateIcon from '~/assets/images/utils/updated.svg?inline'
import CodeIcon from '~/assets/images/sidebar/mod.svg?inline'
import ReportIcon from '~/assets/images/utils/report.svg?inline'
import FollowIcon from '~/assets/images/utils/heart.svg?inline'
import InfoIcon from '~/assets/images/utils/info.svg?inline'
import IssuesIcon from '~/assets/images/utils/issues.svg?inline'
import WikiIcon from '~/assets/images/utils/wiki.svg?inline'
import DiscordIcon from '~/assets/images/external/discord.svg?inline'
import DiscordIconWhite from '~/assets/images/external/discord-white.svg?inline'
import BuyMeACoffeeLogo from '~/assets/images/external/bmac.svg?inline'
import BuyMeACoffeeLogoWhite from '~/assets/images/external/bmac-white.svg?inline'
import UnknownIcon from '~/assets/images/utils/unknown.svg?inline'

import Advertisement from '~/components/ads/Advertisement'
import VersionBadge from '~/components/ui/Badge'
import Categories from '~/components/ui/search/Categories'

export default {
  components: {
    VersionBadge,
    Advertisement,
    IssuesIcon,
    DownloadIcon,
    CalendarIcon,
    UpdateIcon,
    CodeIcon,
    ReportIcon,
    FollowIcon,
    InfoIcon,
    WikiIcon,
    DiscordIcon,
    DiscordIconWhite,
    BuyMeACoffeeLogo,
    BuyMeACoffeeLogoWhite,
    UnknownIcon,
    Categories,
  },
  async asyncData(data) {
    const projectTypes = ['mod', 'modpack']

    try {
      if (
        !data.params.id ||
        !projectTypes.includes(data.params.type.toLowerCase())
      ) {
        data.error({
          statusCode: 404,
          message: 'The page could not be found',
        })

        return
      }

      const [project, members, dependencies, versions, featuredVersions] = (
        await Promise.all([
          data.$axios.get(`project/${data.params.id}`, data.$auth.headers),
          data.$axios.get(
            `project/${data.params.id}/members`,
            data.$auth.headers
          ),
          data.$axios.get(
            `project/${data.params.id}/dependencies`,
            data.$auth.headers
          ),
          data.$axios.get(
            `project/${data.params.id}/version`,
            data.$auth.headers
          ),
          data.$axios.get(
            `project/${data.params.id}/version?featured=true`,
            data.$auth.headers
          ),
        ])
      ).map((it) => it.data)

      if (project.project_type !== data.params.type) {
        data.error({
          statusCode: 404,
          message: 'Project not found',
        })

        return
      }

      members.forEach((it, index) => {
        members[index].avatar_url = it.user.avatar_url
        members[index].name = it.user.username
      })

      const currentMember = data.$auth.user
        ? members.find((x) => x.user.id === data.$auth.user.id)
        : null

      if (project.body_url && !project.body) {
        project.body = (await data.$axios.get(project.body_url)).data
      }

      return {
        project,
        versions,
        featuredVersions,
        members: members.filter((x) => x.accepted),
        allMembers: members,
        currentMember,
        dependencies,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Project not found',
      })
    }
  },
  head() {
    return {
      title: `${this.project.title} - ${
        this.project.project_type.charAt(0).toUpperCase() +
        this.project.project_type.slice(1)
      }s - Modrinth`,
      meta: [
        {
          hid: 'og:type',
          name: 'og:type',
          content: 'website',
        },
        {
          hid: 'og:title',
          name: 'og:title',
          content: this.project.title,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: this.project.title,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.project.description,
        },
        {
          hid: 'description',
          name: 'description',
          content: `${this.project.title}: ${this.project.description} View other minecraft mods on Modrinth today! Modrinth is a new and modern Minecraft modding platform supporting both the Forge and Fabric mod loaders.`,
        },
        {
          hid: 'og:url',
          name: 'og:url',
          content: `https://modrinth.com/${this.project.project_type}/${this.project.id}`,
        },
        {
          hid: 'og:image',
          name: 'og:image',
          content: this.project.icon_url
            ? this.project.icon_url
            : 'https://cdn.modrinth.com/placeholder.png',
        },
        {
          hid: 'robots',
          name: 'robots',
          content: this.project.status !== 'approved' ? 'noindex' : 'all',
        },
      ],
    }
  },
  methods: {
    findPrimary(version) {
      let file = version.files.find((x) => x.primary)

      if (!file) {
        file = version.files[0]
      }

      if (!file) {
        file = { url: `/project/${this.project.id}/version/${version.id}` }
      }

      return file
    },
    async clearMessage() {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `project/${this.project.id}`,
          {
            moderation_message: null,
            moderation_message_body: null,
          },
          this.$auth.headers
        )

        this.project.moderator_message = null
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
    async submitForReview() {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `project/${this.project.id}`,
          {
            status: 'processing',
          },
          this.$auth.headers
        )

        this.project.status = 'processing'
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
  },
}
</script>
<style lang="scss" scoped>
.page-contents {
  display: grid;

  grid-template:
    'header'
    'project-status'
    'content'
    'extra-info'
    'small-advert'
    / 100%;

  @media screen and (min-width: 1024px) {
    grid-template:
      'header       content' auto
      'project-status      content' auto
      'extra-info       content' auto
      'small-advert       content' auto
      'dummy content' 1fr
      / 20rem calc(100% - 20rem);

    &.alt-layout {
      grid-template:
        'content       header' auto
        'content      project-status' auto
        'content       extra-info' auto
        'content       small-advert' auto
        'content       dummy' 1fr
        / 1fr 20rem;
    }
  }

  column-gap: var(--spacing-card-md);
}

.header {
  grid-area: header;

  .icon {
    width: 6rem;
    height: 6rem;
    object-fit: contain;
    border-radius: var(--size-rounded-icon);
  }

  .title {
    margin: 0.25rem 0;
    color: var(--color-text-dark);
    font-size: var(--font-size-xl);
  }

  .side-descriptor {
    display: flex;
    align-items: center;
    color: var(--color-text-dark);
    font-weight: bold;
    font-size: var(--font-size-sm);
    margin-bottom: 0.5rem;

    svg {
      height: 1.25rem;
      margin-right: 0.125rem;
    }
  }

  .buttons {
    display: flex;
    flex-direction: row;

    button,
    a {
      display: flex;
    }
  }

  .description {
    margin-top: var(--spacing-card-sm);
    margin-bottom: 0.5rem;
    color: var(--color-text-dark);
    font-size: var(--font-size-nm);
  }

  .categories {
    margin: 0.25rem 0;
    color: var(--color-text-secondary);
    font-size: var(--font-size-nm);
  }

  .stats {
    .stat {
      font-size: var(--font-size-lg);
      font-weight: bold;
    }

    .label {
      margin-right: 0.125rem;
    }
  }

  .dates {
    margin: 0.75rem 0;

    .date {
      color: var(--color-text-secondary);
      font-size: var(--font-size-nm);
      display: flex;
      align-items: center;
      margin-bottom: 0.25rem;

      .label {
        margin-right: 0.25rem;
      }

      svg {
        height: 1rem;
        margin-right: 0.25rem;
      }
    }
  }
}

.project-status {
  grid-area: project-status;
}

.extra-info {
  grid-area: extra-info;
}

.small-advertisement {
  grid-area: small-advert;
}

.content {
  grid-area: content;
}

.project-info {
  height: auto;
  overflow: hidden;
}

.card-header {
  font-weight: bold;
  color: var(--color-heading);
  margin-bottom: 0.3rem;
}

.featured-version {
  display: flex;
  flex-direction: row;
  margin-top: var(--spacing-card-md);

  .download {
    display: flex;
    align-items: center;
    height: 2.5rem;
    width: 2.5rem;
    border-radius: 1.5rem;
    color: var(--color-brand-inverted);
    background-color: var(--color-brand);
    margin-right: var(--spacing-card-sm);

    &:hover {
      background-color: var(--color-brand-hover);
    }

    &:active {
      background-color: var(--color-brand-active);
    }

    svg {
      width: 1.5rem;
      margin: auto;
    }

    flex-shrink: 0;
  }

  .info {
    display: flex;
    flex-direction: column;

    .top {
      font-weight: bold;
    }
  }
}

.links {
  a {
    display: inline-flex;
    align-items: center;
    border-radius: 1rem;

    svg,
    img {
      height: 1rem;
      width: 1rem;
    }

    span {
      margin-left: 0.25rem;
      text-decoration: underline;
      line-height: 2rem;
    }

    &:focus-visible,
    &:hover {
      svg,
      img,
      span {
        color: var(--color-heading);
      }
    }

    &:active {
      svg,
      img,
      span {
        color: var(--color-text-dark);
      }
    }

    &:not(:last-child)::after {
      content: '•';
      margin: 0 0.25rem;
    }
  }
}

.team-member {
  align-items: center;
  margin-bottom: 0.25rem;

  img {
    border-radius: var(--size-rounded-sm);
    height: 50px;
    width: 50px;
  }

  .member-info {
    overflow: hidden;
    margin: auto 0 auto 0.5rem;

    .name {
      font-weight: bold;
    }

    p {
      font-size: var(--font-size-sm);
      margin: 0.2rem 0;
    }
  }
}

.infos {
  .info {
    display: flex;
    margin: 0.5rem 0;

    .key {
      font-weight: bold;
      color: var(--color-text-secondary);
      width: 40%;
    }

    .value {
      width: 50%;
      text-transform: capitalize;

      &.lowercase {
        text-transform: none;
      }
    }

    .uppercase {
      text-transform: uppercase;
    }
  }
}

@media screen and (max-width: 550px) {
  .title a {
    display: none;
  }
}

@media screen and (max-width: 800px) {
  .project-navigation {
    display: block;
    overflow-x: auto;
    overflow-wrap: break-word;
    overflow-y: hidden;
  }
}

@media screen and (min-width: 1024px) {
  .content {
    max-width: calc(1280px - 21rem);
  }
}
</style>
