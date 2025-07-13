<template>
  <ModalWrapper
    ref="modal"
    :header="'Rename ' + (isScreenshot ? 'Screenshot' : 'File')"
    @hide="onHide"
  >
    <div class="modal-body">
      <div class="input-group">
        <label for="new-filename" class="label">New filename:</label>
        <input
          id="new-filename"
          ref="filenameInput"
          v-model="newFilename"
          type="text"
          class="input"
          placeholder="Enter new filename"
          @keydown.enter="confirmRename"
          @keydown.esc="cancel"
        />
        <div v-if="error" class="error-message">
          {{ error }}
        </div>
      </div>
      <div class="preview">
        <p class="text-sm text-secondary"><strong>Current:</strong> {{ originalFilename }}</p>
        <p class="text-sm text-secondary">
          <strong>New:</strong> {{ newFilename || 'Enter a filename...' }}
        </p>
      </div>
    </div>
    <div class="modal-footer">
      <Button @click="cancel">Cancel</Button>
      <Button color="primary" :disabled="!newFilename.trim() || isRenaming" @click="confirmRename">
        <SpinnerIcon v-if="isRenaming" class="animate-spin w-4 h-4" />
        {{ isRenaming ? 'Renaming...' : 'Rename' }}
      </Button>
    </div>
  </ModalWrapper>
</template>

<script setup>
import { ref, computed, nextTick } from 'vue'
import { Button } from '@modrinth/ui'
import { SpinnerIcon } from '@modrinth/assets'
import { renameFile } from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

// Simple path utilities for Tauri
const pathUtils = {
  basename: (path) => {
    return path.split(/[\\/]/).pop() || ''
  },
  dirname: (path) => {
    const parts = path.split(/[\\/]/)
    parts.pop()
    return parts.join('\\') // Windows path separator
  },
  join: (dir, filename) => {
    return dir + '\\' + filename
  },
}

const modal = ref()
const filenameInput = ref()
const newFilename = ref('')
const originalFilename = ref('')
const originalPath = ref('')
const error = ref('')
const isRenaming = ref(false)
const isScreenshot = ref(true)

const emit = defineEmits(['renamed', 'cancelled'])

// Extract just the filename without extension
const __filenameWithoutExt = computed(() => {
  const filename = newFilename.value || originalFilename.value
  const lastDotIndex = filename.lastIndexOf('.')
  return lastDotIndex > 0 ? filename.substring(0, lastDotIndex) : filename
})

// Extract the file extension
const fileExtension = computed(() => {
  const filename = originalFilename.value
  const lastDotIndex = filename.lastIndexOf('.')
  return lastDotIndex > 0 ? filename.substring(lastDotIndex) : ''
})

// Show the modal with file information
const show = (filePath, filename = null, isScreenshotFile = true) => {
  originalPath.value = filePath
  originalFilename.value = filename || pathUtils.basename(filePath)
  isScreenshot.value = isScreenshotFile

  // Set initial filename (without extension for easier editing)
  const nameWithoutExt = originalFilename.value.replace(/\.[^/.]+$/, '')
  newFilename.value = nameWithoutExt

  error.value = ''
  isRenaming.value = false

  modal.value.show()

  // Focus the input after the modal is shown
  nextTick(() => {
    if (filenameInput.value) {
      filenameInput.value.focus()
      filenameInput.value.select()
    }
  })
}

// Hide the modal
const hide = () => {
  modal.value.hide()
}

// Handle modal hide event
const onHide = () => {
  newFilename.value = ''
  originalFilename.value = ''
  originalPath.value = ''
  error.value = ''
  isRenaming.value = false
  emit('cancelled')
}

// Cancel renaming
const cancel = () => {
  hide()
}

// Validate the new filename
const validateFilename = (filename) => {
  if (!filename.trim()) {
    return 'Filename cannot be empty'
  }

  // Check for invalid characters
  const invalidChars = /[<>:"/\\|?*]/
  if (invalidChars.test(filename)) {
    return 'Filename contains invalid characters'
  }

  // Check if it's the same as original (case-insensitive)
  const originalWithoutExt = originalFilename.value.replace(/\.[^/.]+$/, '')
  if (filename.toLowerCase() === originalWithoutExt.toLowerCase()) {
    return 'New filename must be different from the original'
  }

  return null
}

// Confirm the rename operation
const confirmRename = async () => {
  const trimmedFilename = newFilename.value.trim()

  // Validate the filename
  const validationError = validateFilename(trimmedFilename)
  if (validationError) {
    error.value = validationError
    return
  }

  // Add the original extension back
  const finalFilename = trimmedFilename + fileExtension.value

  // Construct the new path
  const directory = pathUtils.dirname(originalPath.value)
  const newPath = pathUtils.join(directory, finalFilename)

  isRenaming.value = true
  error.value = ''

  try {
    await renameFile(originalPath.value, newPath)
    emit('renamed', {
      originalPath: originalPath.value,
      newPath: newPath,
      originalFilename: originalFilename.value,
      newFilename: finalFilename,
    })
    hide()
  } catch (err) {
    console.error('Failed to rename file:', err)
    error.value = err.message || 'Failed to rename file'
    handleError(err)
  } finally {
    isRenaming.value = false
  }
}

// Expose methods
defineExpose({
  show,
  hide,
})
</script>

<style lang="scss" scoped>
.modal-header {
  padding: var(--gap-lg);
  border-bottom: 1px solid var(--color-divider);
}

.modal-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.modal-body {
  padding: var(--gap-lg);
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
}

.label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.input {
  padding: var(--gap-sm) var(--gap-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg);
  color: var(--color-text-primary);
  font-size: 0.875rem;

  &:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px rgba(var(--color-primary-rgb), 0.2);
  }
}

.error-message {
  color: var(--color-red);
  font-size: 0.75rem;
  margin-top: var(--gap-xs);
}

.preview {
  background: var(--color-bg-secondary);
  padding: var(--gap-md);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
}

.text-sm {
  font-size: 0.875rem;
  margin: 0 0 var(--gap-xs) 0;

  &:last-child {
    margin-bottom: 0;
  }
}

.text-secondary {
  color: var(--color-text-secondary);
}

.modal-footer {
  padding: var(--gap-lg);
  border-top: 1px solid var(--color-divider);
  display: flex;
  justify-content: flex-end;
  gap: var(--gap-sm);
}

/* Ensure this modal appears above other modals like the image viewer */
:deep(.modal-container) {
  z-index: 1100 !important;
}

:deep(.modal-overlay) {
  z-index: 1099 !important;
}

:deep(.tauri-overlay) {
  z-index: 1098 !important;
}
</style>
