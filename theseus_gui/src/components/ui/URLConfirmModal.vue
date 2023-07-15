<script setup>
import { Modal } from 'omorphia'
import {ref} from "vue";
import {useFetch} from "@/helpers/fetch.js";
import SearchCard from "@/components/ui/SearchCard.vue";
import {get_categories} from "@/helpers/tags.js";
import {handleError} from "@/store/notifications.js";

const confirmModal = ref(null);
const project = ref(null);
const categories = ref(null);

defineExpose({
  async show(id) {
    project.value = await useFetch(`https://api.modrinth.com/v2/project/${encodeURIComponent(id)}`, 'project');
    categories.value = (await get_categories().catch(handleError)).filter((cat) => project.value.categories.includes(cat.name) && cat.project_type === 'mod');
    confirmModal.value.show()
  },
})
</script>

<template>
  <Modal ref="confirmModal" :header="`Install ${project?.title}`">
    <div class="modal-body">
      <SearchCard :project="project" class="project-card" :categories="categories"/>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.modal-body {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--gap-md);
  padding: var(--gap-lg);
}

.project-card {
  border: 1px solid var(--color-button-bg);
}
</style>
