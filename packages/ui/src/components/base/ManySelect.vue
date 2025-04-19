<template>
  <ButtonStyled>
    <PopoutMenu
      v-if="options.length > 1 || showAlways"
      v-bind="$attrs"
      :disabled="disabled"
      :position="position"
      :direction="direction"
      :dropdown-id="dropdownId"
      :dropdown-class="dropdownClass"
      :tooltip="tooltip"
      @open="
        () => {
          searchQuery = ''
        }
      "
    >
      <slot />
      <DropdownIcon class="h-5 w-5 text-secondary" />
      <template #menu>
        <div v-if="search" class="iconified-input mb-2 w-full">
          <label for="search-input" hidden>Search...</label>
          <SearchIcon aria-hidden="true" />
          <input
            id="search-input"
            ref="searchInput"
            v-model="searchQuery"
            placeholder="Search..."
            type="text"
            @keydown.enter="
              () => {
                toggleOption(filteredOptions[0])
              }
            "
          />
        </div>
        <ScrollablePanel v-if="search" class="h-[17rem]">
          <Button
            v-for="(option, index) in filteredOptions"
            :key="`option-${index}`"
            :transparent="!manyValues.includes(option)"
            :action="() => toggleOption(option)"
            class="!w-full"
            :color="manyValues.includes(option) ? 'secondary' : 'default'"
          >
            <slot name="option" :option="option">{{ getOptionLabel(option) }}</slot>
            <CheckIcon
              class="h-5 w-5 text-contrast ml-auto transition-opacity"
              :class="{ 'opacity-0': !manyValues.includes(option) }"
            />
          </Button>
        </ScrollablePanel>
        <div v-else class="flex flex-col gap-1">
          <Button
            v-for="(option, index) in filteredOptions"
            :key="`option-${index}`"
            :transparent="!manyValues.includes(option)"
            :action="() => toggleOption(option)"
            class="!w-full"
            :color="manyValues.includes(option) ? 'secondary' : 'default'"
          >
            <slot name="option" :option="option">{{ getOptionLabel(option) }}</slot>
            <CheckIcon
              class="h-5 w-5 text-contrast ml-auto transition-opacity"
              :class="{ 'opacity-0': !manyValues.includes(option) }"
            />
          </Button>
        </div>
        <slot name="footer" />
      </template>
    </PopoutMenu>
  </ButtonStyled>
</template>
<script setup lang="ts">
import { CheckIcon, DropdownIcon, SearchIcon } from '@modrinth/assets'
import { ButtonStyled, PopoutMenu, Button } from '../index'
import { computed, ref } from 'vue'
import ScrollablePanel from './ScrollablePanel.vue'

type Option = string | number | object

const props = withDefaults(
  defineProps<{
    modelValue: Option[]
    options: Option[]
    disabled?: boolean
    position?: string
    direction?: string
    displayName?: (option: Option) => string
    search?: boolean
    dropdownId?: string
    dropdownClass?: string
    showAlways?: boolean
    tooltip?: string
  }>(),
  {
    disabled: false,
    position: 'auto',
    direction: 'auto',
    displayName: undefined,
    search: false,
    dropdownId: '',
    dropdownClass: '',
    showAlways: false,
    tooltip: '',
  },
)

function getOptionLabel(option: Option): string {
  return props.displayName?.(option) ?? (option as string)
}

const emit = defineEmits(['update:modelValue', 'change'])
const selectedValues = ref(props.modelValue || [])
const searchInput = ref()

const searchQuery = ref('')

const manyValues = computed({
  get() {
    return props.modelValue || selectedValues.value
  },
  set(newValue) {
    emit('update:modelValue', newValue)
    emit('change', newValue)
    selectedValues.value = newValue
  },
})

const filteredOptions = computed(() => {
  return props.options.filter(
    (x) =>
      !searchQuery.value ||
      getOptionLabel(x).toLowerCase().includes(searchQuery.value.toLowerCase()),
  )
})

defineOptions({
  inheritAttrs: false,
})

function toggleOption(id: Option) {
  if (manyValues.value.includes(id)) {
    manyValues.value = manyValues.value.filter((x) => x !== id)
  } else {
    manyValues.value = [...manyValues.value, id]
  }
}
</script>
