<template>
  <div data-pyro-telepopover-wrapper class="relative">
    <button
      ref="triggerRef"
      class="teleport-overflow-menu-trigger"
      :aria-expanded="isOpen"
      :aria-haspopup="true"
      @mousedown="handleMouseDown"
      @mouseenter="handleMouseEnter"
      @mouseleave="handleMouseLeave"
      @click="toggleMenu"
    >
      <slot></slot>
    </button>
    <Teleport to="#teleports">
      <Transition
        enter-active-class="transition duration-125 ease-out"
        enter-from-class="transform scale-75 opacity-0"
        enter-to-class="transform scale-100 opacity-100"
        leave-active-class="transition duration-125 ease-in"
        leave-from-class="transform scale-100 opacity-100"
        leave-to-class="transform scale-75 opacity-0"
      >
        <div
          v-if="isOpen"
          ref="menuRef"
          data-pyro-telepopover-root
          class="experimental-styles-within fixed isolate z-[9999] flex w-fit flex-col gap-2 overflow-hidden rounded-2xl border-[1px] border-solid border-divider bg-bg-raised p-2 shadow-lg"
          :style="menuStyle"
          role="menu"
          tabindex="-1"
          @mousedown.stop
          @mouseleave="handleMouseLeave"
        >
          <template
            v-for="(option, index) in filteredOptions"
            :key="isDivider(option) ? `divider-${index}` : option.id"
          >
            <div v-if="isDivider(option)" class="h-px w-full bg-button-bg"></div>
            <ButtonStyled v-else type="transparent" role="menuitem" :color="option.color">
              <button
                v-if="typeof option.action === 'function'"
                :ref="
                  (el) => {
                    if (el) menuItemsRef[index] = el as HTMLElement;
                  }
                "
                class="w-full !justify-start !whitespace-nowrap focus-visible:!outline-none"
                :aria-selected="index === selectedIndex"
                :style="index === selectedIndex ? { background: 'var(--color-button-bg)' } : {}"
                @click="handleItemClick(option, index)"
                @focus="selectedIndex = index"
                @mouseover="handleMouseOver(index)"
              >
                <slot :name="option.id">{{ option.id }}</slot>
              </button>
              <nuxt-link
                v-else-if="typeof option.action === 'string' && option.action.startsWith('/')"
                :ref="
                  (el) => {
                    if (el) menuItemsRef[index] = el as HTMLElement;
                  }
                "
                :to="option.action"
                class="w-full !justify-start !whitespace-nowrap focus-visible:!outline-none"
                :aria-selected="index === selectedIndex"
                :style="index === selectedIndex ? { background: 'var(--color-button-bg)' } : {}"
                @click="handleItemClick(option, index)"
                @focus="selectedIndex = index"
                @mouseover="handleMouseOver(index)"
              >
                <slot :name="option.id">{{ option.id }}</slot>
              </nuxt-link>
              <a
                v-else-if="typeof option.action === 'string' && !option.action.startsWith('http')"
                :ref="
                  (el) => {
                    if (el) menuItemsRef[index] = el as HTMLElement;
                  }
                "
                :href="option.action"
                target="_blank"
                class="w-full !justify-start !whitespace-nowrap focus-visible:!outline-none"
                :aria-selected="index === selectedIndex"
                :style="index === selectedIndex ? { background: 'var(--color-button-bg)' } : {}"
                @click="handleItemClick(option, index)"
                @focus="selectedIndex = index"
                @mouseover="handleMouseOver(index)"
              >
                <slot :name="option.id">{{ option.id }}</slot>
              </a>
              <span v-else>
                <slot :name="option.id">{{ option.id }}</slot>
              </span>
            </ButtonStyled>
          </template>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from "vue";
import { onClickOutside, useElementHover } from "@vueuse/core";

interface Option {
  id: string;
  action?: (() => void) | string;
  shown?: boolean;
  color?: "standard" | "brand" | "red" | "orange" | "green" | "blue" | "purple";
}

type Divider = {
  divider: true;
  shown?: boolean;
};

type Item = Option | Divider;

function isDivider(item: Item): item is Divider {
  return (item as Divider).divider;
}

const props = withDefaults(
  defineProps<{
    options: Item[];
    hoverable?: boolean;
  }>(),
  {
    hoverable: false,
  },
);

