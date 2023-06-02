<template>
  <transition name="fade">
    <ul :id="elementId" ref="contextMenu" class="vue-simple-context-menu">
      <li
        v-for="(option, index) in options"
        :key="index"
        class="vue-simple-context-menu__item"
        :class="[
          option.name === 'divider' ? 'vue-simple-context-menu__divider' : '',
          option.color ?? 'base',
        ]"
        @click.stop="optionClicked(option.name)"
      >
        <slot v-if="option.name !== 'divider'" :name="option.name" />
      </li>
    </ul>
  </transition>
</template>

<script setup>
import {onBeforeUnmount, onMounted, ref} from "vue";

const props = defineProps({
  elementId: {
    type: String,
    required: true,
  },
})

const emit = defineEmits(['menu-closed', 'option-clicked'])

const item = ref(null)
const menuHeight = ref(null)
const menuWidth = ref(null)
const contextMenu = ref(null)
const options = ref([])

defineExpose({
  showMenu: (event, passedItem, passedOptions) => {
  item.value = passedItem;
  options.value = passedOptions;

  var menu = document.getElementById(props.elementId);
  if (!menu) {
    return;
  }

  if (!menuWidth.value || !menuHeight.value) {
    menu.style.visibility = 'hidden';
    menu.style.display = 'block';
    menuWidth.value = menu.offsetWidth;
    menuHeight.value = menu.offsetHeight;
    menu.removeAttribute('style');
  }

  if (menuWidth.value + event.pageX >= window.innerWidth) {
    menu.style.left = event.pageX - menuWidth.value + 2 + 'px';
  } else {
    menu.style.left = event.pageX - 2 + 'px';
  }

  if (menuHeight.value + event.pageY >= window.innerHeight) {
    menu.style.top = event.pageY - menuHeight.value + 2 + 'px';
  } else {
    menu.style.top = event.pageY - 2 + 'px';
  }

  menu.classList.add('vue-simple-context-menu--active');
}})

const hideContextMenu = () => {
  const element = document.getElementById(props.elementId);
  if (element) {
    element.classList.remove('vue-simple-context-menu--active');
    emit('menu-closed');
  }
}

const optionClicked = (option) => {
  console.log('item check', item.value)
  emit('option-clicked', {
    item: item.value,
    option: option,
  });
  hideContextMenu();
}

const onClickOutside = () => {
  hideContextMenu();
}

const onEscKeyRelease = (event) => {
  if (event.keyCode === 27) {
    hideContextMenu();
  }
}

const handleClickOutside = (event) => {
  const elements = document.elementsFromPoint(event.clientX, event.clientY)
  if (
    contextMenu.value &&
    contextMenu.value.$el !== event.target &&
    !elements.includes(contextMenu.value.$el)
  ) {
    onClickOutside()
  }
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
  document.body.addEventListener('keyup', onEscKeyRelease);
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keyup', onEscKeyRelease);
})

</script>

<style lang="scss">

.vue-simple-context-menu {
  background-color: var(--color-raised-bg);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-floating);
  border: 1px solid var(--color-button-bg);
  display: none;
  left: 0;
  list-style: none;
  margin: 0;
  position: fixed;
  top: 0;
  z-index: 1000000;
  overflow: hidden;
  padding: var(--gap-sm);

  &--active {
    display: block;
  }

  &__item {
    align-items: center;
    color: var(--color-base);
    cursor: pointer;
    display: flex;
    gap: var(--gap-sm);
    padding: var(--gap-sm);
    border-radius: var(--radius-sm);

    &:hover {
      &.base {
        background-color: var(--color-button-bg);
        color: var(--color-contrast);
      }

      &.primary {
        background-color: var(--color-brand);
        color: var(--color-accent-contrast);
      }

      &.danger {
        background-color: var(--color-red);
        color: var(--color-accent-contrast);
      }
    }
  }

  &__divider {
    background-clip: content-box;
    background-color: var(--color-button-bg);
    box-sizing: content-box;
    height: 1px;
    padding: var(--gap-sm);
    pointer-events: none;
  }
}


.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
