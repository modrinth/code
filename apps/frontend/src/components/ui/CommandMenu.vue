<template>
  <Transition name="fade">
    <div v-if="show" class="command-menu-overlay" @click="close">
      <div class="command-menu" @click.stop>
        <div class="command-menu-header">
          <SearchIcon class="search-icon" aria-hidden="true" />
          <input
            ref="searchInput"
            v-model="query"
            class="command-menu-input"
            type="text"
            :placeholder="formatMessage(messages.searchPlaceholder)"
            @keydown.esc="close"
            @keydown.enter="handleEnter"
            @keydown.arrow-up.prevent="selectPrevious"
            @keydown.arrow-down.prevent="selectNext"
          />
          <div class="shortcuts" :class="{ 'mac': isMac }">
            <kbd v-if="isMac">⌘</kbd><kbd v-else>Ctrl</kbd><kbd>K</kbd>
          </div>
        </div>

        <div class="command-menu-content">
          <div v-if="query" class="command-menu-section">
            <div class="command-menu-section-header">
              <SearchIcon class="section-icon" aria-hidden="true" />
              {{ formatMessage(messages.searchResults) }}
            </div>
            <div v-if="results.length" ref="resultsContainer" class="command-menu-results" role="listbox">
              <button
                v-for="(result, index) in results"
                :key="result.id"
                class="command-menu-item"
                :class="{ active: selectedIndex === index }"
                role="option"
                @click="selectResult(result)"
                @mouseenter="selectedIndex = index"
              >
                <img :src="result.icon_url" :alt="result.title" class="project-icon" loading="lazy" />
                <div class="project-info">
                  <span class="project-title">{{ result.title }}</span>
                  <span class="project-description">{{ result.description }}</span>
                </div>
                <kbd class="result-shortcut">{{ index + 1 }}</kbd>
              </button>
            </div>
            <div v-else class="command-menu-empty">
              <SearchIcon class="empty-icon" aria-hidden="true" />
              {{ formatMessage(messages.noResults) }}
            </div>
          </div>
          <div v-else class="command-menu-section">
            <div class="command-menu-empty command-menu-start">
              <CompassIcon class="empty-icon" aria-hidden="true" />
              {{ formatMessage(messages.startTyping) }}
              <div class="hints">
                <span>{{ formatMessage(messages.searchHint) }}</span>
                <span>{{ formatMessage(messages.navigationHint) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { SearchIcon, CompassIcon } from '@modrinth/assets'
import { useRouter } from 'vue-router'
import { defineMessage, defineMessages } from '@vintl/vintl'
import { useVIntl } from '#imports'
import { useBaseFetch } from '~/composables/fetch'

const messages = defineMessages({
  searchPlaceholder: {
    id: 'command-menu.search.placeholder',
    defaultMessage: 'Search projects, mods, users...',
  },
  searchResults: {
    id: 'command-menu.search.results',
    defaultMessage: 'Search Results',
  },
  quickActions: {
    id: 'command-menu.quick-actions',
    defaultMessage: 'Quick Actions',
  },
  noResults: {
    id: 'command-menu.search.no-results',
    defaultMessage: 'No matching results found',
  },
  startTyping: {
    id: 'command-menu.search.start-typing',
    defaultMessage: 'Start typing to search across Modrinth...',
  },
  searchHint: {
    id: 'command-menu.search.hint',
    defaultMessage: 'Search for projects, mods, plugins, and more',
  },
  navigationHint: {
    id: 'command-menu.search.navigation-hint',
    defaultMessage: 'Use ↑↓ to navigate, enter to select',
  }
})

const { formatMessage } = useVIntl()
const router = useRouter()

const show = ref(false)
const query = ref('')
const results = ref([])
const selectedIndex = ref(0)
const searchInput = ref(null)
const resultsContainer = ref(null)

const isMac = ref(false)
onMounted(() => {
  isMac.value = navigator?.platform?.toLowerCase().includes('mac') ?? false
})

const searchProjects = async (searchQuery) => {
  if (!searchQuery) {
    results.value = []
    return
  }

  try {
    const response = await useBaseFetch(`search?query=${encodeURIComponent(searchQuery)}&limit=10`)
    results.value = response.hits
  } catch (error) {
    console.error('Search error:', error)
    results.value = []
  }
}

watch(query, (newQuery) => {
  selectedIndex.value = 0
  searchProjects(newQuery)
})

const selectResult = (result) => {
  router.push(`/project/${result.slug}`)
  close()
}

const handleEnter = () => {
  if (results.value[selectedIndex.value]) {
    selectResult(results.value[selectedIndex.value])
  }
}

const selectNext = () => {
  selectedIndex.value = (selectedIndex.value + 1) % results.value.length
  scrollSelectedIntoView()
}

const selectPrevious = () => {
  selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % results.value.length
  scrollSelectedIntoView()
}

const scrollSelectedIntoView = () => {
  if (resultsContainer.value) {
    const activeItem = resultsContainer.value.children[selectedIndex.value]
    if (activeItem) {
      activeItem.scrollIntoView({ block: 'nearest' })
    }
  }
}

const props = defineProps({
  onStateChange: {
    type: Function,
    default: async () => {},
  }
})

const open = async () => {
  show.value = true
  await props.onStateChange(true)
  nextTick(() => {
    searchInput.value?.focus()
  })
}

const close = async () => {
  show.value = false
  await props.onStateChange(false)
  query.value = ''
  results.value = []
  selectedIndex.value = 0
}

const handleKeydown = (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    if (show.value) {
      close()
    } else {
      open()
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})

defineExpose({
  open,
  close,
})
</script>

<style lang="scss" scoped>
.command-menu-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(12px);
  z-index: 100;
  display: grid;
  place-items: start center;
  padding-top: 8vh;
}

.command-menu {
  width: min(640px, 90vw);
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  border-radius: 12px;
  box-shadow: 
    0 0 0 1px rgba(255, 255, 255, 0.1),
    0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  animation: slideDown 0.15s ease-out;

  @media (max-width: 640px) {
    width: 94vw;
    margin: 0 12px;
  }
}

.command-menu-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-divider);
  background: var(--color-bg);

  .search-icon {
    width: 20px;
    height: 20px;
    color: var(--color-text-secondary);
    transition: color 0.15s ease;
  }

  &:focus-within .search-icon {
    color: var(--color-brand);
  }
}

