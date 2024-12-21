<script setup>
import { Button } from '@modrinth/ui'
import { ref } from 'vue'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get_categories } from '@/helpers/tags.js'
import { handleError } from '@/store/notifications.js'
import { get_version, get_project } from '@/helpers/cache.js'
import { install as installVersion } from '@/store/install.js'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const confirmModal = ref(null)
const project = ref(null)
const version = ref(null)
const categories = ref(null)
const installing = ref(false)

defineExpose({
  async show(event) {
    if (event.event === 'InstallVersion') {
      version.value = await get_version(event.id, 'must_revalidate').catch(handleError)
      project.value = await get_project(version.value.project_id, 'must_revalidate').catch(
        handleError,
      )
    } else {
      project.value = await get_project(event.id, 'must_revalidate').catch(handleError)
      version.value = await get_version(
        project.value.versions[project.value.versions.length - 1],
        'must_revalidate',
      ).catch(handleError)
    }
    categories.value = (await get_categories().catch(handleError)).filter(
      (cat) => project.value.categories.includes(cat.name) && cat.project_type === 'mod',
    )
    confirmModal.value.show()
  },
})

async function install() {
  confirmModal.value.hide()
  await installVersion(project.value.id, version.value.id, null, 'URLConfirmModal')
}
</script>

<template>
  <ModalWrapper ref="confirmModal" :header="`Install ${project?.title}`">
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
            Installing <code>{{ version.id }}</code> from Modrinth
          </p>
        </div>
        <div class="button-group">
          <Button :loading="installing" color="primary" @click="install">Install</Button>
        </div>
      </div>
    </div>
  </ModalWrapper>
</template>

<style scoped lang="scss">
.modal-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--gap-md);
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
