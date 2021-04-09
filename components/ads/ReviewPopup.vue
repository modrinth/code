<template>
  <Popup :show-popup="shown"> </Popup>
</template>

<script>
export default {
  name: 'ReviewPopup',
  data() {
    return {
      shown: false,
    }
  },
  methods: {
    show() {
      this.shown = true
    },
    hide() {
      this.shown = false
    },
  },
}
</script>

<style scoped lang="scss"></style>
