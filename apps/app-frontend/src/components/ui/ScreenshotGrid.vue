<template>
  <!-- Grouped view -->
  <div v-if="groupByInstance && typeof organizedScreenshots === 'object'">
    <div v-for="(screenshots, instancePath) in organizedScreenshots" :key="instancePath" class="instance-group">
      <div class="instance-header" @click="toggleInstanceCollapse(instancePath)">
        <h3 class="instance-title">{{ instancePath }}</h3>
        <div class="collapse-icon" :class="{ collapsed: isInstanceCollapsed(instancePath) }">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 12l-4-4h8l-4 4z"/>
          </svg>
        </div>
      </div>
      <div v-if="!isInstanceCollapsed(instancePath)" class="screenshots-grid instance-screenshots">
        <div 
          v-for="screenshot in screenshots" 
          :key="screenshot.path" 
          class="screenshot-card"
          @click="openModal(screenshot)"
        >
          <div class="screenshot-image">
            <img 
              :src="getScreenshotUrl(screenshot.path)" 
              :alt="screenshot.filename"
              loading="lazy"
            />
          </div>
          <div class="screenshot-info bg-bg-raised">
            <p class="screenshot-filename">{{ screenshot.filename }}</p>
            <p v-if="!groupByInstance" class="screenshot-instance">{{ screenshot.profile_path }}</p>
            <p class="screenshot-date">{{ formatDate(screenshot.created) }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
  
  <!-- Flat view (newest first) -->
  <div v-else class="screenshots-grid">
    <div 
      v-for="screenshot in organizedScreenshots" 
      :key="screenshot.path" 
      class="screenshot-card"
      @click="openModal(screenshot)"
    >
      <div class="screenshot-image">
        <img 
          :src="getScreenshotUrl(screenshot.path)" 
          :alt="screenshot.filename"
          loading="lazy"
        />
      </div>
      <div class="screenshot-info bg-bg-raised">
        <p class="screenshot-filename">{{ screenshot.filename }}</p>
        <p v-if="showInstancePath && !groupByInstance" class="screenshot-instance">{{ screenshot.profile_path }}</p>
        <p class="screenshot-date">{{ formatDate(screenshot.created) }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  organizedScreenshots: {
    type: [Object, Array],
    required: true
  },
  groupByInstance: {
    type: Boolean,
    required: true
  },
  showInstancePath: {
    type: Boolean,
    default: true
  },
  toggleInstanceCollapse: {
    type: Function,
    required: true
  },
  isInstanceCollapsed: {
    type: Function,
    required: true
  },
  openModal: {
    type: Function,
    required: true
  },
  getScreenshotUrl: {
    type: Function,
    required: true
  },
  formatDate: {
    type: Function,
    required: true
  }
})
</script>

<style lang="scss" scoped>
.instance-group {
  margin-bottom: var(--gap-xl);

  &:last-child {
    margin-bottom: 0;
  }
}

.instance-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s ease;
  padding: var(--gap-sm);
  margin: calc(var(--gap-sm) * -1);
  border-radius: var(--radius-md);

  &:hover {
    background-color: var(--color-bg-tertiary);
  }
}

.instance-title {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-primary);
  flex: 1;
}

.collapse-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: var(--color-text-secondary);
  transition: transform 0.2s ease;

  &.collapsed {
    transform: rotate(-90deg);
  }

  svg {
    width: 16px;
    height: 16px;
  }
}

.instance-screenshots {
  margin-top: var(--gap-md);
}

.screenshots-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: var(--gap-lg);
  contain: layout style;
}

.screenshot-card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--color-bg-secondary);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  cursor: pointer;
  will-change: transform;
  contain: layout style;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }
}

.screenshot-image {
  width: 100%;
  height: 160px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-tertiary);
  contain: layout style;

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.2s ease;
    will-change: transform;
  }

  .screenshot-card:hover & img {
    transform: scale(1.05);
  }
}

.screenshot-info {
  padding: var(--gap-md);
  background-color: var(--color-raised-bg);

  .screenshot-filename {
    margin: 0 0 var(--gap-xs) 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .screenshot-instance {
    margin: 0 0 var(--gap-xs) 0;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .screenshot-date {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }
}
</style>
