<script setup lang="ts">
import { ref } from 'vue'
import { ButtonStyled, commonMessages } from '@modrinth/ui'
import { EditIcon } from '@modrinth/assets'
import { useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  (e: 'select'): void
  (e: 'edit', event: MouseEvent): void
}>()

const props = withDefaults(defineProps<{
  forwardImageSrc?: string
  backwardImageSrc?: string
  selected: boolean
  editable?: boolean
  tooltip?: string
}>(), {
  editable: false,
})

const pressed = ref(false)
const imagesLoaded = ref({
  forward: Boolean(props.forwardImageSrc),
  backward: Boolean(props.backwardImageSrc)
})

function onImageLoad(type: 'forward' | 'backward') {
  imagesLoaded.value[type] = true
}
</script>

<template>
  <div
    class="skin-button__parent group flex relative border-2 border-solid transform-3d rotate-y-90 transition-all p-0 bg-transparent rounded-xl overflow-hidden"
    :class="[
      selected ? `border-brand` : 'border-transparent',
      {
        'scale-95': pressed,
      },
    ]"
    v-tooltip="tooltip ?? undefined"
  >
    <button
      class="absolute inset-0 rounded-xl cursor-pointer p-0 border-none group-hover:brightness-125"
      :class="selected ? `bg-brand-highlight` : 'bg-button-bg'"
      @mousedown="pressed = true"
      @mouseup="pressed = false"
      @mouseleave="pressed = false"
      @click="emit('select')"
    ></button>

    <div v-if="!forwardImageSrc || !backwardImageSrc" class="skeleton-loader w-full h-full rounded-xl">
      <div class="absolute inset-0 rounded-xl skeleton"></div>
    </div>

    <span v-else class="skin-button__image-parent pointer-events-none w-full h-full flex flex-col justify-end">
      <img
        alt=""
        :src="forwardImageSrc"
        class="skin-button__image-facing rounded-xl object-contain w-full h-auto mx-auto mb-0"
        @load="onImageLoad('forward')"
      />
      <img
        alt=""
        :src="backwardImageSrc"
        class="skin-button__image-away rounded-xl object-contain w-full h-auto mx-auto mb-0"
        @load="onImageLoad('backward')"
      />
    </span>

    <span
      v-if="editable"
      class="absolute pointer-events-none inset-0 flex items-end justify-center p-2 translate-y-4 scale-75 opacity-0 transition-all group-hover:opacity-100 group-hover:scale-100 group-hover:translate-y-0 group-hover:translate-x-0"
    >
      <ButtonStyled color="brand">
        <button
          class="pointer-events-auto shadow-black/50 shadow-lg"
          @mousedown.stop
          @mouseup.stop
          @click="(e) => emit('edit', e)"
        >
          <EditIcon />
          {{ formatMessage(commonMessages.editButton) }}
        </button>
      </ButtonStyled>
    </span>
  </div>
</template>

<style scoped lang="scss">
.skin-button__parent {
  perspective: 1000px;

  .skeleton-loader {
    aspect-ratio: 1 / 1;
  }

  .skeleton {
    background: linear-gradient(
        90deg,
        var(--color-bg, #f0f0f0) 25%,
        var(--color-raised-bg, #e0e0e0) 50%,
        var(--color-bg, #f0f0f0) 75%
    );
    background-size: 200% 100%;
    animation: wave 1500ms infinite linear;
  }

  .skin-button__image-parent {
    transform-style: preserve-3d;
    display: grid;

    /*
      Set the images to be in the same position
     */
    .skin-button__image-facing,
    .skin-button__image-away {
      grid-area: 1 / 1;
    }
  }

  .skin-button__image-parent {
    transition: transform 0.3s ease-in-out;

    .skin-button__image-away {
      opacity: 0;
      transform: rotateY(180deg);
    }
  }

  :not(&:hover) {
    .skin-button__image-parent {
      .skin-button__image-facing {
        animation: appear-halfway 0.3s ease-in-out forwards;
      }

      .skin-button__image-away {
        animation: vanish-halfway 0.3s ease-in-out forwards;
      }
    }
  }

  &:hover {
    .skin-button__image-parent {
      transform: rotateY(180deg);

      .skin-button__image-facing {
        animation: vanish-halfway 0.3s ease-in-out forwards;
      }

      .skin-button__image-away {
        animation: appear-halfway 0.3s ease-in-out forwards;
      }
    }
  }
}

@keyframes wave {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

@keyframes vanish-halfway {
  0% {
    opacity: 1;
  }
  50% {
    opacity: 1;
  }
  51% {
    opacity: 0;
  }
  100% {
    opacity: 0;
  }
}

@keyframes appear-halfway {
  0% {
    opacity: 0;
  }
  50% {
    opacity: 0;
  }
  51% {
    opacity: 1;
  }
  100% {
    opacity: 1;
  }
}
</style>
