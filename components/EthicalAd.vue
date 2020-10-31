<template>
  <div
    :class="$colorMode.value"
    data-ea-publisher="modrinth-com"
    :data-ea-type="type"
    data-ea-manual="true"
  />
</template>

<script>
export default {
  name: 'EthicalAd',
  props: {
    type: {
      type: String,
      default: 'text',
    },
  },
  mounted() {
    try {
      // eslint-disable-next-line no-undef
      ethicalads.load()
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error(err)
    }
  },
}
</script>

<style scoped>
[data-ea-type='image'] {
  margin: auto 10px;
}
</style>
