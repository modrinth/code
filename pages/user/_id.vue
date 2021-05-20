<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div class="card">
          <div class="user-info">
            <img :src="user.avatar_url" :alt="user.username" />
            <div class="text">
              <h2>{{ user.username }}</h2>
              <p v-if="user.role === 'admin'" class="badge red">Admin</p>
              <p v-if="user.role === 'moderator'" class="badge yellow">
                Moderator
              </p>
              <p v-if="user.role === 'developer'" class="badge green">
                Developer
              </p>
            </div>
          </div>
          <p v-if="user.bio" class="bio">{{ user.bio }}</p>
          <div class="buttons">
            <nuxt-link
              v-if="this.$auth.user && this.$auth.user.id != user.id"
              :to="`/report/create?id=${user.id}&t=user`"
              class="iconified-button"
            >
              <ReportIcon />
              Report
            </nuxt-link>
          </div>
        </div>
        <div class="card stats">
          <div class="stat">
            <CalendarIcon />
            <div class="info">
              <h4>Joined</h4>
              <p
                v-tooltip="
                  $dayjs(user.created).format(
                    '[Joined] YYYY-MM-DD [at] HH:mm A'
                  )
                "
                class="value"
              >
                {{ $dayjs(user.created).fromNow() }}
              </p>
            </div>
          </div>
          <div class="stat">
            <DownloadIcon />
            <div class="info">
              <h4>Downloads</h4>
              <p class="value">
                {{ sumDownloads() }}
              </p>
            </div>
          </div>
        </div>
        <Advertisement
          type="square"
          small-screen="square"
          ethical-ads-big
          ethical-ads-small
          ethical-ad-type="image"
        />
        <m-footer class="footer" hide-small />
      </div>
      <div class="content">
        <Advertisement type="banner" small-screen="destroy" />
        <div class="mods">
          <SearchResult
            v-for="result in mods"
            :id="result.slug || result.id"
            :key="result.id"
            :name="result.title"
            :description="result.description"
            :created-at="result.published"
            :updated-at="result.updated"
            :downloads="result.downloads.toString()"
            :icon-url="result.icon_url"
            :author-url="result.author_url"
            :categories="result.categories"
            :is-modrinth="true"
          />
        </div>
        <m-footer class="footer" hide-big centered />
      </div>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import SearchResult from '~/components/ui/ProjectCard'
import MFooter from '~/components/layout/MFooter'

import ReportIcon from '~/assets/images/utils/report.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import Advertisement from '~/components/ads/Advertisement'

export default {
  auth: false,
  components: {
    Advertisement,
    SearchResult,
    CalendarIcon,
    DownloadIcon,
    MFooter,
    ReportIcon,
  },
  async asyncData(data) {
    try {
      let res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${data.params.id}`
      )
      const user = res.data

      let mods = []
      res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${user.id}/mods`
      )
      if (res.data) {
        res = await axios.get(
          `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`
        )
        mods = res.data
      }

      return {
        mods,
        user,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'User not found',
      })
    }
  },
  methods: {
    formatNumber(x) {
      return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    },
    sumDownloads() {
      let sum = 0

      for (const mod of this.mods) {
        sum += mod.downloads
      }

      return this.formatNumber(sum)
    },
  },
  head() {
    return {
      title: this.user.username + ' - Modrinth',
      meta: [
        {
          hid: 'og:type',
          name: 'og:type',
          content: 'website',
        },
        {
          hid: 'og:title',
          name: 'og:title',
          content: this.user.username,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: this.user.username,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.user.bio,
        },
        {
          hid: 'description',
          name: 'description',
          content:
            this.user.bio +
            ' - View minecraft mods on Modrinth today! Modrinth is a new and modern Minecraft modding platform that is compatible with CurseForge too!',
        },
        {
          hid: 'og:url',
          name: 'og:url',
          content: `https://modrinth.com/user/${this.user.id}`,
        },
        {
          hid: 'og:image',
          name: 'og:image',
          content:
            this.user.avatar_url || 'https://cdn.modrinth.com/placeholder.png',
        },
      ],
    }
  },
}
</script>

<style lang="scss" scoped>
.sidebar-l {
  @media screen and (min-width: 1024px) {
    min-width: 21rem;
  }

  .user-info {
    @extend %row;
    img {
      width: 6rem;
      height: 6rem;
      margin-right: var(--spacing-card-md);
      border-radius: var(--size-rounded-icon);
    }
    .text {
      h2 {
        margin: 0;
        color: var(--color-text-dark);
        font-size: var(--font-size-lg);
      }
      .badge {
        display: inline-block;
      }
    }
  }
  .buttons {
    @extend %column;

    .iconified-button {
      max-width: 4.5rem;
    }
  }
  .stats {
    display: flex;
    flex-wrap: wrap;
    .stat {
      @extend %stat;

      svg {
        padding: 0.25rem;
        border-radius: 50%;
        background-color: var(--color-button-bg);
      }
    }
  }
}
</style>
