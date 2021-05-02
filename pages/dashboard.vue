<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div v-if="$auth.user != null" class="card page-nav">
          <nuxt-link :to="'/dashboard/projects'" class="tab last">
            <ModIcon />
            My mods
          </nuxt-link>
          <nuxt-link :to="'/dashboard/notifications'" class="tab last">
            <NotificationsIcon />
            Notifications
          </nuxt-link>
          <nuxt-link :to="'/dashboard/follows'" class="tab last">
            <FollowIcon />
            Followed mods
          </nuxt-link>
          <nuxt-link
            v-if="
              $auth.user.role === 'admin' || $auth.user.role === 'moderator'
            "
            :to="'/dashboard/moderation'"
            class="tab last"
          >
            <ModerationIcon />
            Moderation
          </nuxt-link>
          <nuxt-link :to="'/dashboard/settings'" class="tab last">
            <SettingsIcon />
            Settings
          </nuxt-link>
          <nuxt-link :to="'/dashboard/privacy'" class="tab last">
            <ShieldIcon />
            Privacy Settings
          </nuxt-link>
        </div>
        <div v-else class="card page-nav">
          <a :href="authUrl" class="tab last">
            <UserIcon />
            Log in
          </a>
          <nuxt-link :to="'/dashboard/privacy'" class="tab last">
            <SettingsIcon />
            Privacy Settings
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
      return `https://api.modrinth.com/api/v1/auth/init?url=https://modrinth.com${this.$route.fullPath}`
    },
  },
}
</script>

<style lang="scss" scoped>
.hideSmall {
  padding-top: 0;
}
</style>
