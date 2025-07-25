<template>
  <ModalWrapper ref="modal" :header="'Import from CurseForge Profile Code'">
    <div class="modal-body">
      <div class="input-row">
        <p class="input-label">Profile Code</p>
        <div class="iconified-input">
          <SearchIcon aria-hidden="true" class="text-lg" />
          <input
            ref="codeInput"
            v-model="profileCode"
            autocomplete="off"
            class="h-12 card-shadow"
            spellcheck="false"
            type="text"
            placeholder="Enter CurseForge profile code"
            maxlength="20"
            @keyup.enter="importProfile"
          />
          <Button v-if="profileCode" class="r-btn" @click="() => (profileCode = '')">
            <XIcon />
          </Button>
        </div>
      </div>
      
      <div v-if="metadata && !importing" class="profile-info">
        <h3>Profile Information</h3>
        <p><strong>Name:</strong> {{ metadata.name }}</p>
      </div>
      
      <div v-if="error" class="error-message">
        <p>{{ error }}</p>
      </div>
      
      <div v-if="importing && importProgress.visible" class="progress-section">
        <div class="progress-info">
          <span class="progress-text">{{ importProgress.message }}</span>
          <span class="progress-percentage">{{ Math.floor(importProgress.percentage) }}%</span>
        </div>
        <div class="progress-bar-container">
          <div
            class="progress-bar" 
            :style="{ width: `${importProgress.percentage}%` }"
          ></div>
        </div>
      </div>
      
      <div class="button-row">
        <Button :disabled="importing" @click="hide">
          <XIcon />
          Cancel
        </Button>
        <Button 
          v-if="!metadata" 
          :disabled="!profileCode.trim() || fetching"
          color="secondary"
          @click="fetchMetadata" 
        >
          <SearchIcon v-if="!fetching" />
          {{ fetching ? 'Checking...' : 'Check Profile' }}
        </Button>
        <Button 
          v-if="metadata" 
          :disabled="importing"
          color="primary"
          @click="importProfile" 
        >
          <DownloadIcon v-if="!importing" />
          {{ importing ? 'Importing...' : 'Import Profile' }}
        </Button>
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup>
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { Button } from '@modrinth/ui'
import { 
  XIcon, 
  SearchIcon, 
  DownloadIcon 
} from '@modrinth/assets'
import { 
  fetch_curseforge_profile_metadata, 
  import_curseforge_profile 
} from '@/helpers/import.js'
import { handleError } from '@/store/notifications.js'
import { trackEvent } from '@/helpers/analytics'
import { loading_listener } from '@/helpers/events.js'

const props = defineProps({
  closeParent: {
    type: Function,
    default: null
  }
})

const router = useRouter()
const modal = ref(null)
const codeInput = ref(null)
const profileCode = ref('')
const metadata = ref(null)
const fetching = ref(false)
const importing = ref(false)
const error = ref('')
const importProgress = ref({
  visible: false,
  percentage: 0,
  message: 'Starting import...',
  totalMods: 0,
  downloadedMods: 0
})

let unlistenLoading = null
let activeLoadingBarId = null
let progressFallbackTimer = null

defineExpose({
  show: () => {
    profileCode.value = ''
    metadata.value = null
    fetching.value = false
    importing.value = false
    error.value = ''
    importProgress.value = {
      visible: false,
      percentage: 0,
      message: 'Starting import...',
      totalMods: 0,
      downloadedMods: 0
    }
    modal.value?.show()
    
    nextTick(() => {
      setTimeout(() => {
        codeInput.value?.focus()
      }, 100)
    })
    
    trackEvent('CurseForgeProfileImportStart', { source: 'ImportModal' })
  },
})

const hide = () => {
  modal.value?.hide()
}

const fetchMetadata = async () => {
  if (!profileCode.value.trim()) return
  
  fetching.value = true
  error.value = ''
  
  try {
    const result = await fetch_curseforge_profile_metadata(profileCode.value.trim())
    metadata.value = result
    trackEvent('CurseForgeProfileMetadataFetched', { 
      profileCode: profileCode.value.trim() 
    })
  } catch (err) {
    console.error('Failed to fetch CurseForge profile metadata:', err)
    error.value = 'Failed to fetch profile information. Please check the code and try again.'
    handleError(err)
  } finally {
    fetching.value = false
  }
}

const importProfile = async () => {
  if (!profileCode.value.trim()) return
  
  importing.value = true
  error.value = ''
  activeLoadingBarId = null // Reset for new import session
  importProgress.value = {
    visible: true,
    percentage: 0,
    message: 'Starting import...',
    totalMods: 0,
    downloadedMods: 0
  }
  
  // Fallback progress timer in case loading events don't work
  progressFallbackTimer = setInterval(() => {
    if (importing.value && importProgress.value.percentage < 90) {
      // Slowly increment progress as a fallback
      importProgress.value.percentage = Math.min(90, importProgress.value.percentage + 1)
    }
  }, 1000)
  
  try {
    const { profilePath } = await import_curseforge_profile(profileCode.value.trim())
    
    trackEvent('CurseForgeProfileImported', { 
      profileCode: profileCode.value.trim() 
    })
    
    hide()
    
    // Close the parent modal if provided
    if (props.closeParent) {
      props.closeParent()
    }
    
    // Navigate to the imported profile
    await router.push(`/instance/${encodeURIComponent(profilePath)}`)
  } catch (err) {
    console.error('Failed to import CurseForge profile:', err)
    error.value = 'Failed to import profile. Please try again.'
    handleError(err)
  } finally {
    importing.value = false
    importProgress.value.visible = false
    if (progressFallbackTimer) {
      clearInterval(progressFallbackTimer)
      progressFallbackTimer = null
    }
    activeLoadingBarId = null
  }
}

