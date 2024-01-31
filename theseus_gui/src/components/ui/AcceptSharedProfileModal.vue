<script setup>
import { Modal, Button } from 'omorphia'
import { ref } from 'vue'
import { useFetch } from '@/helpers/fetch.js'
import SearchCard from '@/components/ui/SearchCard.vue'
import { handleError } from '@/store/notifications.js'
import { share_accept, share_install } from '@/helpers/shared_profiles.js'

const confirmModal = ref(null)
const linkId = ref(null)
const sharedProfile = ref(null)

defineExpose({
  async show(event) {
    linkId.value = event.id
    sharedProfile.value = await useFetch(
      `https://staging-api.modrinth.com/_internal/share/${encodeURIComponent(event.id)}`,
      'shared profile'
    )

    confirmModal.value.show()
  },
})

async function install() {
  confirmModal.value.hide()
  await share_accept(linkId.value).catch(handleError)
  await share_install(sharedProfile.value.id).catch(handleError)
}
</script>

<template>
  <Modal ref="confirmModal" :header="`Install ${project?.title}`">
    <div class="modal-body">
      <SearchCard
        :project="project"
        class="project-card"
        :categories="categories"
        @open="confirmModal.hide()"
      />
      <div class="button-row">
        <div class="markdown-body">
          <p>
            Installing <code>{{ sharedProfile.id }}</code> from user {{ sharedProfile.owner_id }}
          </p>
        </div>
        <div class="button-group">
          <Button color="primary" @click="install">Install</Button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.modal-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--gap-md);
  padding: var(--gap-lg);
}

.button-row {
  width: 100%;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: var(--gap-md);
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: var(--gap-sm);
}

.project-card {
  background-color: var(--color-bg);
  width: 100%;

  :deep(.badge) {
    border: 1px solid var(--color-raised-bg);
    background-color: var(--color-accent-contrast);
  }
}
</style>
