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
        <adsbygoogle ad-slot="7510690716" ad-format="rectangle" />
        <m-footer class="footer" />
      </div>
      <div class="content">
        <adsbygoogle ad-slot="7510690716" ad-format="horizontal" />
        <div class="mods">
          <SearchResult
            v-for="result in mods"
            :id="result.slug ? result.slug : result.id"
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
      </div>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import SearchResult from '@/components/ProjectCard'
import MFooter from '@/components/MFooter'

import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'

export default {
  auth: false,
  components: {
    SearchResult,
    CalendarIcon,
    DownloadIcon,
    MFooter,
  },
  async asyncData(data) {
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    try {
      let res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${data.params.id}`,
        config
      )
      const user = res.data

      let mods = []
      res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${data.params.id}/mods`,
        config
      )
      if (res.data) {
        res = await axios.get(
          `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(
            res.data
          )}`,
          config
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
    }
  },
}
</script>

<style lang="scss" scoped>
.sidebar-l {
  min-width: 21rem;

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

.mods {
}
</style>
