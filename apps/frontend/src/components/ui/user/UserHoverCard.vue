<template>
  <span 
    ref="triggerEl" 
    class="user-hover-card-trigger" 
    @mouseenter="startHover" 
    @mouseleave="handleLeave"
  >
    <slot />
  </span>
  <Teleport to="body">
    <Transition name="fade">
      <div 
        v-if="shown && userData?.username" 
        ref="cardEl"
        class="hover-content" 
        :style="cardStyle"
        @mouseenter="keepCard"
        @mouseleave="handleLeave"
      >
        <div class="user-info">
          <div class="user-header">
            <nuxt-link :to="`/user/${userData.username}`" class="user-link">
              <img
                :src="userData.avatar_url"
                :alt="`${userData.username}'s avatar`"
                class="user-avatar"
              />
              <div>
                <h3>{{ userData.username }}</h3>
                <div class="user-joined">
                  Joined {{ new Date(userData.created).toLocaleDateString() }}
                </div>
              </div>
            </nuxt-link>
          </div>
          <div v-if="userData.bio" class="user-bio">
            {{ userData.bio }}
          </div>
          <div class="user-stats">
            <div>
              <span class="stat-value">{{ userData.projects_count }}</span> projects
            </div>
          </div>
          <div v-if="projects?.length" class="user-projects">
            <div v-for="project in projects" :key="project.id" class="project-preview">
              <img :src="project.icon_url" :alt="`${project.title} icon`" class="project-icon" />
              <div class="project-info">
                <nuxt-link 
                  :to="`/${$getProjectTypeForUrl(project.project_type || project.type, project.categories)}/${project.slug || project.id}`" 
                  class="project-title"
                >
                  {{ project.title }}
                </nuxt-link>
                <div class="project-stats">
                  <span>{{ $formatNumber(project.downloads) }} downloads</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'

// Remove router import since we don't need it anymore

const props = defineProps({
  userId: { type: String, required: true }
})

const shown = ref(false)
const userData = ref(null)
const projects = ref([])
const triggerEl = ref(null)
const cardEl = ref(null)
const cardStyle = ref({})

let hoverTimeout = null
let isHovering = false
let isHoveringCard = false

// Add delay before hiding to allow mouse movement to card
let hideTimeout = null

// Increase hide delay constant
const HIDE_DELAY = 500

function updatePosition() {
  if (!triggerEl.value || !shown.value) return
  
  const rect = triggerEl.value.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const showAbove = rect.bottom > viewportHeight - 300 && rect.top > 300

  const style = {
    position: 'fixed',
    left: `${rect.left}px`,
    ...(showAbove 
      ? { bottom: `${viewportHeight - rect.top + 8}px` }
      : { top: `${rect.bottom + 8}px` }),
    minWidth: '300px',
    maxWidth: '400px',
    maxHeight: '300px'
  }
  
  Object.assign(cardStyle.value, style)
}

function handleScroll() {
  // Immediate hide on scroll
  hideCard()
}

function startHideCard() {
  hideTimeout = setTimeout(() => {
    if (!isHoveringCard && !isHovering) {
      hideCard()
    }
  }, HIDE_DELAY)
}

function keepCard() {
  isHoveringCard = true
  if (hideTimeout) {
    clearTimeout(hideTimeout)
  }
}

function handleLeave() {
  isHoveringCard = false
  isHovering = false
  startHideCard()
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll, { passive: true })
  window.addEventListener('resize', handleScroll, { passive: true })
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll)
  window.removeEventListener('resize', handleScroll)
  clearTimeout(hoverTimeout)
  if (hideTimeout) {
    clearTimeout(hideTimeout)
  }
})

async function startHover() {
  if (hoverTimeout) return
  
  isHovering = true
  hoverTimeout = setTimeout(async () => {
    if (!isHovering) return

    if (!userData.value) {
      try {
        const userResp = await useBaseFetch(`user/${props.userId}`)
        const projectsResp = await useBaseFetch(`user/${props.userId}/projects`)
        
        userData.value = userResp || null
        projects.value = (projectsResp || []).slice(0, 3)
        
        if (userData.value) {
          userData.value.projects_count = projectsResp?.length || 0
        }
      } catch (err) {
        console.error('Failed to load user data:', err)
        return
      }
    }

    if (isHovering) {
      shown.value = true
      nextTick(() => {
        updatePosition()
      })
    }
  }, 150) // 150ms delay before loading
}

function hideCard() {
  shown.value = false
  if (hoverTimeout) {
    clearTimeout(hoverTimeout)
    hoverTimeout = null
  }
}

</script>

<style lang="scss" scoped>
.user-hover-card-trigger {
  position: relative;
  color: inherit;
  cursor: pointer;

  a {
    text-decoration: none;
    
    &:hover {
      text-decoration: underline;
    }
  }
}

.hover-content {
  z-index: 9999;
  position: fixed;
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  border-radius: var(--size-rounded-lg);
  box-shadow: var(--shadow-card);
  overflow-y: auto;
  pointer-events: auto;
}

.loading-spinner {
  display: inline-block;
  width: 1.5rem;
  height: 1.5rem;
  border: 2px solid var(--color-text-secondary);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.user-info {
  padding: var(--spacing-card-md);
}

.user-header {
  .user-link {
    display: flex;
    align-items: center;
    gap: var(--spacing-card-sm);
    color: inherit;
    text-decoration: none;
    
    &:hover h3 {
      color: var(--color-link);
      text-decoration: underline;
    }

    h3 {
      margin: 0;
      line-height: 1.2;
      transition: color 0.2s ease;
    }
  }
}

.user-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
}

.user-joined {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.user-bio {
  margin-top: var(--spacing-card-sm);
  font-size: var(--font-size-sm);
}

.user-stats {
  margin-top: var(--spacing-card-sm);
  display: flex;
  gap: var(--spacing-card-md);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);

  .stat-value {
    font-weight: bold;
    color: var(--color-text);
  }
}

.loading {
  padding: var(--spacing-card-md);
}

.user-projects {
  margin-top: var(--spacing-card-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);
}

.project-preview {
  display: flex;
  align-items: center;
  gap: var(--spacing-card-sm);
  padding: var(--spacing-card-sm);
  border-radius: var(--size-rounded-sm);
  background: var(--color-button-bg);
  transition: background 0.2s ease;

  &:hover {
    background: var(--color-button-bg-hover);
  }
}

.project-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--size-rounded-sm);
}

.project-info {
  flex: 1;
  min-width: 0;
}

.project-title {
  display: block;
  font-weight: 500;
  color: var(--color-text);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover {
    color: var(--color-link);
  }
}

.project-stats {
  font-size: var(--font-size-sm);
  color: var (--color-text-secondary);
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
