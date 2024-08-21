<template>
  <PopoutMenu
    ref="dropdown"
    v-bind="$attrs"
    :disabled="disabled"
    :position="position"
    :direction="direction"
  >
    <slot></slot>
    <template #menu>
      <template v-for="(option, index) in options.filter((x) => x.shown === undefined || x.shown)">
        <div
          v-if="option.divider"
          :key="`divider-${index}`"
          class="h-px mx-3 my-2 bg-button-bg"
        ></div>
        <Button
          v-else
          :key="`option-${option.id}`"
          :color="option.color ? option.color : 'default'"
          :hover-filled="option.hoverFilled"
          :hover-filled-only="option.hoverFilledOnly"
          transparent
          :action="
            option.action
              ? (event) => {
                  option.action(event)
                  if (!option.remainOnClick) {
                    close()
                  }
                }
              : null
          "
          :link="option.link ? option.link : null"
          :external="option.external ? option.external : false"
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
import { ref } from 'vue'
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
  action?: () => void
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
}

type Option = Divider | Item

const props = withDefaults(
  defineProps<{
    options: Option[]
    disabled?: boolean
    position?: string
    direction?: string
  }>(),
  {
    options: () => [],
    disabled: false,
    position: 'auto',
    direction: 'auto',
  },
)

defineOptions({
  inheritAttrs: false,
})

const dropdown = ref(null)

const close = () => {
  console.log('closing!')
  dropdown.value.hide()
}
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
