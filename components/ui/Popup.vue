<template>
  <div v-if="showPopup">
    <div class="popup-overlay" />
    <div class="popup-body">
      <slot></slot>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Popup',
  props: {
    showPopup: {
      type: Boolean,
      default: false,
    },
  },
}
</script>

<style lang="scss" scoped>
.popup-overlay {
  top: 0;
  left: 0;
  z-index: 10;
  position: fixed;
  width: 100%;
  height: 100%;
  background-color: var(--color-button-bg);
  border: none;
  opacity: 0.6;
  overflow-x: hidden;
}

.popup-body {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 11;
  box-shadow: 0 2px 3px 1px var(--color-button-bg);
  border-radius: 10px;
  max-height: 80%;
  overflow-y: auto;
  background-color: var(--color-raised-bg);
}
</style>
