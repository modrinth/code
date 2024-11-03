<script setup lang="ts">
import { DropdownIcon, FolderOpenIcon, SearchIcon } from '@modrinth/assets'
import { Button, OverflowMenu } from '@modrinth/ui'
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
    path: `/browse/${props.instance.loader === 'vanilla' ? 'datapack' : 'mod'}`,
    query: { i: props.instance.path },
  })
}
</script>

<template>
  <div class="joined-buttons">
    <Button color="primary" @click="handleSearchContent"><SearchIcon /> Add content </Button>

    <OverflowMenu
      :options="[
        {
          id: 'search',
          action: handleSearchContent,
        },
        {
          id: 'from_file',
          action: handleAddContentFromFile,
        },
      ]"
      class="btn btn-primary btn-dropdown-animation icon-only"
    >
      <DropdownIcon />
      <template #search>
        <SearchIcon />
        <span class="no-wrap"> Search </span>
      </template>
      <template #from_file>
        <FolderOpenIcon />
        <span class="no-wrap"> Add from file </span>
      </template>
    </OverflowMenu>
  </div>
</template>
