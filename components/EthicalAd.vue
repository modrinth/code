<template>
  <div
    class="ethical-ad"
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

      const element = document.getElementsByClassName('ethical-ad')
      element[0].className = 'ethical-ad loaded ' + this.$colorMode.preference
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error(err)
    }
  },
}
</script>

<style scoped>
[data-ea-type='text'] {
  min-height: 70px;
}
[data-ea-type='image'] {
  margin: auto 10px;
  min-height: 260px;
}
</style>
