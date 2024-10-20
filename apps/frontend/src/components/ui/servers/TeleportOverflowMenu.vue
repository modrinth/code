<template>
  <div data-pyro-telepopover-wrapper class="relative">
    <button
      ref="triggerRef"
      :aria-expanded="isOpen"
      :aria-haspopup="true"
      @mousedown="handleMouseDown"
      @click="handleClick"
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
          class="fixed isolate z-[9999] flex w-fit flex-col gap-2 overflow-hidden rounded-xl border-[1px] border-solid border-button-bg bg-bg-raised p-2 shadow-lg"
          :style="menuStyle"
          role="menu"
          tabindex="-1"
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
const isAnimating = ref(false);
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

const updateMenuPosition = () => {
  if (!triggerRef.value || !menuRef.value) return;

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const menuRect = menuRef.value.getBoundingClientRect();

  let top = props.position === "bottom" ? triggerRect.bottom : triggerRect.top - menuRect.height;
  let left = props.direction === "left" ? triggerRect.left : triggerRect.right - menuRect.width;
  if (left + menuRect.width > window.innerWidth - 8) {
    left = window.innerWidth - menuRect.width - 8;
  }
  if (left < 8) {
    left = 8;
  }
  if (top + menuRect.height > window.innerHeight - 8) {
    top = window.innerHeight - menuRect.height - 8;
  }
  if (top < 8) {
    top = 8;
  }

  menuStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
  };
};

const openMenu = () => {
  if (isAnimating.value) return;
  isAnimating.value = true;
  isOpen.value = true;
  disableBodyScroll();
  nextTick(() => {
    updateMenuPosition();
    document.addEventListener("mousemove", handleMouseMove);
    setTimeout(() => {
      isAnimating.value = false;
      focusFirstMenuItem();
    }, 125);
  });
};

const closeMenu = () => {
  if (isAnimating.value) return;
  isAnimating.value = true;
  isOpen.value = false;
  selectedIndex.value = -1;
  enableBodyScroll();
  document.removeEventListener("mousemove", handleMouseMove);
  setTimeout(() => {
    isAnimating.value = false;
  }, 125);
};

const selectOption = (option: Option) => {
  emit("select", option);
  option.action();
  closeMenu();
};

const handleMouseDown = (event: MouseEvent) => {
  event.preventDefault();
  isMouseDown.value = true;
  if (!isOpen.value) {
    openMenu();
  }
  document.addEventListener("mouseup", handleMouseUp);
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

const handleMouseUp = (event: MouseEvent) => {
  if (!isOpen.value) return;

  const menuRect = menuRef.value?.getBoundingClientRect();
  if (!menuRect) return;

  if (
    event.clientX >= menuRect.left &&
    event.clientX <= menuRect.right &&
    event.clientY >= menuRect.top &&
    event.clientY <= menuRect.bottom
  ) {
    if (selectedIndex.value >= 0 && selectedIndex.value < filteredOptions.value.length) {
      selectOption(filteredOptions.value[selectedIndex.value]);
    }
  } else {
    closeMenu();
  }

  isMouseDown.value = false;
  document.removeEventListener("mouseup", handleMouseUp);
};

const handleClick = (event: MouseEvent) => {
  event.stopPropagation();
  if (!isMouseDown.value) {
    if (isOpen.value) {
      closeMenu();
    } else {
      openMenu();
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

const disableBodyScroll = () => {
  document.body.style.overflow = "hidden";
};

const enableBodyScroll = () => {
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
  window.addEventListener("resize", updateMenuPosition);
  window.addEventListener("scroll", updateMenuPosition);
});

onUnmounted(() => {
  triggerRef.value?.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("resize", updateMenuPosition);
  window.removeEventListener("scroll", updateMenuPosition);
  document.removeEventListener("mousemove", handleMouseMove);
  document.removeEventListener("mouseup", handleMouseUp);
  if (typeAheadTimeout.value) {
    clearTimeout(typeAheadTimeout.value);
  }
  enableBodyScroll();
});

watch(isOpen, (newValue) => {
  if (newValue) {
    nextTick(() => {
      updateMenuPosition();
      menuRef.value?.addEventListener("keydown", handleKeydown);
    });
  } else {
    menuRef.value?.removeEventListener("keydown", handleKeydown);
  }
});

onClickOutside(menuRef, () => {
  closeMenu();
  triggerRef.value?.focus();
});
</script>
