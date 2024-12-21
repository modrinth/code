<template>
  <div
    ref="dropdown"
    data-pyro-dropdown
    tabindex="0"
    role="combobox"
    :aria-expanded="dropdownVisible"
    class="relative inline-block h-9 w-full max-w-80"
    @focus="onFocus"
    @blur="onBlur"
    @mousedown.prevent
    @keydown="handleKeyDown"
  >
    <div
      data-pyro-dropdown-trigger
      class="duration-50 flex h-full w-full cursor-pointer select-none items-center justify-between gap-4 rounded-xl bg-button-bg px-4 py-2 shadow-sm transition-all ease-in-out"
      :class="triggerClasses"
      @click="toggleDropdown"
    >
      <span>{{ selectedOption }}</span>
      <DropdownIcon
        class="transition-transform duration-200 ease-in-out"
        :class="{ 'rotate-180': dropdownVisible }"
      />
    </div>

    <Teleport to="#teleports">
      <transition
        enter-active-class="transition-opacity duration-200 ease-out"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition-opacity duration-200 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div
          v-if="dropdownVisible"
          ref="optionsContainer"
          data-pyro-dropdown-options
          class="experimental-styles-within fixed z-50 bg-button-bg shadow-lg"
          :class="{
            'rounded-b-xl': !isRenderingUp,
            'rounded-t-xl': isRenderingUp,
          }"
          :style="positionStyle"
          @keydown.stop="handleDropdownKeyDown"
        >
          <div
            class="overflow-y-auto"
            :style="{ height: `${virtualListHeight}px` }"
            data-pyro-dropdown-options-virtual-scroller
            @scroll="handleScroll"
          >
            <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
              <div
                v-for="item in visibleOptions"
                :key="item.index"
                data-pyro-dropdown-option
                :style="{
                  position: 'absolute',
                  top: 0,
                  transform: `translateY(${item.index * ITEM_HEIGHT}px)`,
                  width: '100%',
                  height: `${ITEM_HEIGHT}px`,
                }"
              >
                <div
                  :ref="(el) => handleOptionRef(el as HTMLElement, item.index)"
                  role="option"
                  :tabindex="focusedOptionIndex === item.index ? 0 : -1"
                  class="hover:brightness-85 flex h-full cursor-pointer select-none items-center px-4 transition-colors duration-150 ease-in-out focus:border-none focus:outline-none"
                  :class="{
                    'bg-brand font-bold text-brand-inverted': selectedValue === item.option,
                    'bg-bg-raised': focusedOptionIndex === item.index,
                  }"
                  :aria-selected="selectedValue === item.option"
                  @click="selectOption(item.option, item.index)"
                  @mouseover="focusedOptionIndex = item.index"
                  @focus="focusedOptionIndex = item.index"
                >
                  <input
                    :id="`${name}-${item.index}`"
                    v-model="radioValue"
                    type="radio"
                    :value="item.option"
                    :name="name"
                    class="hidden"
                  />
                  <label :for="`${name}-${item.index}`" class="w-full cursor-pointer">
                    {{ displayName(item.option) }}
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts" generic="OptionValue extends string | number | Record<string, any>">
import { DropdownIcon } from '@modrinth/assets'
import { computed, ref, watch, onMounted, onUnmounted, nextTick } from 'vue'
import type { CSSProperties } from 'vue'

const ITEM_HEIGHT = 44
const BUFFER_ITEMS = 5

interface Props {
  options: OptionValue[]
  name: string
  defaultValue?: OptionValue | null
  placeholder?: string | number | null
  modelValue?: OptionValue | null
  renderUp?: boolean
  disabled?: boolean
  displayName?: (option: OptionValue) => string
}

const props = withDefaults(defineProps<Props>(), {
  defaultValue: null,
  placeholder: null,
  modelValue: null,
  renderUp: false,
  disabled: false,
  displayName: (option: OptionValue) => String(option),
})

const emit = defineEmits<{
  (e: 'input' | 'update:modelValue', value: OptionValue): void
  (e: 'change', value: { option: OptionValue; index: number }): void
}>()

