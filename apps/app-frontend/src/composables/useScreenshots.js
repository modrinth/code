import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { list, getAllScreenshots, showInFolder } from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'
import { convertFileSrc } from '@tauri-apps/api/core'

/**
 * Composable for managing screenshot functionality
 * @param {Object} options - Configuration options
 * @param {Function} options.filterScreenshots - Function to filter screenshots
 * @param {boolean} options.defaultGrouping - Default grouping preference
 */
export function useScreenshots({ filterScreenshots, defaultGrouping = false } = {}) {
  // Reactive state
  const instances = ref([])
  const screenshots = ref([])
  const selectedScreenshot = ref(null)
  const showModal = ref(false)
  const groupByInstance = ref(defaultGrouping)
  const collapsedInstances = ref(new Set())
  const renameModal = ref(null)
  const searchQuery = ref('')
  const debouncedSearchQuery = ref('')

  // Debounce search query to improve performance
  let searchTimeout = null
  watch(searchQuery, (newQuery) => {
    if (searchTimeout) {
      clearTimeout(searchTimeout)
    }
    searchTimeout = setTimeout(() => {
      debouncedSearchQuery.value = newQuery
    }, 300) // 300ms debounce
  }, { immediate: true })

  // Computed property to organize screenshots based on grouping preference
  const organizedScreenshots = computed(() => {
    // Filter screenshots based on debounced search query
    let filteredScreenshots = screenshots.value
    if (debouncedSearchQuery.value.trim()) {
      const query = debouncedSearchQuery.value.toLowerCase().trim()
      filteredScreenshots = screenshots.value.filter(screenshot => {
        return (
          screenshot.filename.toLowerCase().includes(query) ||
          screenshot.profile_path.toLowerCase().includes(query)
        )
      })
    }

    if (groupByInstance.value) {
      // Group screenshots by instance
      const grouped = {}
      filteredScreenshots.forEach((screenshot) => {
        const instancePath = screenshot.profile_path
        if (!grouped[instancePath]) {
          grouped[instancePath] = []
        }
        grouped[instancePath].push(screenshot)
      })

      // Sort screenshots within each group by date (newest first)
      Object.keys(grouped).forEach((instancePath) => {
        grouped[instancePath].sort((a, b) => b.created - a.created)
      })

      return grouped
    } else {
      // Return flat array sorted by newest first
      return [...filteredScreenshots].sort((a, b) => b.created - a.created)
    }
  })

  // Functions
  const toggleGrouping = () => {
    groupByInstance.value = !groupByInstance.value
    // Clear collapsed state when switching modes
    collapsedInstances.value.clear()
  }

  const toggleInstanceCollapse = (instancePath) => {
    if (collapsedInstances.value.has(instancePath)) {
      collapsedInstances.value.delete(instancePath)
    } else {
      collapsedInstances.value.add(instancePath)
    }
  }

  const isInstanceCollapsed = (instancePath) => {
    return collapsedInstances.value.has(instancePath)
  }

  const formatDate = (timestamp) => {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  }

  const getScreenshotUrl = (path) => {
    try {
      return convertFileSrc(path)
    } catch (error) {
      console.error('Failed to convert file path:', path, error)
      return null
    }
  }

  const openModal = (screenshot) => {
    selectedScreenshot.value = screenshot
    showModal.value = true
  }

  const closeModal = () => {
    showModal.value = false
    selectedScreenshot.value = null
  }

  const showInExplorer = async (screenshotPath) => {
    try {
      await showInFolder(screenshotPath)
    } catch (error) {
      console.error('Failed to show file in explorer:', error)
      handleError(error)
    }
  }

  const showRenameModal = (screenshot) => {
    if (renameModal.value) {
      renameModal.value.show(screenshot.path, screenshot.filename, true)
    }
  }

  const onFileRenamed = async (renameData) => {
    try {
      // Update the screenshot in our list
      const screenshotIndex = screenshots.value.findIndex((s) => s.path === renameData.originalPath)
      if (screenshotIndex !== -1) {
        screenshots.value[screenshotIndex] = {
          ...screenshots.value[screenshotIndex],
          path: renameData.newPath,
          filename: renameData.newFilename,
        }
      }

      // Update selectedScreenshot if it's the one being renamed
      if (selectedScreenshot.value && selectedScreenshot.value.path === renameData.originalPath) {
        selectedScreenshot.value = {
          ...selectedScreenshot.value,
          path: renameData.newPath,
          filename: renameData.newFilename,
        }
      }
    } catch (error) {
      console.error('Failed to update screenshot after rename:', error)
      handleError(error)
    }
  }

  const handleKeydown = (event) => {
    if (event.key === 'Escape') {
      closeModal()
    } else if (event.key === 'ArrowLeft') {
      goToPrevious()
    } else if (event.key === 'ArrowRight') {
      goToNext()
    }
  }

  // Navigation functions
  const getFlatScreenshots = () => {
    // Get a flat array of all screenshots in current view
    if (groupByInstance.value && typeof organizedScreenshots.value === 'object') {
      const flat = []
      Object.values(organizedScreenshots.value).forEach((screenshots) => {
        flat.push(...screenshots)
      })
      return flat.sort((a, b) => b.created - a.created)
    } else {
      return organizedScreenshots.value
    }
  }

  const getCurrentIndex = () => {
    if (!selectedScreenshot.value) return -1
    const flatScreenshots = getFlatScreenshots()
    return flatScreenshots.findIndex((s) => s.path === selectedScreenshot.value.path)
  }

  const hasPrevious = computed(() => {
    const currentIndex = getCurrentIndex()
    return currentIndex > 0
  })

  const hasNext = computed(() => {
    const currentIndex = getCurrentIndex()
    const flatScreenshots = getFlatScreenshots()
    return currentIndex !== -1 && currentIndex < flatScreenshots.length - 1
  })

  const goToPrevious = () => {
    const currentIndex = getCurrentIndex()
    if (currentIndex > 0) {
      const flatScreenshots = getFlatScreenshots()
      selectedScreenshot.value = flatScreenshots[currentIndex - 1]
    }
  }

  const goToNext = () => {
    const currentIndex = getCurrentIndex()
    const flatScreenshots = getFlatScreenshots()
    if (currentIndex !== -1 && currentIndex < flatScreenshots.length - 1) {
      selectedScreenshot.value = flatScreenshots[currentIndex + 1]
    }
  }

  const clearSearch = () => {
    searchQuery.value = ''
    debouncedSearchQuery.value = ''
    if (searchTimeout) {
      clearTimeout(searchTimeout)
      searchTimeout = null
    }
  }

  const loadScreenshots = async () => {
    try {
      // Load instances and screenshots
      instances.value = await list().catch(handleError)
      const allScreenshots = await getAllScreenshots()

      // Apply filter if provided, otherwise use all screenshots
      if (filterScreenshots) {
        screenshots.value = filterScreenshots(allScreenshots, instances.value)
      } else {
        screenshots.value = allScreenshots
      }
    } catch (error) {
      console.error('Failed to load screenshots:', error)
      handleError(error)
    }
  }

  // Lifecycle
  onMounted(() => {
    loadScreenshots()
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  return {
    // State
    instances,
    screenshots,
    selectedScreenshot,
    showModal,
    groupByInstance,
    collapsedInstances,
    renameModal,
    searchQuery,

    // Computed
    organizedScreenshots,
    hasPrevious,
    hasNext,

    // Functions
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
    loadScreenshots,
    clearSearch,
  }
}
