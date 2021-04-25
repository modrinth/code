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
        <a target="_blank" href="https://github.com/modrinth">open source</a>,
        while giving back to the modding community as much as possible.
      </p>

      <p>
        While we still are in beta, we hope we can be a major modding platform
        for all modders.
      </p>
      <h2>How does Modrinth work?</h2>
      <p>
        Modrinth is not just a website: it is made of several different
        components that all come together to make a fast and flexible modding
        platform.
      </p>

      <p>
        On the technical level, Modrinth is made up of two main components: the
        Rust-based backend named
        <a target="_blank" href="https://github.com/modrinth/labrinth"
          >Labrinth</a
        >, and the Vue-based frontend named
        <a target="_blank" href="https://github.com/modrinth/knossos">Knossos</a
        >.
      </p>

      <p>
        Additionally, some other custom-created resources exist, including but
        not limited to:
        <a target="_blank" href="https://github.com/modrinth/minotaur"
          >Minotaur</a
        >, a Gradle plugin for easily publishing mods to Modrinth, and
        <a target="_blank" href="https://github.com/modrinth/minos">Minos</a>,
        an authentication provider. All of Modrinth's code can be found on
        <a target="_blank" href="https://github.com/modrinth">our GitHub page</a
        >.
      </p>
      <h2>Backend Documentation</h2>
      <p>
        Documentation for the Modrinth API (Labrinth) can be found on the GitHub
        repository's wiki
        <a
          target="_blank"
          href="https://github.com/modrinth/labrinth/wiki/API-Documentation"
          >here</a
        >.
      </p>
    </div>
    <m-footer class="footer" centered />
  </div>
</template>

<script>
import MFooter from '~/components/layout/MFooter'

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

a {
  text-decoration: underline;
}
</style>
