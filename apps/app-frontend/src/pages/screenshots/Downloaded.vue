<script setup>
import { computed } from 'vue'
import { useScreenshots } from '@/composables/useScreenshots.js'
import RenameFileModal from '@/components/ui/RenameFileModal.vue'
import ScreenshotGrid from '@/components/ui/ScreenshotGrid.vue'
import ScreenshotModal from '@/components/ui/ScreenshotModal.vue'

// Use the composable with a filter for downloaded instances only
const {
  instances,
  screenshots,
  organizedScreenshots,
  selectedScreenshot,
  showModal,
  groupByInstance,
  renameModal,
  __searchQuery,
  hasPrevious,
  hasNext,
  toggleGrouping,
  toggleInstanceCollapse,
  isInstanceCollapsed,
  formatDate,
  getScreenshotUrl,
  openModal,
  closeModal,
  goToPrevious,
  goToNext,
  showInExplorer,
  showRenameModal,
  onFileRenamed,
  __clearSearch,
} = useScreenshots({
  filterScreenshots: (allScreenshots, instances) => {
    const downloadedInstances = instances.filter((i) => i.linked_data)
    const downloadedInstancePaths = downloadedInstances.map((i) => i.path)
    return allScreenshots.filter((screenshot) =>
      downloadedInstancePaths.includes(screenshot.profile_path),
    )
  },
  defaultGrouping: false,
})

// Computed properties
const hasDownloadedInstances = computed(() => instances.value.some((i) => i.linked_data))
</script>

<template>
  <div>
    <div v-if="screenshots.length > 0" class="screenshots-section">
      <div class="section-header">
        <h2 class="text-xl font-semibold">Screenshots from Downloaded Instances</h2>
        <button
          class="group-toggle-btn"
          :class="{ 'bg-bg-raised': groupByInstance }"
          @click="toggleGrouping"
        >
          {{ groupByInstance ? 'Sort by Newest' : 'Group by Instance' }}
        </button>
      </div>

      <ScreenshotGrid
        :organized-screenshots="organizedScreenshots"
        :group-by-instance="groupByInstance"
        :show-instance-path="!groupByInstance"
        :toggle-instance-collapse="toggleInstanceCollapse"
        :is-instance-collapsed="isInstanceCollapsed"
        :open-modal="openModal"
        :get-screenshot-url="getScreenshotUrl"
        :format-date="formatDate"
      />
    </div>

    <div v-else-if="hasDownloadedInstances" class="no-screenshots">
      <h3>No screenshots found</h3>
      <p>Take some screenshots in your downloaded instances to see them here!</p>
    </div>

    <!-- Screenshot Modal -->
    <ScreenshotModal
      :show-modal="showModal"
      :selected-screenshot="selectedScreenshot"
      :show-instance-path="true"
      :close-modal="closeModal"
      :get-screenshot-url="getScreenshotUrl"
      :format-date="formatDate"
      :show-in-explorer="showInExplorer"
      :show-rename-modal="showRenameModal"
      :has-previous="hasPrevious"
      :has-next="hasNext"
      :go-to-previous="goToPrevious"
      :go-to-next="goToNext"
    />
  </div>

  <!-- Rename Modal -->
  <RenameFileModal ref="renameModal" @renamed="onFileRenamed" />
</template>

<style lang="scss" scoped>
.screenshots-section {
  margin-top: var(--gap-lg);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--gap-lg);
}

.group-toggle-btn {
  background: var(--color-button-bg);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--gap-sm) var(--gap-md);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.875rem;
  font-weight: 500;

  &:hover {
    background: var(--color-button-bg-hover);
    transform: translateY(-1px);
  }
}

.no-screenshots {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  gap: var(--gap-sm);
  text-align: center;
  color: var(--color-text-secondary);

  h3 {
    margin: 0;
    color: var(--color-text-primary);
  }

  p {
    margin: 0;
  }
}
</style>
