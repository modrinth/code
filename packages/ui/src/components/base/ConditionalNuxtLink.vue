<template>
  <router-link v-if="isLink" :to="to">
    <slot />
  </router-link>
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
