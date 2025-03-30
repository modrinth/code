<template>
  <div v-if="isOpen" class="command-prompt-overlay" @click="close">
    <div class="command-prompt" @click.stop>
      <div class="search-header">
        <div class="search-input">
          <SearchIcon class="search-icon" aria-hidden="true" />
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            placeholder="Type to search..."
            @keydown.esc="close"
            @keydown.enter="onResultSelect"
            @keydown.arrow-down="selectNextResult"
            @keydown.arrow-up="selectPrevResult"
          />
        </div>
        <div class="shortcut-hint">
          <kbd>↑↓</kbd> to navigate
          <kbd>enter</kbd> to select
          <kbd>esc</kbd> to close
        </div>
      </div>
      
      <div v-if="searchQuery" class="search-results" :class="{ 'is-loading': loading }">
        <div v-if="loading" class="loading-container">
          <div class="loading-spinner"></div>
          <span>Searching Modrinth...</span>
        </div>
        
        <template v-else>
          <div class="results-list">
            <div 
              v-for="(project, index) in projects" 
              :key="project.id" 
              class="result-item"
              :class="{ 'selected': selectedIndex === index }" 
              @click="navigateToResult(project)"
              @mouseenter="selectedIndex = index"
            >
              <Avatar :src="project.icon_url" :alt="project.title" size="sm" class="project-avatar" />
              <div class="project-info">
                <span class="project-title">{{ project.title }}</span>
                <span class="project-type">{{ project.project_type }}</span>
              </div>
              <ChevronRightIcon class="chevron-icon" />
            </div>
          </div>

          <div v-if="!projects.length" class="empty-state">
            <XIcon class="empty-icon" />
            <span>No results found</span>
            <p>Try searching for something else</p>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { SearchIcon, ChevronRightIcon, XIcon } from '@modrinth/assets'
import Avatar from './Avatar.vue'

const isOpen = ref(false)
const searchInput = ref(null)
const searchQuery = ref('')
const loading = ref(false)
const projects = ref([])
const selectedIndex = ref(-1)

// Direct search without debounce
watch(searchQuery, async (query) => {
  if (!query) {
    projects.value = []
    return
  }

  loading.value = true
  try {
    const results = await $fetch(`${useRuntimeConfig().public.apiBaseUrl}search`, {
      method: 'GET',
      params: {
        query: query,
        limit: 10, // Increased limit since we only show projects now
        index: 'relevance'
      }
    })
    projects.value = results.hits
  } catch (err) {
    console.error('Search failed:', err)
  }
  loading.value = false
  selectedIndex.value = -1
})

const open = () => {
  isOpen.value = true
  nextTick(() => {
    searchInput.value?.focus()
  })
}

const close = () => {
  isOpen.value = false
  searchQuery.value = ''
}

const navigateToResult = (result) => {
  navigateTo(`/${result.project_type}/${result.slug}`)
  close()
}

const selectNextResult = () => {
  const maxIndex = projects.value.length - 1
  selectedIndex.value = selectedIndex.value >= maxIndex ? 0 : selectedIndex.value + 1
}

const selectPrevResult = () => {
  const maxIndex = projects.value.length - 1  
  selectedIndex.value = selectedIndex.value <= 0 ? maxIndex : selectedIndex.value - 1
}

const onResultSelect = () => {
  if (selectedIndex.value === -1) return
  
  const allResults = [...projects.value]
  const selected = allResults[selectedIndex.value]
  if (selected) {
    navigateToResult(selected)
  }
}

// Handle keyboard shortcut
const handleKeydown = (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    if (!isOpen.value) {
      open()
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

defineExpose({ open, close })
</script>

<style lang="scss" scoped>
.command-prompt-overlay {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-background) 40%, transparent);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
  z-index: 100;
  backdrop-filter: blur(8px);
  animation: overlayShow 150ms ease;
}

.command-prompt {
  width: 90%;
  max-width: 640px;
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  border-radius: var(--radius-lg);
  box-shadow: 
    0 4px 6px -1px rgb(0 0 0 / 0.1),
    0 2px 4px -2px rgb(0 0 0 / 0.1),
    0 0 0 1px rgb(255 255 255 / 0.05);
  overflow: hidden;
  animation: commandPromptShow 200ms cubic-bezier(0.16, 1, 0.3, 1);
}

.search-header {
  padding: 1rem;
  border-bottom: 1px solid var(--color-divider);
}

.search-input {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  background: var(--color-button-bg);
  border-radius: var(--radius-md);
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.5rem;

  .search-icon {
    width: 1rem;
    height: 1rem;
    color: var(--color-text-secondary);
  }

  input {
    flex: 1;
    border: none;
    background: none;
    font-size: 0.9375rem;
    color: var(--color-text);
    outline: none;

    &::placeholder {
      color: var(--color-text-secondary);
    }
  }
}

.shortcut-hint {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0 0.25rem;
  font-size: 0.75rem;
  color: var(--color-text-secondary);

  kbd {
    padding: 0.125rem 0.375rem;
    background: var(--color-button-bg);
    border: 1px solid var(--color-divider);
    border-radius: var(--radius-sm);
    font-family: inherit;
    font-size: 0.75rem;
  }
}

.search-results {
  max-height: min(400px, 60vh);
  overflow-y: auto;

  &.is-loading {
    min-height: 150px;
  }
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2rem;
  color: var(--color-text-secondary);

  .loading-spinner {
    width: 1.5rem;
    height: 1.5rem;
    border: 2px solid var(--color-divider);
    border-top-color: var(--color-brand);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
}

.results-list {
  padding: 0.5rem;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 0.75rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color 150ms ease;

  &:hover, &.selected {
    background: var(--color-button-bg);
  }

  .project-avatar {
    flex-shrink: 0;
  }

  .project-info {
    flex: 1;
    min-width: 0;
  }

  .project-title {
    display: block;
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--color-text);
  }

  .project-type {
    display: block;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    text-transform: capitalize;
  }

  .chevron-icon {
    width: 1rem;
    height: 1rem;
    color: var(--color-text-secondary);
    opacity: 0;
    transition: opacity 150ms ease;
  }

  &:hover .chevron-icon,
  &.selected .chevron-icon {
    opacity: 1;
  }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 3rem 1rem;
  color: var(--color-text-secondary);
  text-align: center;

  .empty-icon {
    width: 2rem;
    height: 2rem;
    margin-bottom: 0.5rem;
    opacity: 0.5;
  }

  span {
    font-weight: 500;
    margin-bottom: 0.25rem;
  }

  p {
    margin: 0;
    font-size: 0.875rem;
  }
}

@keyframes overlayShow {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes commandPromptShow {
  from {
    opacity: 0;
    transform: translateY(-2vh);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
