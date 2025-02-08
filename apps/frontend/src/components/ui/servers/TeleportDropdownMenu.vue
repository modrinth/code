<template>
  <div class="relative inline-block h-9 w-full max-w-80">
    <button
      ref="triggerRef"
      type="button"
      aria-haspopup="listbox"
      :aria-expanded="dropdownVisible"
      :aria-controls="listboxId"
      :aria-labelledby="listboxId"
      class="duration-50 flex h-full w-full cursor-pointer select-none appearance-none items-center justify-between gap-4 rounded-xl border-none bg-button-bg px-4 py-2 shadow-sm !outline-none transition-all ease-in-out"
      :class="triggerClasses"
      @click="toggleDropdown"
      @keydown="handleTriggerKeyDown"
    >
      <span>{{ selectedOption }}</span>
      <DropdownIcon
        class="transition-transform duration-200 ease-in-out"
        :class="{ 'rotate-180': dropdownVisible }"
      />
    </button>

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
          :id="listboxId"
          ref="optionsContainer"
          role="listbox"
          tabindex="-1"
          :aria-activedescendant="activeDescendant"
          class="experimental-styles-within fixed z-50 bg-button-bg shadow-lg outline-none"
          :class="{
            'rounded-b-xl': !isRenderingUp,
            'rounded-t-xl': isRenderingUp,
          }"
          :style="positionStyle"
          @keydown="handleListboxKeyDown"
        >
          <div
            class="overflow-y-auto"
            :style="{ height: `${virtualListHeight}px` }"
            @scroll="handleScroll"
          >
            <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
              <div
                v-for="item in visibleOptions"
                :key="item.index"
                :style="{
                  position: 'absolute',
                  top: 0,
                  transform: `translateY(${item.index * ITEM_HEIGHT}px)`,
                  width: '100%',
                  height: `${ITEM_HEIGHT}px`,
                }"
              >
                <div
                  :id="`${listboxId}-option-${item.index}`"
                  role="option"
                  :aria-selected="selectedValue === item.option"
                  class="hover:brightness-85 flex h-full cursor-pointer select-none items-center px-4 transition-colors duration-150 ease-in-out"
                  :class="{
                    'bg-brand font-bold text-brand-inverted': selectedValue === item.option,
                    'bg-bg-raised': focusedOptionIndex === item.index,
                    'rounded-b-xl': item.index === props.options.length - 1 && !isRenderingUp,
                    'rounded-t-xl': item.index === 0 && isRenderingUp,
                  }"
                  @click="selectOption(item.option, item.index)"
                  @mousemove="focusedOptionIndex = item.index"
                >
                  {{ displayName(item.option) }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { DropdownIcon } from "@modrinth/assets";
import { computed, ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import type { CSSProperties } from "vue";

const ITEM_HEIGHT = 44;
const BUFFER_ITEMS = 5;

type OptionValue = string | number | Record<string, any>;

interface Props {
  options: OptionValue[];
  name: string;
  defaultValue?: OptionValue | null;
  placeholder?: string | number | null;
  modelValue?: OptionValue | null;
  renderUp?: boolean;
  disabled?: boolean;
  displayName?: (option: OptionValue) => string;
}

const props = withDefaults(defineProps<Props>(), {
  defaultValue: null,
  placeholder: null,
  modelValue: null,
  renderUp: false,
  disabled: false,
  displayName: (option: OptionValue) => String(option),
});

const emit = defineEmits<{
  (e: "input", value: OptionValue): void;
  (e: "change", value: { option: OptionValue; index: number }): void;
  (e: "update:modelValue", value: OptionValue): void;
}>();

const dropdownVisible = ref(false);
const selectedValue = ref<OptionValue | null>(props.modelValue || props.defaultValue);
const focusedOptionIndex = ref<number | null>(null);
const optionsContainer = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const isRenderingUp = ref(false);
const virtualListHeight = ref(300);
const isOpen = ref(false);
const openDropdownCount = ref(0);
const listboxId = `pyro-listbox-${Math.random().toString(36).substring(2, 11)}`;
const triggerRef = ref<HTMLButtonElement | null>(null);

const positionStyle = ref<CSSProperties>({
  position: "fixed",
  top: "0px",
  left: "0px",
  width: "0px",
  zIndex: 999,
});

const totalHeight = computed(() => props.options.length * ITEM_HEIGHT);

const visibleOptions = computed(() => {
  const startIndex = Math.floor(scrollTop.value / ITEM_HEIGHT) - BUFFER_ITEMS;
  const visibleCount = Math.ceil(virtualListHeight.value / ITEM_HEIGHT) + 2 * BUFFER_ITEMS;

  return Array.from({ length: visibleCount }, (_, i) => {
    const index = startIndex + i;
    if (index >= 0 && index < props.options.length) {
      return {
        index,
        option: props.options[index],
      };
    }
    return null;
  }).filter((item): item is { index: number; option: OptionValue } => item !== null);
});

const selectedOption = computed(() => {
  if (selectedValue.value !== null && selectedValue.value !== undefined) {
    return props.displayName(selectedValue.value as OptionValue);
  }
  return props.placeholder || "Select an option";
});

const radioValue = computed<OptionValue>({
  get() {
    return props.modelValue ?? selectedValue.value ?? "";
  },
  set(newValue: OptionValue) {
    emit("update:modelValue", newValue);
    selectedValue.value = newValue;
  },
});

const triggerClasses = computed(() => ({
  "!cursor-not-allowed opacity-50 grayscale": props.disabled,
  "rounded-b-none": dropdownVisible.value && !isRenderingUp.value && !props.disabled,
  "rounded-t-none": dropdownVisible.value && isRenderingUp.value && !props.disabled,
}));

const updatePosition = async () => {
  if (!triggerRef.value) return;

  await nextTick();
  const triggerRect = triggerRef.value.getBoundingClientRect();
  const viewportHeight = window.innerHeight;
  const margin = 8;

  const contentHeight = props.options.length * ITEM_HEIGHT;
  const preferredHeight = Math.min(contentHeight, 300);

  const spaceBelow = viewportHeight - triggerRect.bottom;
  const spaceAbove = triggerRect.top;

  isRenderingUp.value = spaceBelow < preferredHeight && spaceAbove > spaceBelow;

  virtualListHeight.value = isRenderingUp.value
    ? Math.min(spaceAbove - margin, preferredHeight)
    : Math.min(spaceBelow - margin, preferredHeight);

  positionStyle.value = {
    position: "fixed",
    left: `${triggerRect.left}px`,
    width: `${triggerRect.width}px`,
    zIndex: 999,
    ...(isRenderingUp.value
      ? { bottom: `${viewportHeight - triggerRect.top}px`, top: "auto" }
      : { top: `${triggerRect.bottom}px`, bottom: "auto" }),
  };
};

const toggleDropdown = () => {
  if (!props.disabled) {
    if (dropdownVisible.value) {
      closeDropdown();
    } else {
      openDropdown();
    }
  }
};

const handleResize = () => {
  if (dropdownVisible.value) {
    requestAnimationFrame(() => {
      updatePosition();
    });
  }
};

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement;
  scrollTop.value = target.scrollTop;
};

const closeAllDropdowns = () => {
  const event = new CustomEvent("close-all-dropdowns");
  window.dispatchEvent(event);
};

const selectOption = (option: OptionValue, index: number) => {
  radioValue.value = option;
  emit("change", { option, index });
  closeDropdown();
};

const focusNextOption = () => {
  if (focusedOptionIndex.value === null) {
    focusedOptionIndex.value = 0;
  } else {
    focusedOptionIndex.value = (focusedOptionIndex.value + 1) % props.options.length;
  }
  scrollToFocused();
};

const focusPreviousOption = () => {
  if (focusedOptionIndex.value === null) {
    focusedOptionIndex.value = props.options.length - 1;
  } else {
    focusedOptionIndex.value =
      (focusedOptionIndex.value - 1 + props.options.length) % props.options.length;
  }
  scrollToFocused();
};

const scrollToFocused = () => {
  if (focusedOptionIndex.value === null) return;

  const optionsElement = optionsContainer.value?.querySelector(".overflow-y-auto");
  if (!optionsElement) return;

  const targetScrollTop = focusedOptionIndex.value * ITEM_HEIGHT;
  const scrollBottom = optionsElement.clientHeight;

  if (targetScrollTop < optionsElement.scrollTop) {
    optionsElement.scrollTop = targetScrollTop;
  } else if (targetScrollTop + ITEM_HEIGHT > optionsElement.scrollTop + scrollBottom) {
    optionsElement.scrollTop = targetScrollTop - scrollBottom + ITEM_HEIGHT;
  }
};

const openDropdown = async () => {
  if (!props.disabled) {
    closeAllDropdowns();
    dropdownVisible.value = true;
    isOpen.value = true;
    openDropdownCount.value++;
    document.body.style.overflow = "hidden";
    await updatePosition();

    nextTick(() => {
      optionsContainer.value?.focus();
    });
  }
};

const closeDropdown = () => {
  if (isOpen.value) {
    dropdownVisible.value = false;
    isOpen.value = false;
    openDropdownCount.value--;
    if (openDropdownCount.value === 0) {
      document.body.style.overflow = "";
    }
    focusedOptionIndex.value = null;
    triggerRef.value?.focus();
  }
};

const handleTriggerKeyDown = (event: KeyboardEvent) => {
  switch (event.key) {
    case "ArrowDown":
    case "ArrowUp":
      event.preventDefault();
      if (!dropdownVisible.value) {
        openDropdown();
        focusedOptionIndex.value = event.key === "ArrowUp" ? props.options.length - 1 : 0;
      } else if (event.key === "ArrowDown") {
        focusNextOption();
      } else {
        focusPreviousOption();
      }
      break;
    case "Enter":
    case " ":
      event.preventDefault();
      if (!dropdownVisible.value) {
        openDropdown();
        focusedOptionIndex.value = 0;
      } else if (focusedOptionIndex.value !== null) {
        selectOption(props.options[focusedOptionIndex.value], focusedOptionIndex.value);
      }
      break;
    case "Escape":
      event.preventDefault();
      closeDropdown();
      break;
    case "Tab":
      if (dropdownVisible.value) {
        event.preventDefault();
      }
      break;
  }
};

const handleListboxKeyDown = (event: KeyboardEvent) => {
  switch (event.key) {
    case "Enter":
    case " ":
      event.preventDefault();
      if (focusedOptionIndex.value !== null) {
        selectOption(props.options[focusedOptionIndex.value], focusedOptionIndex.value);
      }
      break;
    case "ArrowDown":
      event.preventDefault();
      focusNextOption();
      break;
    case "ArrowUp":
      event.preventDefault();
      focusPreviousOption();
      break;
    case "Escape":
      event.preventDefault();
      closeDropdown();
      break;
    case "Tab":
      event.preventDefault();
      break;
    case "Home":
      event.preventDefault();
      focusedOptionIndex.value = 0;
      scrollToFocused();
      break;
    case "End":
      event.preventDefault();
      focusedOptionIndex.value = props.options.length - 1;
      scrollToFocused();
      break;
    default:
      if (event.key.length === 1) {
        const char = event.key.toLowerCase();
        const index = props.options.findIndex((option) =>
          props.displayName(option).toLowerCase().startsWith(char),
        );
        if (index !== -1) {
          focusedOptionIndex.value = index;
          scrollToFocused();
        }
      }
      break;
  }
};

onMounted(() => {
  window.addEventListener("resize", handleResize);
  window.addEventListener("scroll", handleResize, true);
  window.addEventListener("click", (event) => {
    if (!isChildOfDropdown(event.target as HTMLElement)) {
      closeDropdown();
    }
  });
  window.addEventListener("close-all-dropdowns", closeDropdown);

  if (selectedValue.value) {
    focusedOptionIndex.value = props.options.findIndex((option) => option === selectedValue.value);
  }
});

onUnmounted(() => {
  window.removeEventListener("resize", handleResize);
  window.removeEventListener("scroll", handleResize, true);
  window.removeEventListener("click", (event) => {
    if (!isChildOfDropdown(event.target as HTMLElement)) {
      closeDropdown();
    }
  });
  window.removeEventListener("close-all-dropdowns", closeDropdown);

  if (isOpen.value) {
    openDropdownCount.value--;
    if (openDropdownCount.value === 0) {
      document.body.style.overflow = "";
    }
  }
});

watch(
  () => props.modelValue,
  (newValue) => {
    selectedValue.value = newValue;
  },
);

watch(dropdownVisible, async (newValue) => {
  if (newValue) {
    await updatePosition();
    scrollTop.value = 0;
  }
});

const activeDescendant = computed(() =>
  focusedOptionIndex.value !== null ? `${listboxId}-option-${focusedOptionIndex.value}` : undefined,
);

const isChildOfDropdown = (element: HTMLElement | null): boolean => {
  let currentNode: HTMLElement | null = element;
  while (currentNode) {
    if (currentNode === triggerRef.value || currentNode === optionsContainer.value) {
      return true;
    }
    currentNode = currentNode.parentElement;
  }
  return false;
};
</script>
