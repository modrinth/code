<template>
  <div class="edit-page">
    <div class="left-side">
      <div class="profile-picture card">
        <h3>Profile picture</h3>
        <div class="uploader">
          <img
            :src="previewImage ? previewImage : $auth.user.avatar_url"
            @click="developerMode++"
          />
          <file-input
            accept="image/png,image/jpeg,image/gif,image/webp"
            class="choose-image"
            prompt="Choose image or drag it here"
            @change="showPreviewImage"
          />
        </div>
        <button
          class="iconified-button"
          @click="
            icon = null
            previewImage = null
          "
        >
          <TrashIcon />
          Reset
        </button>
      </div>
      <div class="recap card">
        <section>
          <h2>Profile Recap</h2>
          <div>
            <Badge
              v-if="$auth.user.role === 'admin'"
              type="Admin"
              color="red"
            />
            <Badge
              v-else-if="$auth.user.role === 'moderator'"
              type="Moderator"
              color="yellow"
            />
            <Badge v-else type="Developer" color="green" />
            <div class="stat">
              <SunriseIcon />
              <span>Joined {{ $dayjs($auth.user.created).fromNow() }}</span>
            </div>
          </div>
        </section>
        <section>
          <div class="stat">
            <DownloadIcon />
            <span>
              <strong>{{ sumDownloads() }}</strong> downloads
            </span>
          </div>
          <div class="stat">
            <HeartIcon />
            <span>
              <strong>{{ sumFollows() }}</strong> followers of projects
            </span>
          </div>
        </section>
      </div>
    </div>
    <div class="right-side">
      <section class="card">
        <h3>Username</h3>
        <label>
          <span>
            The username used on Modrinth to identify yourself. This must be
            unique.
          </span>
          <input
            v-model="username"
            type="text"
            placeholder="Enter your username"
          />
        </label>
        <h3>Email</h3>
        <label>
          <span>
            The email for your account. This is private information which is not
            exposed in any API routes or on your profile. It is also optional.
          </span>
          <input v-model="email" type="email" placeholder="Enter your email" />
        </label>
        <h3>Bio</h3>
        <label>
          <span>
            A description of yourself which other users can see on your profile.
          </span>
          <input v-model="bio" type="text" placeholder="Enter your bio" />
        </label>
        <h3>Theme</h3>
        <label>
          <span>
            Change the global site theme. It can also be changed between light
            and dark in the navigation bar.
          </span>
          <Multiselect
            v-model="$colorMode.preference"
            :options="['system', 'light', 'dark', 'oled']"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
          />
        </label>
        <h3>Search sidebar on right side</h3>
        <label>
          <span>
            Sets the sidebar direction for search pages. Enabling this will put
            the search bar on the right side.
          </span>
          <input
            v-model="searchLayout"
            class="switch stylized-toggle"
            type="checkbox"
            @change="changeLayout"
          />
        </label>
        <h3>Project sidebar on right side</h3>
        <label>
          <span>
            Sets the sidebar direction for project pages. Enabling this will
            make projects look closer to the legacy layout, with project
            information on the right side.
          </span>
          <input
            v-model="projectLayout"
            class="switch stylized-toggle"
            type="checkbox"
            @change="changeLayout"
          />
        </label>
        <section v-if="developerMode > 6">
          <h3>Developer options</h3>
          <label>
            <span>
              Set the API endpoint. This value is not stored, and is intended
              for temporary usage.</span
            >
            <Multiselect
              v-model="apiEndpoint"
              :options="['production', 'staging']"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
              @input="changeApiEndpoint()"
            />
          </label>
        </section>
      </section>
    </div>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import FileInput from '~/components/ui/FileInput'
import Badge from '~/components/ui/Badge'

import HeartIcon from '~/assets/images/utils/heart.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import SunriseIcon from '~/assets/images/utils/sunrise.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'

