<template>
  <span v-if="to === undefined" v-bind="$attrs">
    <slot />
  </span>
  <router-link v-else-if="link.type === 'router'" :to="link.destination" v-bind="$attrs" class="has-link has-router-link" @click="link.callback">
    <slot />
  </router-link>
  <a v-else-if="link.type === 'external'" :href="link.destination" v-bind="$attrs" class="has-link has-external-link" @click="link.callback">
    <slot />
  </a>
</template>

<script setup lang="ts">
import { computed, type Ref } from 'vue'
import {
  asLink,
  type Link,
  type Linkish
} from '../../utils/link'

const props = defineProps<{
  to?: Linkish
}>()

defineOptions({
  inheritAttrs: false,
})

const link: Ref<Link> = computed(() => {
  const linkTest = asLink(props.to as Link)
  console.log(linkTest)
  return linkTest
})
</script>