const emit = defineEmits<{
  (e: "select", option: Option): void;
}>();

const isOpen = ref(false);
const selectedIndex = ref(-1);
const menuRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const isMouseDown = ref(false);
const typeAheadBuffer = ref("");
const typeAheadTimeout = ref<number | null>(null);
const menuItemsRef = ref<HTMLElement[]>([]);

const hoveringTrigger = useElementHover(triggerRef);
const hoveringMenu = useElementHover(menuRef);

const hovering = computed(() => hoveringTrigger.value || hoveringMenu.value);

const menuStyle = ref({
  top: "0px",
  left: "0px",
});

const filteredOptions = computed(() => props.options.filter((option) => option.shown !== false));

const calculateMenuPosition = () => {
  if (!triggerRef.value || !menuRef.value) return { top: "0px", left: "0px" };

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const menuRect = menuRef.value.getBoundingClientRect();
  const menuWidth = menuRect.width;
  const menuHeight = menuRect.height;
  const margin = 8;

  let top: number;
  let left: number;

  // okay gang lets calculate this shit
  // from the top now yall
  // y
  if (triggerRect.bottom + menuHeight + margin <= window.innerHeight) {
    top = triggerRect.bottom + margin;
  } else if (triggerRect.top - menuHeight - margin >= 0) {
    top = triggerRect.top - menuHeight - margin;
  } else {
    top = Math.max(margin, window.innerHeight - menuHeight - margin);
  }

  // x
  if (triggerRect.left + menuWidth + margin <= window.innerWidth) {
    left = triggerRect.left;
  } else if (triggerRect.right - menuWidth - margin >= 0) {
    left = triggerRect.right - menuWidth;
  } else {
    left = Math.max(margin, window.innerWidth - menuWidth - margin);
  }

  return {
    top: `${top}px`,
    left: `${left}px`,
  };
};

const toggleMenu = (event: MouseEvent) => {
  event.stopPropagation();
  if (!props.hoverable) {
    if (isOpen.value) {
      closeMenu();
    } else {
      openMenu();
    }
  }
};

const openMenu = () => {
  isOpen.value = true;
  disableBodyScroll();
  nextTick(() => {
    menuStyle.value = calculateMenuPosition();
    document.addEventListener("mousemove", handleMouseMove);
    focusFirstMenuItem();
  });
};

const closeMenu = () => {
  isOpen.value = false;
  selectedIndex.value = -1;
  enableBodyScroll();
  document.removeEventListener("mousemove", handleMouseMove);
};

const selectOption = (option: Option) => {
  emit("select", option);
  if (typeof option.action === "function") {
    option.action();
  }
  closeMenu();
};

const handleMouseDown = (event: MouseEvent) => {
  event.preventDefault();
  isMouseDown.value = true;
};

const handleMouseEnter = () => {
  if (props.hoverable) {
    openMenu();
  }
};

const handleMouseLeave = () => {
  if (props.hoverable) {
    setTimeout(() => {
      if (!hovering.value) {
        closeMenu();
      }
    }, 250);
  }
};

const handleMouseMove = (event: MouseEvent) => {
  if (!isOpen.value || !isMouseDown.value) return;

  const menuRect = menuRef.value?.getBoundingClientRect();
  if (!menuRect) return;

  const menuItems = menuRef.value?.querySelectorAll('[role="menuitem"]');
  if (!menuItems) return;

  for (let i = 0; i < menuItems.length; i++) {
    const itemRect = (menuItems[i] as HTMLElement).getBoundingClientRect();
    if (
      event.clientX >= itemRect.left &&
      event.clientX <= itemRect.right &&
      event.clientY >= itemRect.top &&
      event.clientY <= itemRect.bottom
    ) {
      selectedIndex.value = i;
      break;
    }
  }
};

const handleItemClick = (option: Option, index: number) => {
  selectedIndex.value = index;
  selectOption(option);
};

const handleMouseOver = (index: number) => {
  selectedIndex.value = index;
  menuItemsRef.value[selectedIndex.value].focus?.();
};

// Scrolling is disabled for keyboard navigation
const disableBodyScroll = () => {
  // Make opening not shift page when there's a vertical scrollbar
  const scrollBarWidth = window.innerWidth - document.documentElement.clientWidth;
  if (scrollBarWidth > 0) {
    document.body.style.paddingRight = `${scrollBarWidth}px`;
  } else {
    document.body.style.paddingRight = "";
  }

  document.body.style.overflow = "hidden";
};

