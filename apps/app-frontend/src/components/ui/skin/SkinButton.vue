<script setup lang="ts">
import { ref, computed } from 'vue'
import { ButtonStyled, commonMessages } from '@modrinth/ui'
import { EditIcon } from '@modrinth/assets'
import { useVIntl } from '@vintl/vintl'
import type { Cape, Skin } from '@/helpers/skins.ts'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  (e: 'select'): void
  (e: 'edit', event: MouseEvent): void
}>()

const props = withDefaults(defineProps<{
  skin: Skin
  selected: boolean
  defaultCape?: Cape
  editable?: boolean
}>(), {
  defaultCape: undefined,
  editable: false,
})

const base64Prefix = 'data:image/png;base64,'
const mcUrlRegex = /texture\/([a-fA-F0-9]+)$/

const texture = computed(() => {
  const mcTextureMatch = props.skin.texture.match(mcUrlRegex)

  if (mcTextureMatch) {
    return mcTextureMatch[1]
  } else if (props.skin.texture.startsWith(base64Prefix)) {
    return props.skin.texture.split(base64Prefix)[1]
  } else {
    return props.skin.texture
  }
})

const slim = computed(() => props.skin.variant === 'Slim')

const skinUrl = computed(
  () => `https://vzge.me/bust/${texture.value}.png?no=ears${slim.value ? '&slim' : ''}`,
)
const backUrl = computed(() => `${skinUrl.value}&y=130`)

const pressed = ref(false)
</script>

<template>
  <div
    class="skin-button__parent group flex relative border-2 border-solid transform-3d rotate-y-90 transition-all h-40 p-0 bg-transparent rounded-xl overflow-hidden"
    :class="[
      selected ? `border-brand` : 'border-transparent',
      {
        'scale-95': pressed,
      },
    ]"
  >
    <button
      class="absolute inset-0 rounded-xl cursor-pointer p-0 border-none group-hover:brightness-125"
      :class="selected ? `bg-brand-highlight` : 'bg-button-bg'"
      @mousedown="pressed = true"
      @mouseup="pressed = false"
      @mouseleave="pressed = false"
      @click="emit('select')"
    ></button>
    <span class="skin-button__image-parent pointer-events-none w-full h-full">
      <img
        alt=""
        :src="skinUrl"
        class="skin-button__image-facing rounded-xl object-contain object-bottom w-full h-full mt-auto mx-auto"
      />
      <img
        alt=""
        :src="backUrl"
        class="skin-button__image-away rounded-xl object-contain object-bottom w-full h-full mt-auto mx-auto"
      />
    </span>
    <span
      v-if="editable"
      class="absolute pointer-events-none inset-0 flex items-end p-2 translate-y-4 -translate-x-4 scale-75 opacity-0 transition-all group-hover:opacity-100 group-hover:scale-100 group-hover:translate-y-0 group-hover:translate-x-0"
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
