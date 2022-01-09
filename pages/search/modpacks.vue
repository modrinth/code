<template>
  <div></div>
</template>

<script>
export default {
  name: 'Modpacks',
  asyncData(ctx) {
    ctx.params.projectType = 'modpack'
  },
  head: {
    title: 'Modpacks - Modrinth',
    meta: [
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Modpacks',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Modpacks',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/modpacks`,
      },
    ],
  },
}
</script>

<style lang="scss" scoped></style>
