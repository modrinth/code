<template>
  <div data-pyro-telepopover-wrapper class="relative">
    <button
      ref="triggerRef"
      :aria-expanded="isOpen"
      :aria-haspopup="true"
      @mousedown="handleMouseDown"
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
          class="fixed isolate z-[9999] flex w-fit flex-col gap-2 overflow-hidden rounded-2xl border-[1px] border-solid border-button-bg bg-bg-raised p-2 shadow-lg"
          :style="menuStyle"
          role="menu"
          tabindex="-1"
          @mousedown.stop
        >
          <ButtonStyled
            v-for="(option, index) in filteredOptions"
            :key="option.id"
            type="transparent"
            role="menuitem"
            :color="option.color"
          >
            <button
              :ref="
                (el) => {
                  if (el) menuItemsRef[index] = el as HTMLElement;
                }
              "
              class="w-full !justify-start focus-visible:!outline-none"
              :aria-selected="index === selectedIndex"
              :style="index === selectedIndex ? { background: 'var(--color-button-bg)' } : {}"
              @click="handleItemClick(option, index)"
              @focus="selectedIndex = index"
              @mouseover="handleMouseOver(index)"
            >
              <slot :name="option.id">{{ option.id }}</slot>
            </button>
          </ButtonStyled>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from "vue";
import { onClickOutside } from "@vueuse/core";

interface Option {
  id: string;
  action: () => void;
  shown?: boolean;
  color?: "standard" | "brand" | "red" | "orange" | "green" | "blue" | "purple";
}

const props = defineProps<{
  options: Option[];
  position?: "top" | "bottom";
  direction?: "left" | "right";
}>();

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

const menuStyle = ref({
  top: "0px",
  left: "0px",
});

const filteredOptions = computed(() => props.options.filter((option) => option.shown !== false));

const calculateMenuPosition = () => {
  if (!triggerRef.value) return { top: "0px", left: "0px" };

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const menuWidth = 140;
  const menuHeight = filteredOptions.value.length * 50;
  const margin = 8;

  let top: number;
  let left: number;

  if (triggerRect.top + triggerRect.height + menuHeight <= window.innerHeight - margin) {
    top = triggerRect.bottom + 8;
  } else if (triggerRect.top - menuHeight >= margin) {
    top = triggerRect.top - menuHeight - 8;
  } else {
    top = window.innerHeight - menuHeight - margin - 8;
  }

  if (triggerRect.left + menuWidth <= window.innerWidth - margin) {
    left = triggerRect.left;
  } else {
    left = triggerRect.right - menuWidth;
  }

  left = Math.max(margin, Math.min(left, window.innerWidth - menuWidth - margin));
  top = Math.max(margin, Math.min(top, window.innerHeight - menuHeight - margin));

  return {
    top: `${top}px`,
    left: `${left}px`,
  };
};

const toggleMenu = (event: MouseEvent) => {
  event.stopPropagation();
  if (isOpen.value) {
    closeMenu();
  } else {
    openMenu();
  }
};

const openMenu = () => {
  menuStyle.value = calculateMenuPosition();
  isOpen.value = true;
  disableBodyScroll();
  nextTick(() => {
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
  option.action();
  closeMenu();
};

const handleMouseDown = (event: MouseEvent) => {
  event.preventDefault();
  isMouseDown.value = true;
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
  menuItemsRef.value[selectedIndex.value].focus();
};

// scrolling is disabled for keyboard navigation
const disableBodyScroll = () => {
  // make opening not shift page when there's a vertical scrollbar
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
    menuItemsRef.value[0].focus();
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
      menuItemsRef.value[selectedIndex.value].focus();
      break;
    case "ArrowUp":
      event.preventDefault();
      selectedIndex.value =
        (selectedIndex.value - 1 + filteredOptions.value.length) % filteredOptions.value.length;
      menuItemsRef.value[selectedIndex.value].focus();
      break;
    case "Home":
      event.preventDefault();
      selectedIndex.value = 0;
      menuItemsRef.value[selectedIndex.value].focus();
      break;
    case "End":
      event.preventDefault();
      selectedIndex.value = filteredOptions.value.length - 1;
      menuItemsRef.value[selectedIndex.value].focus();
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
      triggerRef.value?.focus();
      break;
    case "Tab":
      event.preventDefault();
      if (event.shiftKey) {
        selectedIndex.value =
          (selectedIndex.value - 1 + filteredOptions.value.length) % filteredOptions.value.length;
      } else {
        selectedIndex.value = (selectedIndex.value + 1) % filteredOptions.value.length;
      }
      menuItemsRef.value[selectedIndex.value].focus();
      break;
    default:
      if (event.key.length === 1) {
        typeAheadBuffer.value += event.key.toLowerCase();
        const matchIndex = filteredOptions.value.findIndex((option) =>
          option.id.toLowerCase().startsWith(typeAheadBuffer.value),
        );
        if (matchIndex !== -1) {
          selectedIndex.value = matchIndex;
          menuItemsRef.value[selectedIndex.value].focus();
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

onMounted(() => {
  triggerRef.value?.addEventListener("keydown", handleKeydown);
  window.addEventListener("resize", () => {
    if (isOpen.value) {
      menuStyle.value = calculateMenuPosition();
    }
  });
  window.addEventListener("scroll", () => {
    if (isOpen.value) {
      menuStyle.value = calculateMenuPosition();
    }
  });
});

onUnmounted(() => {
  triggerRef.value?.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("resize", () => {
    if (isOpen.value) {
      menuStyle.value = calculateMenuPosition();
    }
  });
  window.removeEventListener("scroll", () => {
    if (isOpen.value) {
      menuStyle.value = calculateMenuPosition();
    }
  });
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
