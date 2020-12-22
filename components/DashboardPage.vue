<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div class="card page-nav">
          <nuxt-link :to="'/dashboard/projects'" class="tab last">
            <ModIcon />
            My mods
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
        <client-only>
          <EthicalAd type="image" ad-id="dashboard" />
        </client-only>
        <m-footer class="footer" />
      </div>
      <div class="content">
        <slot />
      </div>
    </div>
  </div>
</template>
<script>
import ModIcon from '~/assets/images/sidebar/mod.svg?inline'
import ModerationIcon from '~/assets/images/sidebar/admin.svg?inline'
import SettingsIcon from '~/assets/images/sidebar/settings.svg?inline'

export default {
  name: 'DashboardPage',
  components: {
    ModIcon,
    ModerationIcon,
    SettingsIcon,
  },
}
</script>

<style lang="scss" scoped>
.section-header {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);
  h3 {
    margin: auto 0;
    color: var(--color-text-dark);
    font-weight: var(--font-weight-extrabold);
  }
}

.footer {
  padding-top: 0;
}
</style>