export default {
  components: {
    TrashIcon,
    SunriseIcon,
    DownloadIcon,
    HeartIcon,
    Badge,
    FileInput,
    Multiselect,
  },
  asyncData(ctx) {
    return {
      username: ctx.$auth.user.username,
      email: ctx.$auth.user.email,
      bio: ctx.$auth.user.bio,
    }
  },
  data() {
    return {
      icon: null,
      previewImage: null,
      searchLayout: false,
      projectLayout: false,
      apiEndpoint: this.getApiEndpoint(),
      developerMode: 0,
    }
  },
  fetch() {
    this.searchLayout = this.$store.state.cosmetics.searchLayout
    this.projectLayout = this.$store.state.cosmetics.projectLayout

    this.$emit('update:action-button', 'Save')
    this.$emit('update:action-button-callback', this.saveChanges)
  },
  head: {
    title: 'Settings - Modrinth',
  },
  created() {
    this.$emit('update:action-button', 'Save')
    this.$emit('update:action-button-callback', this.saveChanges)
  },
  methods: {
    changeTheme() {
      const shift = event.shiftKey
      switch (this.$colorMode.preference) {
        case 'dark':
          this.$colorMode.preference = shift ? 'light' : 'oled'
          break
        case 'oled':
          this.$colorMode.preference = shift ? 'dark' : 'light'
          break
        default:
          this.$colorMode.preference = shift ? 'oled' : 'dark'
      }
    },
    changeApiEndpoint() {
      const subdomain =
        this.apiEndpoint === 'production' ? 'api' : 'staging-api'
      this.$axios.defaults.baseURL =
        'https://' + subdomain + '.modrinth.com/v2/'
    },
    getApiEndpoint() {
      return this.$axios.defaults.baseURL === 'https://api.modrinth.com/v2/'
        ? 'production'
        : 'staging'
    },
    showPreviewImage(files) {
      const reader = new FileReader()
      this.icon = files[0]
      reader.readAsDataURL(this.icon)

      reader.onload = (event) => {
        this.previewImage = event.target.result
      }
    },
    sumDownloads() {
      let sum = 0

      for (const projects of this.$user.projects) {
        sum += projects.downloads
      }

      return this.$formatNumber(sum)
    },
    sumFollows() {
      let sum = 0

      for (const projects of this.$user.projects) {
        sum += projects.followers
      }

      return this.$formatNumber(sum)
    },
    async changeLayout() {
      await this.$store.dispatch('cosmetics/save', {
        searchLayout: this.searchLayout,
        projectLayout: this.projectLayout,
        $cookies: this.$cookies,
      })
    },
    async saveChanges() {
      this.$nuxt.$loading.start()
      try {
        if (this.icon) {
          await this.$axios.patch(
            `user/${this.$auth.user.id}/icon?ext=${
              this.icon.type.split('/')[this.icon.type.split('/').length - 1]
            }`,
            this.icon,
            this.$auth.headers
          )
        }

        const data = {
          email: this.email,
          bio: this.bio,
        }

        if (this.username !== this.$auth.user.username) {
          data.username = this.username
        }
        await this.$axios.patch(
          `user/${this.$auth.user.id}`,
          data,
          this.$auth.headers
        )

        await this.$store.dispatch('auth/fetchUser', {
          token: this.$auth.token,
        })
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
.edit-page {
  display: flex;
  flex-direction: column;

  @media screen and (min-width: 1024px) {
    flex-direction: row;

    .left-side {
      margin-right: var(--spacing-card-bg);
    }
  }
}

.left-side {
  min-width: 20rem;

  .profile-picture {
    h3 {
      font-size: var(--font-size-lg);
    }

    .uploader {
      margin: 1rem 0;
      text-align: center;

      img {
        box-shadow: var(--shadow-card);

        border-radius: var(--size-rounded-md);
        width: 8rem;
        margin-bottom: 0.5rem;
      }
    }
  }

  .recap {
    section {
      h2 {
        font-size: var(--font-size-lg);
        margin: 0 0 0.5rem 0;
      }

      .version-badge {
        text-transform: none;
        margin-bottom: 0.25rem;

        &::first-letter {
          text-transform: uppercase;
        }
      }
    }
  }
}

.stat {
  display: flex;
  align-items: center;
  margin: 0.5rem 0;

  svg {
    width: auto;
    height: 1.25rem;

    margin-right: 0.25rem;
  }

  span {
    strong {
      font-weight: bolder;
    }
  }
}
</style>
