<template>
  <div class="main">
    <div class="container">
      <h1>About</h1>
      <p>
        Founded in 2020, Modrinth was created to provide modders with an open
        and intuitive platform to publish their mods on.
      </p>

      <p>
        Our primary goal is to be as open as possible, with all our code being
        Open Source, while giving back to the modding community as much as
        possible.
      </p>

      <p>
        While we still are in beta, we hope we can be a major modding platform
        for all modders.
      </p>
    </div>
    <m-footer class="footer" centered />
  </div>
</template>

<script>
import MFooter from '@/components/MFooter'

export default {
  components: {
    MFooter,
  },
  auth: false,
  head: {
    title: 'About - Modrinth',
    meta: [
      {
        hid: 'description',
        name: 'description',
        content:
          'View information about Modrinth, an open source modding platform here! Modrinth currently supports Minecraft, including the forge and fabric mod loaders.',
      },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'About',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'About',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/about`,
      },
    ],
  },
}
</script>

<style lang="scss" scoped>
.main {
  margin: var(--spacing-card-sm) auto;
  max-width: 800px;
}

.container {
  @extend %card;
  padding: var(--spacing-card-sm) var(--spacing-card-lg);
}
</style>
