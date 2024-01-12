<template>
  <div>
    <ChartDisplay :projects="projects ?? undefined" :personal="true" />
  </div>
</template>

<script setup>
import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

definePageMeta({
  middleware: 'auth',
})

useHead({
  title: 'Analytics - Modrinth',
})

const auth = await useAuth()
const id = auth.value?.user?.id

const { data: projects } = await useAsyncData(`user/${id}/projects`, () =>
  useBaseFetch(`user/${id}/projects`)
)
</script>
