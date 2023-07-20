<template>
  <div>
    <section class="universal-card">
      <h2>Messages</h2>

      <ThreadSummary
        v-for="thread in inbox"
        :key="thread.id"
        :thread="thread"
        :link="getLink(thread)"
        :auth="auth"
      />
    </section>
  </div>
</template>
<script setup>
import ThreadSummary from '~/components/ui/thread/ThreadSummary.vue'

useHead({
  title: 'Moderation inbox - Modrinth',
})

const auth = await useAuth()
const { data: inbox } = await useAsyncData('thread/inbox', () => useBaseFetch('thread/inbox'))

function getLink(thread) {
  if (thread.report_id) {
    return `/moderation/report/${thread.report_id}`
  } else if (thread.project_id) {
    return `/project/${thread.project_id}/moderation`
  }
  return null
}
</script>
<style lang="scss" scoped>
.thread-summary:not(:last-child) {
  margin-bottom: var(--spacing-card-md);
}
</style>
