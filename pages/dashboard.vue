<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div v-if="$auth.user != null" class="card page-nav">
          <nuxt-link :to="'/dashboard/projects'" class="tab">
            <ModIcon />
            My mods
          </nuxt-link>
          <nuxt-link :to="'/dashboard/notifications'" class="tab">
            <NotificationsIcon />
            Notifications
            <div v-if="this.$user.notifications.count > 0" class="notif-count">
              {{ this.$user.notifications.count }}
            </div>
          </nuxt-link>
          <nuxt-link :to="'/dashboard/follows'" class="tab">
            <FollowIcon />
            Followed mods
          </nuxt-link>
          <nuxt-link
            v-if="
              $auth.user.role === 'admin' || $auth.user.role === 'moderator'
            "
            :to="'/dashboard/moderation'"
            class="tab"
          >
            <ModerationIcon />
            Moderation
          </nuxt-link>
          <nuxt-link :to="'/dashboard/settings'" class="tab">
            <SettingsIcon />
            Settings
          </nuxt-link>
          <nuxt-link :to="'/dashboard/privacy'" class="tab">
            <ShieldIcon />
            Privacy settings
          </nuxt-link>
        </div>
        <div v-else class="card page-nav">
          <a :href="authUrl" class="tab">
            <UserIcon />
            Log in
          </a>
          <nuxt-link :to="'/dashboard/privacy'" class="tab">
            <SettingsIcon />
            Privacy settings
          </nuxt-link>
        </div>
        <m-footer class="footer" hide-small />
      </div>
      <div class="content">
        <NuxtChild />
        <m-footer class="footer" hide-big centered />
      </div>
    </div>
  </div>
</template>
<script>
import ModIcon from '~/assets/images/sidebar/mod.svg?inline'
import ModerationIcon from '~/assets/images/sidebar/admin.svg?inline'
import SettingsIcon from '~/assets/images/sidebar/settings.svg?inline'
import NotificationsIcon from '~/assets/images/sidebar/notifications.svg?inline'
import FollowIcon from '~/assets/images/utils/heart.svg?inline'
import UserIcon from '~/assets/images/utils/user.svg?inline'
import ShieldIcon from '~/assets/images/utils/shield.svg?inline'

import MFooter from '~/components/layout/MFooter'

export default {
  name: 'DashboardPage',
  components: {
    ModIcon,
    ModerationIcon,
    SettingsIcon,
    NotificationsIcon,
    FollowIcon,
    UserIcon,
    ShieldIcon,
    MFooter,
  },
  computed: {
    authUrl() {
      return `${this.$axios.defaults.baseURL}auth/init?url=${this.$store.app.$config.utils.domain}${this.$route.fullPath}`
    },
  },
}
</script>

<style lang="scss" scoped>
.hideSmall {
  padding-top: 0;
}

.notif-count {
  display: flex;
  justify-content: center;
  align-items: center;

  background-color: rgba(180, 180, 180, 0.4);
  border-radius: 2rem;
  padding: 0.1rem 0.35rem;
  margin: 0 0.2rem 0 auto;

  font-size: 0.9rem;
}
</style>
