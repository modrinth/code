<template>
  <div class="ad-wrapper">
    <adsbygoogle
      ad-slot="7510690716"
      :ad-format="format"
      :page-url="pageUrl ? pageUrl : undefined"
    />
  </div>
</template>

<script>
export default {
  name: 'Advertisement',
  props: {
    format: {
      type: String,
      default: 'horizontal',
    },
    pageUrl: {
      type: String,
      required: false,
      default: '',
    },
  },
}
</script>

<style lang="scss" scoped>
.ad-wrapper {
  width: 100%;
  @extend %card;
  margin-bottom: var(--spacing-card-md);
}
</style>
