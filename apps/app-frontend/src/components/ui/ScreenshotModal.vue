<template>
  <div v-if="showModal && selectedScreenshot" class="modal-overlay" @click="closeModal">
    <button class="modal-close" @click="closeModal">&times;</button>

    <!-- Navigation buttons -->
    <button
      v-if="hasPrevious"
      class="nav-btn nav-prev"
      @click.stop="goToPrevious"
      title="Previous image"
    >
      <svg width="24" height="24" viewBox="0 0 16 16" fill="currentColor">
        <path
          fill-rule="evenodd"
          d="M11.354 1.646a.5.5 0 0 1 0 .708L5.707 8l5.647 5.646a.5.5 0 0 1-.708.708l-6-6a.5.5 0 0 1 0-.708l6-6a.5.5 0 0 1 .708 0z"
        />
      </svg>
    </button>

    <button v-if="hasNext" class="nav-btn nav-next" @click.stop="goToNext" title="Next image">
      <svg width="24" height="24" viewBox="0 0 16 16" fill="currentColor">
        <path
          fill-rule="evenodd"
          d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"
        />
      </svg>
    </button>

    <div class="modal-content" @click.stop>
      <img
        :src="getScreenshotUrl(selectedScreenshot.path)"
        :alt="selectedScreenshot.filename"
        class="modal-image"
      />
      <div class="modal-info">
        <h3>{{ selectedScreenshot.filename }}</h3>
        <p v-if="showInstancePath">Instance: {{ selectedScreenshot.profile_path }}</p>
        <p>Date: {{ formatDate(selectedScreenshot.created) }}</p>
        <div class="modal-actions">
          <button
            class="action-btn"
            @click="showInExplorer(selectedScreenshot.path)"
            title="Show in File Explorer"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path
                d="M1.5 1A1.5 1.5 0 0 0 0 2.5v11A1.5 1.5 0 0 0 1.5 15h13a1.5 1.5 0 0 0 1.5-1.5v-11A1.5 1.5 0 0 0 14.5 1h-13zM1 2.5A.5.5 0 0 1 1.5 2h13a.5.5 0 0 1 .5.5V4H1V2.5zM1 5h14v8.5a.5.5 0 0 1-.5.5h-13a.5.5 0 0 1-.5-.5V5z"
              />
              <path
                d="M3.5 6.5a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 0 1h-8a.5.5 0 0 1-.5-.5zm0 2a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 0 1h-8a.5.5 0 0 1-.5-.5z"
              />
            </svg>
            Show in Explorer
          </button>
          <button
            class="action-btn"
            @click="showRenameModal(selectedScreenshot)"
            title="Rename File"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path
                d="M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708L14.5 5.207l-3-3L12.146.146ZM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207 11.207 2.5Zm1.586 3L10.5 3.207 4 9.707V13h3.293l6.5-6.5Z"
              />
              <path
                fill-rule="evenodd"
                d="M1 13.5A1.5 1.5 0 0 0 2.5 15h11a1.5 1.5 0 0 0 1.5-1.5v-6a.5.5 0 0 0-1 0v6a.5.5 0 0 1-.5.5h-11a.5.5 0 0 1-.5-.5v-11a.5.5 0 0 1 .5-.5H9a.5.5 0 0 0 0-1H2.5A1.5 1.5 0 0 0 1 2.5v11Z"
              />
            </svg>
            Rename
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  showModal: {
    type: Boolean,
    required: true,
  },
  selectedScreenshot: {
    type: Object,
    default: null,
  },
  showInstancePath: {
    type: Boolean,
    default: true,
  },
  closeModal: {
    type: Function,
    required: true,
  },
  getScreenshotUrl: {
    type: Function,
    required: true,
  },
  formatDate: {
    type: Function,
    required: true,
  },
  showInExplorer: {
    type: Function,
    required: true,
  },
  showRenameModal: {
    type: Function,
    required: true,
  },
  // Navigation props
  hasPrevious: {
    type: Boolean,
    default: false,
  },
  hasNext: {
    type: Boolean,
    default: false,
  },
  goToPrevious: {
    type: Function,
    default: () => {},
  },
  goToNext: {
    type: Function,
    default: () => {},
  },
})
</script>

<style lang="scss" scoped>
/* Modal styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--gap-lg);
  backdrop-filter: blur(4px);
}

.modal-content {
  position: relative;
  max-width: 95vw;
  max-height: 95vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--gap-md);
}

.modal-close {
  position: fixed;
  top: 70px;
  right: 70px;
  background: rgba(79, 79, 79, 0.7);
  border: none;
  color: white;
  font-size: 2rem;
  cursor: pointer;
  z-index: 1001;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: background-color 0.2s ease;
  backdrop-filter: blur(4px);

  &:hover {
    background: rgba(133, 83, 83, 0.9);
  }
}

.nav-btn {
  position: fixed;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(0, 0, 0, 0.7);
  border: none;
  color: white;
  font-size: 1.5rem;
  cursor: pointer;
  z-index: 1001;
  width: 50px;
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s ease;
  backdrop-filter: blur(4px);

  &:hover {
    background: rgba(0, 0, 0, 0.9);
    transform: translateY(-50%) scale(1.1);
  }

  svg {
    width: 24px;
    height: 24px;
  }
}

.nav-prev {
  left: 20px;
}

.nav-next {
  right: 20px;
}

.modal-image {
  max-width: 100%;
  max-height: 80vh;
  object-fit: contain;
  border-radius: var(--radius-md);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.modal-info {
  background: var(--color-bg-secondary);
  padding: var(--gap-md);
  border-radius: var(--radius-md);
  text-align: center;
  min-width: 300px;

  h3 {
    margin: 0 0 var(--gap-sm) 0;
    color: var(--color-text-primary);
    font-size: 1.125rem;
  }

  p {
    margin: 0 0 var(--gap-xs) 0;
    color: var(--color-text-secondary);
    font-size: 0.875rem;

    &:last-child {
      margin-bottom: var(--gap-md);
    }
  }
}

.modal-actions {
  display: flex;
  gap: var(--gap-sm);
  margin-top: var(--gap-md);
}

.action-btn {
  background: var(--color-button-bg);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--gap-sm) var(--gap-md);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.875rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--gap-xs);

  &:hover {
    background: var(--color-button-bg-hover);
    transform: translateY(-1px);
  }

  svg {
    width: 16px;
    height: 16px;
  }
}
</style>
