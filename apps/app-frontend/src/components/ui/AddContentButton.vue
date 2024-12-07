<script setup lang="ts">
import { DropdownIcon, PlusIcon, FolderOpenIcon } from '@modrinth/assets'
import { ButtonStyled, OverflowMenu } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { add_project_from_path } from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'
import { useRouter } from 'vue-router'

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

const router = useRouter()

const handleAddContentFromFile = async () => {
  const newProject = await open({ multiple: true })
  if (!newProject) return

  for (const project of newProject) {
    await add_project_from_path(props.instance.path, project.path ?? project).catch(handleError)
  }
}

const handleSearchContent = async () => {
  await router.push({
    path: `/browse/${props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'}`,
    query: { i: props.instance.path },
  })
}
</script>

<template>
  <div class="joined-buttons">
    <ButtonStyled>
      <button @click="handleSearchContent">
        <PlusIcon />
        Install content
      </button>
    </ButtonStyled>
    <ButtonStyled>
      <OverflowMenu
        :options="[
          {
            id: 'from_file',
            action: handleAddContentFromFile,
          },
        ]"
      >
        <DropdownIcon />
        <template #from_file>
          <FolderOpenIcon />
          <span class="no-wrap"> Add from file </span>
        </template>
      </OverflowMenu>
    </ButtonStyled>
  </div>
</template>
