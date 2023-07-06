<script setup>
import { Button, Checkbox, Modal, SendIcon, XIcon } from 'omorphia'
import { PackageIcon, VersionIcon } from '@/assets/icons'
import { ref } from 'vue'
import { export_profile_mrpack, get_potential_override_folders } from '@/helpers/profile.js'
import { open } from '@tauri-apps/api/dialog'
import { handleError } from '@/store/notifications.js'

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

defineExpose({
  show: () => {
    exportModal.value.show()
    initFiles()
  },
})

const exportModal = ref(null)
const nameInput = ref(props.instance.metadata.name)
const versionInput = ref('1.0.0')
const files = ref([])
const folders = ref([])

const initFiles = async () => {
  const newFolders = new Map()
  files.value = []
  await get_potential_override_folders(props.instance.path).then((filePaths) =>
    filePaths
      .map((folder) => ({
        path: folder,
        name: folder.split('/').pop(),
        selected: false,
      }))
      .forEach((pathData) => {
        const parent = pathData.path.split('/').slice(0, -1).join('/')
        if (parent !== '') {
          if (newFolders.has(parent)) {
            newFolders.get(parent).push(pathData)
          } else {
            newFolders.set(parent, [pathData])
          }
        } else {
          files.value.push(pathData)
        }
      })
  )
  folders.value = [...newFolders.entries()].map(([name, value]) => [
    {
      name,
      showingMore: false,
    },
    value,
  ])
}

await initFiles()

const exportPack = async () => {
  const filesToExport = files.value.filter((file) => file.selected).map((file) => file.path)
  folders.value.forEach((args) => {
    args[1].forEach((child) => {
      if (child.selected) {
        filesToExport.push(child.path)
      }
    })
  })
  const outputPath = await open({
    directory: true,
    multiple: false,
  })

  if (outputPath) {
    export_profile_mrpack(
      props.instance.path,
      outputPath + `/${nameInput.value} ${versionInput.value}.mrpack`,
      filesToExport,
      versionInput.value
    ).catch((err) => handleError(err))
    exportModal.value.hide()
  }
}
</script>

<template>
  <Modal ref="exportModal" header="Export modpack">
    <div class="modal-body">
      <div class="labeled_input">
        <p>Modpack Name</p>
        <div class="iconified-input">
          <PackageIcon />
          <input v-model="nameInput" type="text" placeholder="Modpack name" class="input" />
          <Button @click="nameInput = ''">
            <XIcon />
          </Button>
        </div>
      </div>
      <div class="labeled_input">
        <p>Version number</p>
        <div class="iconified-input">
          <VersionIcon />
          <input v-model="versionInput" type="text" placeholder="1.0.0" class="input" />
          <Button @click="versionInput = ''">
            <XIcon />
          </Button>
        </div>
      </div>
      <div class="table">
        <div class="table-head">
          <div class="table-cell">Select files as overrides</div>
        </div>
        <div class="table-content">
          <div v-for="[path, children] of folders" :key="path.name" class="table-row">
            <div class="table-cell file-entry">
              <div class="file-primary">
                <Checkbox
                  :model-value="children.every((child) => child.selected)"
                  :label="path.name"
                  class="select-checkbox"
                  @update:model-value="
                    (newValue) => children.forEach((child) => (child.selected = newValue))
                  "
                />
                <Checkbox
                  v-model="path.showingMore"
                  class="select-checkbox dropdown"
                  collapsing-toggle-style
                />
              </div>
              <div v-if="path.showingMore" class="file-secondary">
                <div v-for="child in children" :key="child.path" class="file-secondary-row">
                  <Checkbox v-model="child.selected" :label="child.name" class="select-checkbox" />
                </div>
              </div>
            </div>
          </div>
          <div v-for="file in files" :key="file.path" class="table-row">
            <div class="table-cell file-entry">
              <div class="file-primary">
                <Checkbox v-model="file.selected" :label="file.name" class="select-checkbox" />
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="button-row push-right">
        <Button @click="exportModal.hide">
          <XIcon />
          Cancel
        </Button>
        <Button disabled>
          <SendIcon />
          Share
        </Button>
        <Button color="primary" @click="exportPack">
          <PackageIcon />
          Export
        </Button>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.modal-body {
  padding: var(--gap-xl);
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
}

.labeled_input {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);

  p {
    margin: 0;
  }
}

.select-checkbox {
  button.checkbox {
    border: none;
  }

  &.dropdown {
    margin-left: auto;
  }
}

.table-content {
  max-height: 18rem;
  overflow-y: auto;
}

.table {
  border: 1px solid var(--color-bg);
}

.file-entry {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
}

.file-primary {
  display: flex;
  align-items: center;
  gap: var(--gap-sm);
}

.file-secondary {
  margin-left: var(--gap-xl);
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  height: 100%;
  vertical-align: center;
}

.file-secondary-row {
  display: flex;
  align-items: center;
  gap: var(--gap-sm);
}

.button-row {
  display: flex;
  gap: var(--gap-sm);
}
</style>
