<template>
  <div class="gallery">
    <Card v-for="(image, index) in project.gallery" :key="image.url" class="gallery-item">
      <a @click="expandImage(image, index)">
        <img :src="image.url" :alt="image.title" class="gallery-image" />
      </a>
      <div class="gallery-body">
        <h3>{{ image.title }}</h3>
        {{ image.description }}
      </div>
      <span class="gallery-time">
        <CalendarIcon />
        {{
          new Date(image.created).toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
          })
        }}
      </span>
    </Card>
  </div>
  <div v-if="expandedGalleryItem" class="expanded-image-modal" @click="expandedGalleryItem = null">
    <div class="content">
      <img
        class="image"
        :class="{ 'zoomed-in': zoomedIn }"
        :src="
          expandedGalleryItem.url
            ? expandedGalleryItem.url
            : 'https://cdn.modrinth.com/placeholder-banner.svg'
        "
        :alt="expandedGalleryItem.title ? expandedGalleryItem.title : 'gallery-image'"
        @click.stop=""
      />

      <div class="floating" @click.stop="">
        <div class="text">
          <h2 v-if="expandedGalleryItem.title">
            {{ expandedGalleryItem.title }}
          </h2>
          <p v-if="expandedGalleryItem.description">
            {{ expandedGalleryItem.description }}
          </p>
        </div>
        <div class="controls">
          <div class="buttons">
            <Button class="close" icon-only @click="expandedGalleryItem = null">
              <XIcon aria-hidden="true" />
            </Button>
            <a
              class="open btn icon-only"
              target="_blank"
              :href="
                expandedGalleryItem.url
                  ? expandedGalleryItem.url
                  : 'https://cdn.modrinth.com/placeholder-banner.svg'
              "
            >
              <ExternalIcon aria-hidden="true" />
            </a>
            <Button icon-only @click="zoomedIn = !zoomedIn">
              <ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
              <ContractIcon v-else aria-hidden="true" />
            </Button>
            <Button
              v-if="project.gallery.length > 1"
              class="previous"
              icon-only
              @click="previousImage()"
            >
              <LeftArrowIcon aria-hidden="true" />
            </Button>
            <Button v-if="project.gallery.length > 1" class="next" icon-only @click="nextImage()">
              <RightArrowIcon aria-hidden="true" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  Card,
  ExpandIcon,
  RightArrowIcon,
  LeftArrowIcon,
  ExternalIcon,
  ContractIcon,
  XIcon,
  CalendarIcon,
  Button,
} from 'omorphia'
import { ref } from 'vue'

const props = defineProps({
  project: {
    type: Object,
    default: () => {},
  },
})

let expandedGalleryItem = ref(null)
let expandedGalleryIndex = ref(0)
let zoomedIn = ref(false)

const nextImage = () => {
  expandedGalleryIndex.value++
  if (expandedGalleryIndex.value >= props.project.gallery.length) {
    expandedGalleryIndex.value = 0
  }
  expandedGalleryItem.value = props.project.gallery[expandedGalleryIndex.value]
}

const previousImage = () => {
  expandedGalleryIndex.value--
  if (expandedGalleryIndex.value < 0) {
    expandedGalleryIndex.value = props.project.gallery.length - 1
  }
  expandedGalleryItem.value = props.project.gallery[expandedGalleryIndex.value]
}

const expandImage = (item, index) => {
  expandedGalleryItem.value = item
  expandedGalleryIndex.value = index
  zoomedIn.value = false
}
</script>

<style scoped lang="scss">
.gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
  width: 100%;
  gap: 1rem;
}

.gallery-item {
  padding: 0;
  overflow: hidden;
  margin: 0;
  display: flex;
  flex-direction: column;

  .gallery-image {
    width: 100%;
    aspect-ratio: 2/1;
    object-fit: cover;
    object-position: center;
  }

  .gallery-body {
    flex-grow: 1;
    padding: 1rem;
  }

  .gallery-time {
    padding: 0 1rem 1rem;
    vertical-align: center;
  }
}

.expanded-image-modal {
  position: fixed;
  z-index: 10;
  overflow: auto;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: #000000;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;

  .content {
    position: relative;
    width: calc(100vw - 2 * var(--gap-lg));
    height: calc(100vh - 2 * var(--gap-lg));

    .circle-button {
      padding: 0.5rem;
      line-height: 1;
      display: flex;
      max-width: 2rem;
      color: var(--color-button-text);
      background-color: var(--color-button-bg);
      border-radius: var(--size-rounded-max);
      margin: 0;
      box-shadow: inset 0px -1px 1px rgb(17 24 39 / 10%);

      &:not(:last-child) {
        margin-right: 0.5rem;
      }

      &:hover {
        background-color: var(--color-button-bg-hover) !important;

        svg {
          color: var(--color-button-text-hover) !important;
        }
      }

      &:active {
        background-color: var(--color-button-bg-active) !important;

        svg {
          color: var(--color-button-text-active) !important;
        }
      }

      svg {
        height: 1rem;
        width: 1rem;
      }
    }

    .image {
      position: absolute;
      left: 50%;
      top: 50%;
      transform: translate(-50%, -50%);
      max-width: calc(100vw - 2 * var(--gap-lg));
      max-height: calc(100vh - 2 * var(--gap-lg));
      border-radius: var(--radius-lg);

      &.zoomed-in {
        object-fit: cover;
        width: auto;
        height: calc(100vh - 2 * var(--gap-lg));
        max-width: calc(100vw - 2 * var(--gap-lg));
      }
    }
    .floating {
      position: absolute;
      left: 50%;
      transform: translateX(-50%);
      bottom: var(--gap-md);
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: var(--gap-md);
      transition: opacity 0.25s ease-in-out;
      opacity: 1;
      padding: 2rem 2rem 0 2rem;

      &:not(&:hover) {
        opacity: 0.4;
        .text {
          transform: translateY(2.5rem) scale(0.8);
          opacity: 0;
        }
        .controls {
          transform: translateY(0.25rem) scale(0.9);
        }
      }

      .text {
        display: flex;
        flex-direction: column;
        max-width: 40rem;
        transition: opacity 0.25s ease-in-out, transform 0.25s ease-in-out;
        text-shadow: 1px 1px 10px #000000d4;
        margin-bottom: 0.25rem;
        gap: 0.5rem;

        h2 {
          color: var(--dark-color-base);
          font-size: 1.25rem;
          text-align: center;
          margin: 0;
        }

        p {
          color: var(--dark-color-base);
          margin: 0;
        }
      }
      .controls {
        background-color: var(--color-raised-bg);
        padding: var(--gap-md);
        border-radius: var(--radius-md);
        transition: opacity 0.25s ease-in-out, transform 0.25s ease-in-out;
      }
    }
  }
}

.buttons {
  display: flex;
  gap: 0.5rem;
}
</style>