const enableBodyScroll = () => {
  document.body.style.paddingRight = "";
  document.body.style.overflow = "";
};

const focusFirstMenuItem = () => {
  if (menuItemsRef.value.length > 0) {
    menuItemsRef.value[0].focus?.();
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (!isOpen.value) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      openMenu();
    }
    return;
  }

  switch (event.key) {
    case "ArrowDown":
      event.preventDefault();
      selectedIndex.value = (selectedIndex.value + 1) % filteredOptions.value.length;
      menuItemsRef.value[selectedIndex.value].focus?.();
      break;
    case "ArrowUp":
      event.preventDefault();
      selectedIndex.value =
        (selectedIndex.value - 1 + filteredOptions.value.length) % filteredOptions.value.length;
      menuItemsRef.value[selectedIndex.value].focus?.();
      break;
    case "Home":
      event.preventDefault();
      if (menuItemsRef.value.length > 0) {
        selectedIndex.value = 0;
        menuItemsRef.value[selectedIndex.value].focus?.();
      }
      break;
    case "End":
      event.preventDefault();
      if (menuItemsRef.value.length > 0) {
        selectedIndex.value = filteredOptions.value.length - 1;
        menuItemsRef.value[selectedIndex.value].focus?.();
      }
      break;
    case "Enter":
    case " ":
      event.preventDefault();
      if (selectedIndex.value >= 0) {
        const option = filteredOptions.value[selectedIndex.value];
        if (isDivider(option)) break;
        selectOption(option);
      }
      break;
    case "Escape":
      event.preventDefault();
      closeMenu();
      triggerRef.value?.focus?.();
      break;
    case "Tab":
      event.preventDefault();
      if (menuItemsRef.value.length > 0) {
        if (event.shiftKey) {
          selectedIndex.value =
            (selectedIndex.value - 1 + filteredOptions.value.length) % filteredOptions.value.length;
        } else {
          selectedIndex.value = (selectedIndex.value + 1) % filteredOptions.value.length;
        }
        menuItemsRef.value[selectedIndex.value].focus?.();
      }
      break;
    default:
      if (event.key.length === 1) {
        typeAheadBuffer.value += event.key.toLowerCase();
        const matchIndex = filteredOptions.value.findIndex(
          (option) =>
            !isDivider(option) && option.id.toLowerCase().startsWith(typeAheadBuffer.value),
        );
        if (matchIndex !== -1) {
          selectedIndex.value = matchIndex;
          menuItemsRef.value[selectedIndex.value].focus?.();
        }
        if (typeAheadTimeout.value) {
          clearTimeout(typeAheadTimeout.value);
        }
        typeAheadTimeout.value = setTimeout(() => {
          typeAheadBuffer.value = "";
        }, 1000) as unknown as number;
      }
      break;
  }
};

const handleResizeOrScroll = () => {
  if (isOpen.value) {
    menuStyle.value = calculateMenuPosition();
  }
};

const throttle = (func: (...args: any[]) => void, limit: number): ((...args: any[]) => void) => {
  let inThrottle: boolean;
  return function (...args: any[]) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
};

const throttledHandleResizeOrScroll = throttle(handleResizeOrScroll, 100);

onMounted(() => {
  triggerRef.value?.addEventListener("keydown", handleKeydown);
  window.addEventListener("resize", throttledHandleResizeOrScroll);
  window.addEventListener("scroll", throttledHandleResizeOrScroll);
});

onUnmounted(() => {
  triggerRef.value?.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("resize", throttledHandleResizeOrScroll);
  window.removeEventListener("scroll", throttledHandleResizeOrScroll);
  document.removeEventListener("mousemove", handleMouseMove);
  if (typeAheadTimeout.value) {
    clearTimeout(typeAheadTimeout.value);
  }
  enableBodyScroll();
});

watch(isOpen, (newValue) => {
  if (newValue) {
    nextTick(() => {
      menuRef.value?.addEventListener("keydown", handleKeydown);
    });
  } else {
    menuRef.value?.removeEventListener("keydown", handleKeydown);
  }
});

onClickOutside(menuRef, (event) => {
  if (!triggerRef.value?.contains(event.target as Node)) {
    closeMenu();
  }
});
</script>
