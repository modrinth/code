<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="content">
        <h1 v-if="$auth.user">Settings for {{ $auth.user.username }}</h1>
        <h1 v-else>Settings</h1>
        <div class="card styled-tabs">
          <nuxt-link v-if="$auth.user" class="tab" to="/settings" exact
            ><span>General</span></nuxt-link
          >
          <nuxt-link v-if="$auth.user" class="tab" to="/settings/follows">
            <span>Followed projects</span>
          </nuxt-link>
          <nuxt-link v-if="$auth.user" class="tab" to="/settings/security">
            <span>Security</span>
          </nuxt-link>
        </div>
        <NuxtChild />
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Settings',
}
</script>

<style lang="scss" scoped>
.page-contents {
  display: flex;
  flex-direction: column;

  h1 {
    color: var(--color-text-dark);
    margin: 0 0 0.5rem 1.5rem;
  }
}

@media screen and (min-width: 1024px) {
  .page-contents {
    max-width: 60rem !important;
  }
}
</style>
