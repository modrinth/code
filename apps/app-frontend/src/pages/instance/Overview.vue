<template>
  {{ instance.name}} overview
</template>
<script setup lang="ts">

import type { GameInstance } from '@/helpers/types'

defineProps< {
  instance: GameInstance,
  offline: boolean,
}>()
</script>
