<script setup>
import { Avatar, Modal, Button } from 'omorphia'
import { computed, ref } from 'vue'
import { add_project_from_version as installMod, list } from '@/helpers/profile'
import { convertFileSrc } from '@tauri-apps/api/tauri'

const version = ref('')
const installModal = ref(null)
const searchFilter = ref('')

defineExpose({
  show: (id) => {
    version.value = id
    installModal.value.show()
    searchFilter.value = ''
  },
})

const profiles = ref(await list().then(Object.values))

async function install(instance) {
  console.log(instance)
  await installMod(instance, version.value)
  installModal.value.hide()
}

const filteredVersions = computed(() => {
  return profiles.value.filter((profile) => {
    return profile.metadata.name.toLowerCase().includes(searchFilter.value.toLowerCase())
  })
})
</script>

<template>
  <Modal ref="installModal" header="Install Mod">
    <div class="modal-body">
      <input v-model="searchFilter" type="text" class="search" placeholder="Search for a profile" />
      <div class="profiles">
        <div
          v-for="profile in filteredVersions"
          :key="profile.metadata.name"
          class="option btn"
          @click="install(profile.path)"
        >
          <Avatar :src="convertFileSrc(profile.metadata.icon)" size="xs" />
          <div class="name">{{ profile.metadata.name }}</div>
        </div>
      </div>
      <div class="footer">
        <Button @click="installModal.hide()">Cancel</Button>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}

.profiles {
  max-height: 18rem;
  overflow-y: auto;
}

.option {
  width: calc(100%);
  background: var(--color-raised-bg);
  color: var(--color-base);
  box-shadow: none;
  display: flex;
  flex-direction: row;
  padding: 0.5rem;

  img {
    margin-right: 0.5rem;
  }

  .name {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
}

.footer {
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
