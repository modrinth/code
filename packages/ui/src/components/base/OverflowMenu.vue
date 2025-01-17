<template>
  <PopoutMenu
    ref="dropdown"
    v-bind="$attrs"
    :disabled="disabled"
    :dropdown-id="dropdownId"
    :tooltip="tooltip"
  >
    <slot></slot>
    <template #menu>
      <template v-for="(option, index) in options.filter((x) => x.shown === undefined || x.shown)">
        <div
          v-if="isDivider(option)"
          :key="`divider-${index}`"
          class="h-px mx-3 my-2 bg-button-bg"
        ></div>
        <Button
          v-else
          :key="`option-${option.id}`"
          v-tooltip="option.tooltip"
          :color="option.color ? option.color : 'default'"
          :hover-filled="option.hoverFilled"
          :hover-filled-only="option.hoverFilledOnly"
          transparent
          :v-close-popper="!option.remainOnClick"
          :action="
            option.action
              ? (event: MouseEvent) => {
                  option.action?.(event)
                  if (!option.remainOnClick) {
                    close()
                  }
                }
              : undefined
          "
          :link="option.link ? option.link : undefined"
          :external="option.external ? option.external : false"
          :disabled="option.disabled"
          @click="
            () => {
              if (option.link && !option.remainOnClick) {
                close()
              }
            }
          "
        >
          <template v-if="!$slots[option.id]">{{ option.id }}</template>
          <slot :name="option.id"></slot>
        </Button>
      </template>
    </template>
  </PopoutMenu>
</template>

<script setup lang="ts">
import { type Ref, ref } from 'vue'
import Button from './Button.vue'
import PopoutMenu from './PopoutMenu.vue'

interface BaseOption {
  shown?: boolean
}

interface Divider extends BaseOption {
  divider?: boolean
}

interface Item extends BaseOption {
  id: string
  action?: (event?: MouseEvent) => void
  link?: string
  external?: boolean
  color?:
    | 'primary'
    | 'danger'
    | 'secondary'
    | 'highlight'
    | 'red'
    | 'orange'
    | 'green'
    | 'blue'
    | 'purple'
  hoverFilled?: boolean
  hoverFilledOnly?: boolean
  remainOnClick?: boolean
  disabled?: boolean
  tooltip?: string
}

type Option = Divider | Item

withDefaults(
  defineProps<{
    options: Option[]
    disabled?: boolean
    dropdownId?: string
    tooltip?: string
  }>(),
  {
    options: () => [],
    disabled: false,
    dropdownId: undefined,
    tooltip: undefined,
  },
)

defineOptions({
  inheritAttrs: false,
})

const dropdown: Ref<InstanceType<typeof PopoutMenu> | null> = ref(null)

const close = () => {
  dropdown.value?.hide()
}

const open = () => {
  dropdown.value?.show()
}

function isDivider(option: BaseOption): option is Divider {
  return 'divider' in option
}

defineExpose({ open, close })
</script>

<style lang="scss" scoped>
.btn {
  white-space: nowrap;
  width: 100%;
  box-shadow: none;
  --text-color: var(--color-base);
  --background-color: transparent;
  justify-content: flex-start;

  &:not(:last-child) {
    margin-bottom: var(--gap-xs);
  }
}
</style>
