<template>
  <div class="edit-page">
    <section class="card account-settings">
      <div class="header">
        <h2 class="title">Account settings</h2>
        <div class="controls">
          <button
            class="brand-button-colors iconified-button"
            title="Save account settings changes"
            @click="saveChanges()"
          >
            <SaveIcon />
            Save changes
          </button>
        </div>
      </div>
      <div class="left-side">
        <h3>Profile picture</h3>
        <div class="profile-picture">
          <img :src="previewImage ? previewImage : $auth.user.avatar_url" />
          <div class="uploader">
            <SmartFileInput
              :show-icon="false"
              :max-size="2097152"
              accept="image/png,image/jpeg,image/gif,image/webp"
              class="choose-image"
              prompt="Choose image or drag it here"
              @change="showPreviewImage"
            />
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
        </div>
      </div>
      <div class="right-side">
        <label>
          <span>
            <h3>Username</h3>
            <span>This must be unique.</span>
          </span>

          <input
            v-model="username"
            type="text"
            placeholder="Enter your username"
          />
        </label>
        <label>
          <span>
            <h3>Email (optional)</h3>
            <span>This is kept private.</span>
          </span>
          <input v-model="email" type="email" placeholder="Enter your email" />
        </label>
        <label>
          <span>
            <h3>Bio</h3>
            <span>Describe yourself to other users!</span>
          </span>
          <input v-model="bio" type="text" placeholder="Enter your bio" />
        </label>
      </div>
    </section>
    <section class="card">
      <div class="header">
        <h2 class="title">Display settings</h2>
      </div>
      <label>
        <span>
          <h3>Theme</h3>
          <span>Change the global site theme.</span>
        </span>
        <Multiselect
          v-model="$colorMode.preference"
          :options="['system', 'light', 'dark', 'oled']"
          :custom-label="
            (value) =>
              value === 'oled'
                ? 'OLED'
                : value.charAt(0).toUpperCase() + value.slice(1)
          "
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
        />
      </label>
      <label>
        <span>
          <h3>Search sidebar on the right</h3>
          <span>
            Enabling this will put the search page's filters sidebar on the
            right side.
          </span>
        </span>
        <input
          v-model="searchLayout"
          class="switch stylized-toggle"
          type="checkbox"
          @change="changeLayout"
        />
      </label>
      <label>
        <span>
          <h3>Project sidebar on the right</h3>

          <span>
            Enabling this will put the project pages' info sidebars on the right
            side.
          </span>
        </span>
        <input
          v-model="projectLayout"
          class="switch stylized-toggle"
          type="checkbox"
          @change="changeLayout"
        />
      </label>
    </section>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import SmartFileInput from '~/components/ui/SmartFileInput'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import SaveIcon from '~/assets/images/utils/save.svg?inline'

export default {
  components: {
    TrashIcon,
    SaveIcon,
    SmartFileInput,
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
    }
  },
  fetch() {
    this.searchLayout = this.$store.state.cosmetics.searchLayout
    this.projectLayout = this.$store.state.cosmetics.projectLayout
  },
  head: {
    title: 'Settings - Modrinth',
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
            this.$defaultHeaders()
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
          this.$defaultHeaders()
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
.account-settings {
  display: grid;
  grid-template: 'header header' auto 'left-side left-side' auto 'right-side right-side' auto;

  @media screen and (min-width: 1024px) {
    grid-template:
      'header header' auto
      'left-side right-side' auto;
  }

  .left-side {
    grid-area: left-side;
    min-width: 20rem;

    .profile-picture {
      display: flex;
      flex-direction: row;
      gap: 0.5rem;
      align-items: center;

      img {
        box-shadow: var(--shadow-card);
        border-radius: var(--size-rounded-md);
        width: 10rem;
        height: 10rem;
        object-fit: contain;
        margin-bottom: 0.5rem;
      }

      .uploader {
        text-align: center;
        .iconified-button {
          margin-top: 0.5rem;
        }
      }
    }
  }
  .right-side {
    grid-area: right-side;
    margin-left: var(--spacing-card-lg);
  }
}

.card span {
  margin-bottom: 1rem;
}

label {
  align-items: center;
}

.header {
  display: flex;
  align-items: center;
  padding-bottom: 1rem;
  grid-area: header;

  .title {
    flex-grow: 1;
    margin: 0;
  }

  .controls {
    display: flex;
    flex-direction: row;
    gap: 0.5rem;
  }
}
</style>
