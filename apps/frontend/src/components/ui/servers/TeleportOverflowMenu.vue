<template>
  <div data-pyro-telepopover-wrapper class="relative">
    <button
      v-if="!isContextMenu"
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
          <ButtonStyled
            v-for="(option, index) in filteredOptions"
            :key="option.id"
            type="transparent"
            role="menuitem"
            :color="option.color"
          >
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
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from "vue";
import { onClickOutside, useElementHover } from "@vueuse/core";

export interface Option {
  id: string;
  action?: (() => void) | string;
  shown?: boolean;
  color?: "standard" | "brand" | "red" | "orange" | "green" | "blue" | "purple";
}

interface Props {
  options: Option[];
  hoverable?: boolean;
  isContextMenu?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  hoverable: false,
  isContextMenu: false,
});

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
const contextPosition = ref({ x: 0, y: 0 });

const hoveringTrigger = useElementHover(triggerRef);
const hoveringMenu = useElementHover(menuRef);

const hovering = computed(() => hoveringTrigger.value || hoveringMenu.value);

const menuStyle = ref({
  top: "0px",
  left: "0px",
});

const menuDimensions = ref({ width: 0, height: 0 });
const initialRender = ref(true);

const filteredOptions = computed(() => props.options.filter((option) => option.shown !== false));

const calculateMenuPosition = () => {
  const margin = 8;
  const safeMargin = margin * 2;

  if (props.isContextMenu) {
    let x = contextPosition.value.x;
    let y = contextPosition.value.y;

    const menuWidth = initialRender.value
      ? menuDimensions.value.width
      : menuRef.value?.getBoundingClientRect().width ?? 0;
    const menuHeight = initialRender.value
      ? menuDimensions.value.height
      : menuRef.value?.getBoundingClientRect().height ?? 0;

    if (x + menuWidth > window.innerWidth - margin) {
      x = Math.max(margin, window.innerWidth - menuWidth - safeMargin);
    }

    if (y + menuHeight > window.innerHeight - margin) {
      y = Math.max(margin, window.innerHeight - menuHeight - safeMargin);
    }

    x = Math.min(Math.max(margin, x), window.innerWidth - menuWidth - safeMargin);
    y = Math.min(Math.max(margin, y), window.innerHeight - menuHeight - safeMargin);

    return {
      top: `${y}px`,
      left: `${x}px`,
    };
  }

  if (!triggerRef.value || !menuRef.value) return { top: "0px", left: "0px" };

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const menuRect = menuRef.value.getBoundingClientRect();
  const menuWidth = menuRect.width;
  const menuHeight = menuRect.height;

  let top: number;
  let left: number;

  // okay gang lets calculate this shit
  // from the top now yall
  // y
  // now with even better calcs
  if (triggerRect.bottom + menuHeight <= window.innerHeight - margin) {
    top = triggerRect.bottom + margin;
  } else if (triggerRect.top - menuHeight >= margin) {
    top = triggerRect.top - menuHeight - margin;
  } else {
    top = Math.max(margin, Math.min(window.innerHeight - menuHeight - margin, triggerRect.top));
  }

  if (triggerRect.left + menuWidth <= window.innerWidth - margin) {
    left = triggerRect.left;
  } else if (triggerRect.right - menuWidth >= margin) {
    left = triggerRect.right - menuWidth;
  } else {
    left = Math.max(margin, Math.min(window.innerWidth - menuWidth - margin, triggerRect.left));
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
        selectOption(filteredOptions.value[selectedIndex.value]);
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
        const matchIndex = filteredOptions.value.findIndex((option) =>
          option.id.toLowerCase().startsWith(typeAheadBuffer.value),
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

watch(
  filteredOptions,
  () => {
    if (menuRef.value && isOpen.value) {
      const rect = menuRef.value.getBoundingClientRect();
      menuDimensions.value = {
        width: rect.width,
        height: rect.height,
      };
      menuStyle.value = calculateMenuPosition();
    }
  },
  { deep: true },
);

onClickOutside(menuRef, (event) => {
  if (!triggerRef.value?.contains(event.target as Node)) {
    closeMenu();
  }
});

const openContextMenu = (x: number, y: number) => {
  contextPosition.value = { x, y };
  initialRender.value = true;
  isOpen.value = true;
  disableBodyScroll();

  nextTick(() => {
    if (menuRef.value) {
      const rect = menuRef.value.getBoundingClientRect();
      menuDimensions.value = {
        width: rect.width,
        height: rect.height,
      };
      menuStyle.value = calculateMenuPosition();

      setTimeout(() => {
        initialRender.value = false;
        menuStyle.value = calculateMenuPosition();
      }, 150);
    }
    document.addEventListener("mousemove", handleMouseMove);
    focusFirstMenuItem();
  });
};

defineExpose({
  openContextMenu,
  close: closeMenu,
});
</script>
