<template>
  <Modal ref="incompatibleModal" header="Incompatibility warning">
    <div class="modal-body">
      <p>
        This {{ versions?.length > 0 ? 'project' : 'version' }} is not compatible with the instance
        you're trying to install it on. Are you sure you want to continue? Dependencies will not be
        installed.
      </p>
      <table>
        <tr class="header">
          <th>{{ instance?.metadata.name }}</th>
          <th>{{ projectTitle }}</th>
        </tr>
        <tr class="content">
          <td class="data">
            {{ instance?.metadata.loader }} {{ instance?.metadata.game_version }}
          </td>
          <td>
            <DropdownSelect
              v-if="versions?.length > 1"
              v-model="selectedVersion"
              :options="versions"
              placeholder="Select version"
              :display-name="
                (version) =>
                  `${version?.name} (${version?.loaders
                    .map((name) => formatCategory(name))
                    .join(', ')} - ${version?.game_versions.join(', ')})`
              "
              render-up
            />
            <span v-else>
              <span>
                {{ selectedVersion?.name }} ({{
                  selectedVersion?.loaders.map((name) => formatCategory(name)).join(', ')
                }}
                - {{ selectedVersion?.game_versions.join(', ') }})
              </span>
            </span>
          </td>
        </tr>
      </table>
      <div class="button-group">
        <Button @click="() => incompatibleModal.hide()"><XIcon />Cancel</Button>
        <Button color="primary" :disabled="installing" @click="install()">
          <DownloadIcon /> {{ installing ? 'Installing' : 'Install' }}
        </Button>
      </div>
    </div>
  </Modal>
</template>

<script setup>
import { Button, Modal, XIcon, DownloadIcon, DropdownSelect, formatCategory } from 'omorphia'
import { add_project_from_version as installMod } from '@/helpers/profile'
import { defineExpose, ref } from 'vue'
import { handleError } from '@/store/state.js'

const instance = ref(null)
const projectTitle = ref(null)
const versions = ref(null)
const selectedVersion = ref(null)
const incompatibleModal = ref(null)
const installing = ref(false)

let markInstalled = () => {}

defineExpose({
  show: (instanceVal, projectTitleVal, selectedVersions, extMarkInstalled) => {
    instance.value = instanceVal
    projectTitle.value = projectTitleVal
    versions.value = selectedVersions
    selectedVersion.value = selectedVersions[0]
    incompatibleModal.value.show()
    markInstalled = extMarkInstalled
  },
})

const install = async () => {
  installing.value = true
  await installMod(instance.value.path, selectedVersion.value.id).catch(handleError)
  installing.value = false
  markInstalled()
  incompatibleModal.value.hide()
}
</script>

<style lang="scss" scoped>
.data {
  text-transform: capitalize;
}

table {
  width: 100%;
  border-radius: var(--radius-lg);
  border-collapse: collapse;
  box-shadow: 0 0 0 1px var(--color-button-bg);
}

th {
  text-align: left;
  padding: 1rem;
  background-color: var(--color-bg);
  overflow: hidden;
  border-bottom: 1px solid var(--color-button-bg);
}

th:first-child {
  border-top-left-radius: var(--radius-lg);
  border-right: 1px solid var(--color-button-bg);
}

th:last-child {
  border-top-right-radius: var(--radius-lg);
}

td {
  padding: 1rem;
}

td:first-child {
  border-right: 1px solid var(--color-button-bg);
}

.button-group {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}
</style>