const dropdownVisible = ref(false)
const selectedValue = ref<OptionValue | null>(props.modelValue || props.defaultValue)
const focusedOptionIndex = ref<number | null>(null)
const focusedOptionRef = ref<HTMLElement | null>(null)
const dropdown = ref<HTMLElement | null>(null)
const optionsContainer = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const isRenderingUp = ref(false)
const virtualListHeight = ref(300)
const lastFocusedElement = ref<HTMLElement | null>(null)

const positionStyle = ref<CSSProperties>({
  position: 'fixed',
  top: '0px',
  left: '0px',
  width: '0px',
  zIndex: 999,
})

const handleOptionRef = (el: HTMLElement | null, index: number) => {
  if (focusedOptionIndex.value === index) {
    focusedOptionRef.value = el
  }
}

const onFocus = async () => {
  if (!props.disabled) {
    focusedOptionIndex.value = props.options.findIndex((option) => option === selectedValue.value)
    lastFocusedElement.value = document.activeElement as HTMLElement
    dropdownVisible.value = true
    await updatePosition()
    nextTick(() => {
      dropdown.value?.focus()
    })
  }
}

const onBlur = (event: FocusEvent) => {
  if (!isChildOfDropdown(event.relatedTarget as HTMLElement | null)) {
    closeDropdown()
  }
}

const isChildOfDropdown = (element: HTMLElement | null): boolean => {
  let currentNode: HTMLElement | null = element
  while (currentNode) {
    if (currentNode === dropdown.value || currentNode === optionsContainer.value) {
      return true
    }
    currentNode = currentNode.parentElement
  }
  return false
}

const totalHeight = computed(() => props.options.length * ITEM_HEIGHT)

const visibleOptions = computed(() => {
  const startIndex = Math.floor(scrollTop.value / ITEM_HEIGHT) - BUFFER_ITEMS
  const visibleCount = Math.ceil(virtualListHeight.value / ITEM_HEIGHT) + 2 * BUFFER_ITEMS

  return Array.from({ length: visibleCount }, (_, i) => {
    const index = startIndex + i
    if (index >= 0 && index < props.options.length) {
      return {
        index,
        option: props.options[index],
      }
    }
    return null
  }).filter((item): item is { index: number; option: OptionValue } => item !== null)
})

const selectedOption = computed(() => {
  if (selectedValue.value !== null && selectedValue.value !== undefined) {
    return props.displayName(selectedValue.value as OptionValue)
  }
  return props.placeholder || 'Select an option'
})

const radioValue = computed<OptionValue>({
  get() {
    return props.modelValue ?? selectedValue.value ?? ''
  },
  set(newValue: OptionValue) {
    emit('update:modelValue', newValue)
    selectedValue.value = newValue
  },
})

const triggerClasses = computed(() => ({
  'cursor-not-allowed opacity-50 grayscale': props.disabled,
  'rounded-b-none': dropdownVisible.value && !isRenderingUp.value && !props.disabled,
  'rounded-t-none': dropdownVisible.value && isRenderingUp.value && !props.disabled,
}))

const updatePosition = async () => {
  if (!dropdown.value) return

  await nextTick()
  const triggerRect = dropdown.value.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const margin = 8

  const contentHeight = props.options.length * ITEM_HEIGHT
  const preferredHeight = Math.min(contentHeight, 300)

  const spaceBelow = viewportHeight - triggerRect.bottom
  const spaceAbove = triggerRect.top

  isRenderingUp.value = spaceBelow < preferredHeight && spaceAbove > spaceBelow

  virtualListHeight.value = isRenderingUp.value
    ? Math.min(spaceAbove - margin, preferredHeight)
    : Math.min(spaceBelow - margin, preferredHeight)

  positionStyle.value = {
    position: 'fixed',
    left: `${triggerRect.left}px`,
    width: `${triggerRect.width}px`,
    zIndex: 999,
    ...(isRenderingUp.value
      ? { bottom: `${viewportHeight - triggerRect.top}px`, top: 'auto' }
      : { top: `${triggerRect.bottom}px`, bottom: 'auto' }),
  }
}

const openDropdown = async () => {
  if (!props.disabled) {
    closeAllDropdowns()
    dropdownVisible.value = true
    focusedOptionIndex.value = props.options.findIndex((option) => option === selectedValue.value)
    lastFocusedElement.value = document.activeElement as HTMLElement
    await updatePosition()

    requestAnimationFrame(() => {
      updatePosition()
    })
  }
}