.command-menu-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 14px;
  padding: 0 8px;
  color: var(--color-text);
  
  &::placeholder {
    color: var(--color-text-secondary);
  }
}

.shortcuts {
  display: flex;
  gap: 4px;
  padding: 4px 8px;
  background: var(--color-button-bg);
  border-radius: 6px;
  margin-left: 4px;
  
  kbd {
    padding: 1px 4px;
    font-size: 11px;
    font-family: inherit;
    color: var(--color-text-secondary);
    background: transparent;
    border: none;
  }

  &.mac kbd:first-child {
    font-size: 13px;
  }
}

.command-menu-section {
  &-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    
    .section-icon {
      width: 14px;
      height: 14px;
      opacity: 0.7;
    }
  }
}

.command-menu-content {
  max-height: min(400px, 60vh);
  overflow-y: auto;
}

.command-menu-results {
  padding: 4px;
}

.command-menu-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--color-text);
  text-align: left;
  transition: all 0.15s ease;

  &:hover,
  &.active {
    background: var(--color-button-bg);
    
    .project-title {
      color: var(--color-brand);
    }
  }

  .project-icon {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    object-fit: cover;
  }

  .project-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .project-title {
    font-weight: 600;
    font-size: 14px;
    transition: color 0.15s ease;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-description {
    font-size: 12px;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-shortcut {
    padding: 2px 6px;
    font-size: 11px;
    color: var(--color-text-secondary);
    background: var(--color-button-bg);
    border-radius: 4px;
  }
}

.command-menu-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 16px;
  color: var(--color-text-secondary);
  font-size: 14px;
  text-align: center;

  .empty-icon {
    width: 24px;
    height: 24px;
    opacity: 0.5;
  }
}

.command-menu-start {
  .hints {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    opacity: 0.7;
  }
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
