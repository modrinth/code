<template>
  <div class="ad-wrapper">
    <div class="ad">
      <GptAd :ad-unit="adUnit" :size="size" />
    </div>
  </div>
</template>

<script>
/* eslint-disable no-undef */
export default {
  name: 'Advertisement',
  props: {
    size: {
      type: String,
      required: true,
    },
    adUnit: {
      type: String,
      required: true,
    },
  },
}
</script>

<style lang="scss" scoped>
.ad-wrapper {
  width: 100%;
  @extend %card;
  display: flex;
  flex-direction: row;
  margin-bottom: var(--spacing-card-md);
  justify-content: center;
}
</style>
