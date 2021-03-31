<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div class="card page-nav">
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
            Followed Mods
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
        </div>
        <m-footer class="footer" />
      </div>
      <div class="content">
        <NuxtChild />
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

export default {
  name: 'DashboardPage',
  components: {
    ModIcon,
    ModerationIcon,
    SettingsIcon,
    NotificationsIcon,
    FollowIcon,
  },
}
</script>

<style lang="scss" scoped>
.footer {
  padding-top: 0;
}
</style>