onMounted(async () => {
  // Listen for loading events to update progress
  unlistenLoading = await loading_listener((event) => {
    console.log('Loading event received:', event) // Debug log
    
    // Handle all loading events that could be related to CurseForge profile import
    const isCurseForgeEvent = event.event?.type === 'curseforge_profile_download'
    const hasProfileName = event.event?.profile_name && importing.value
    
    if ((isCurseForgeEvent || hasProfileName) && importing.value) {
      // Store the loading bar ID for this import session
      if (!activeLoadingBarId) {
        activeLoadingBarId = event.loader_uuid
      }
      
      // Only process events for our current import session
      if (event.loader_uuid === activeLoadingBarId) {
        if (event.fraction !== null && event.fraction !== undefined) {
          const baseProgress = (event.fraction || 0) * 100
          
          // Calculate custom progress based on the message
          let finalProgress = baseProgress
          const message = event.message || 'Importing profile...'
          
          // Custom progress calculation for different stages
          if (message.includes('Fetching') || message.includes('metadata')) {
            finalProgress = Math.min(10, baseProgress)
          } else if (message.includes('Downloading profile ZIP') || message.includes('profile ZIP')) {
            finalProgress = Math.min(15, 10 + (baseProgress - 10) * 0.5)
          } else if (message.includes('Extracting') || message.includes('ZIP')) {
            finalProgress = Math.min(20, 15 + (baseProgress - 15) * 0.5)
          } else if (message.includes('Configuring') || message.includes('profile')) {
            finalProgress = Math.min(30, 20 + (baseProgress - 20) * 0.5)
          } else if (message.includes('Copying') || message.includes('files')) {
            finalProgress = Math.min(40, 30 + (baseProgress - 30) * 0.5)
          } else if (message.includes('Downloaded mod') && message.includes(' of ')) {
            // Parse "Downloaded mod X of Y" message
            const match = message.match(/Downloaded mod (\d+) of (\d+)/)
            if (match) {
              const current = parseInt(match[1])
              const total = parseInt(match[2])
              // Mods take 40% of progress (from 40% to 80%)
              const modProgress = (current / total) * 40
              finalProgress = 40 + modProgress
            } else {
              finalProgress = Math.min(80, 40 + (baseProgress - 40) * 0.5)
            }
          } else if (message.includes('Downloading mod') || message.includes('mods')) {
            // General mod downloading stage (40% to 80%)
            finalProgress = Math.min(80, 40 + (baseProgress - 40) * 0.4)
          } else if (message.includes('Installing Minecraft') || message.includes('Minecraft')) {
            finalProgress = Math.min(95, 80 + (baseProgress - 80) * 0.75)
          } else if (message.includes('Finalizing') || message.includes('completed')) {
            finalProgress = Math.min(100, 95 + (baseProgress - 95))
          } else {
            // Default: use the base progress but ensure minimum progression
            finalProgress = Math.max(importProgress.value.percentage, baseProgress)
          }
          
          importProgress.value.percentage = Math.min(100, Math.max(0, finalProgress))
          importProgress.value.message = message
        } else {
          // Loading complete
          importProgress.value.percentage = 100
          importProgress.value.message = 'Import completed!'
          activeLoadingBarId = null
        }
      }
    }
  })
})

onUnmounted(() => {
  if (unlistenLoading) {
    unlistenLoading()
  }
  if (progressFallbackTimer) {
    clearInterval(progressFallbackTimer)
  }
})
</script>

<style lang="scss" scoped>
.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}

.input-row {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.input-label {
  font-weight: 600;
  color: var(--color-contrast);
  margin: 0;
}

.profile-info {
  background: var(--color-bg);
  border: 1px solid var(--color-button);
  border-radius: var(--radius-md);
  padding: 1rem;
  
  h3 {
    margin: 0 0 0.5rem 0;
    color: var(--color-contrast);
  }
  
  p {
    margin: 0.25rem 0;
    color: var(--color-base);
  }
}

.error-message {
  background: var(--color-red);
  border: 1px solid var(--color-red);
  border-radius: var(--radius-md);
  padding: 0.75rem;
  
  p {
    margin: 0;
    color: var(--color-contrast);
    font-weight: 500;
  }
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  
  .progress-text {
    color: var(--color-base);
    font-size: 0.875rem;
  }
  
  .progress-percentage {
    color: var(--color-contrast);
    font-size: 0.875rem;
    font-weight: 600;
  }
}

.progress-bar-container {
  width: 100%;
  height: 4px;
  background: var(--color-button);
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: var(--color-brand);
  border-radius: 2px;
  transition: width 0.3s ease;
  min-width: 0;
}

.button-row {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
  margin-top: auto;
}
</style>
