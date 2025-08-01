<template>
  <nuxt-link v-if="isLink" :to="to">
    <slot />
  </nuxt-link>
  <span v-else>
    <slot />
  </span>
</template>

<script setup>
defineProps({
  to: {
    type: String,
    required: true,
  },
  isLink: {
    type: Boolean,
    required: true,
  },
})
</script>
