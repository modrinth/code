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
      <template v-for="(option, index) in options">
        <div v-if="option.divider" :key="`divider-${index}`" class="card-divider"></div>
        <Button
          v-else
          :key="`option-${option.id}`"
          :color="option.color ? option.color : 'default'"
          :hover-filled="option.hoverFilled"
          :hover-filled-only="option.hoverFilledOnly"
          transparent
          :action="
            option.action
              ? () => {
                  option.action()
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

<script setup>
import { ref } from 'vue'
import Button from './Button.vue'
import PopoutMenu from './PopoutMenu.vue'

defineProps({
  options: {
    type: Array,
    required: true,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  position: {
    type: String,
    default: 'bottom',
  },
  direction: {
    type: String,
    default: 'left',
  },
})
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
