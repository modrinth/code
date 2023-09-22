<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <h1>Moderation</h1>
        <NavStack>
          <NavStackItem link="/moderation" label="Overview">
            <ModrinthIcon />
          </NavStackItem>
          <NavStackItem link="/moderation/review" label="Review projects">
            <ModerationIcon />
          </NavStackItem>
          <NavStackItem link="/moderation/messages" label="Messages">
            <MessageIcon />
          </NavStackItem>
          <NavStackItem link="/moderation/reports" label="Reports">
            <ReportIcon />
          </NavStackItem>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <NuxtPage />
    </div>
  </div>
</template>
<script setup>
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

import ModrinthIcon from '~/assets/images/utils/modrinth.svg'
import ModerationIcon from '~/assets/images/sidebar/admin.svg'
import ReportIcon from '~/assets/images/utils/report.svg'
import MessageIcon from '~/assets/images/utils/message.svg'

definePageMeta({
  middleware: 'auth',
})
</script>
