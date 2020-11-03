<template>
  <div class="main">
    <h1>About</h1>
    <p>
      Founded in 2020, Modrinth was created to provide modders with an open and
      intuitive platform to publish their mods on.
    </p>

    <p>
      Our primary goal is to be as open as possible, with all our code being
      Open Source, while giving back to the modding community as much as
      possible.
    </p>

    <p>
      While we still are in early alpha, we hope we can soon be a major modding
      platform for all modders :)
    </p>
  </div>
</template>

<script>
export default {
  auth: false,
  layout: 'home',
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
  margin: 0 auto;
  max-width: 800px;
}
</style>