const toggleDropdown = () => {
  if (!props.disabled) {
    if (dropdownVisible.value) {
      closeDropdown()
    } else {
      openDropdown()
    }
  }
}

const handleResize = () => {
  if (dropdownVisible.value) {
    requestAnimationFrame(() => {
      updatePosition()
    })
  }
}

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement
  scrollTop.value = target.scrollTop
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (!dropdownVisible.value) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      lastFocusedElement.value = document.activeElement as HTMLElement
      toggleDropdown()
    }
  } else {
    handleDropdownKeyDown(event)
  }
}

const handleDropdownKeyDown = (event: KeyboardEvent) => {
  event.stopPropagation()

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      focusNextOption()
      break
    case 'ArrowUp':
      event.preventDefault()
      focusPreviousOption()
      break
    case 'Enter':
      event.preventDefault()
      if (focusedOptionIndex.value !== null) {
        selectOption(props.options[focusedOptionIndex.value], focusedOptionIndex.value)
      }
      break
    case 'Escape':
      event.preventDefault()
      event.stopPropagation()
      closeDropdown()
      break
    case 'Tab':
      event.preventDefault()
      if (event.shiftKey) {
        focusPreviousOption()
      } else {
        focusNextOption()
      }
      break
  }
}

const closeDropdown = () => {
  dropdownVisible.value = false
  focusedOptionIndex.value = null
  if (lastFocusedElement.value) {
    lastFocusedElement.value.focus()
    lastFocusedElement.value = null
  }
}

const closeAllDropdowns = () => {
  const event = new CustomEvent('close-all-dropdowns')
  window.dispatchEvent(event)
}

const selectOption = (option: OptionValue, index: number) => {
  radioValue.value = option
  emit('change', { option, index })
  closeDropdown()
}

const focusNextOption = () => {
  if (focusedOptionIndex.value === null) {
    focusedOptionIndex.value = 0
  } else {
    focusedOptionIndex.value = (focusedOptionIndex.value + 1) % props.options.length
  }
  scrollToFocused()
  nextTick(() => {
    focusedOptionRef.value?.focus()
  })
}

const focusPreviousOption = () => {
  if (focusedOptionIndex.value === null) {
    focusedOptionIndex.value = props.options.length - 1
  } else {
    focusedOptionIndex.value =
      (focusedOptionIndex.value - 1 + props.options.length) % props.options.length
  }
  scrollToFocused()
  nextTick(() => {
    focusedOptionRef.value?.focus()
  })
}

const scrollToFocused = () => {
  if (focusedOptionIndex.value === null) return

  const optionsElement = optionsContainer.value?.querySelector('.overflow-y-auto')
  if (!optionsElement) return

  const targetScrollTop = focusedOptionIndex.value * ITEM_HEIGHT
  const scrollBottom = optionsElement.clientHeight

  if (targetScrollTop < optionsElement.scrollTop) {
    optionsElement.scrollTop = targetScrollTop
  } else if (targetScrollTop + ITEM_HEIGHT > optionsElement.scrollTop + scrollBottom) {
    optionsElement.scrollTop = targetScrollTop - scrollBottom + ITEM_HEIGHT
  }
}

onMounted(() => {
  window.addEventListener('resize', handleResize)
  window.addEventListener('scroll', handleResize, true)
  window.addEventListener('click', (event) => {
    if (!isChildOfDropdown(event.target as HTMLElement)) {
      closeDropdown()
    }
  })
  window.addEventListener('close-all-dropdowns', closeDropdown)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  window.removeEventListener('scroll', handleResize, true)
  window.removeEventListener('click', (event) => {
    if (!isChildOfDropdown(event.target as HTMLElement)) {
      closeDropdown()
    }
  })
  window.removeEventListener('close-all-dropdowns', closeDropdown)
  lastFocusedElement.value = null
})

watch(
  () => props.modelValue,
  (newValue) => {
    selectedValue.value = newValue
  },
)

watch(dropdownVisible, async (newValue) => {
  if (newValue) {
    await updatePosition()
    scrollTop.value = 0
  }
})
</script>
