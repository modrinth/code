<template>
  <div>
    <div
      v-if="expandedGalleryItem != null"
      class="expanded-image-modal"
      @click="expandedGalleryItem = null"
    >
      <div class="content">
        <img
          class="image"
          :class="{ 'zoomed-in': zoomedIn }"
          :src="
            expandedGalleryItem.url
              ? expandedGalleryItem.url
              : 'https://cdn.modrinth.com/placeholder-banner.svg'
          "
          :alt="
            expandedGalleryItem.title
              ? expandedGalleryItem.title
              : 'gallery-image'
          "
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
            <div v-if="gallery.length > 1" class="buttons">
              <button
                class="close circle-button"
                @click="expandedGalleryItem = null"
              >
                <CrossIcon aria-hidden="true" />
              </button>
              <a
                class="open circle-button"
                target="_blank"
                :href="
                  expandedGalleryItem.url
                    ? expandedGalleryItem.url
                    : 'https://cdn.modrinth.com/placeholder-banner.svg'
                "
              >
                <ExternalIcon aria-hidden="true" />
              </a>
              <button class="circle-button" @click="zoomedIn = !zoomedIn">
                <ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
                <ContractIcon v-else aria-hidden="true" />
              </button>
              <button class="previous circle-button" @click="previousImage()">
                <LeftArrowIcon aria-hidden="true" />
              </button>
              <button class="next circle-button" @click="nextImage()">
                <RightArrowIcon aria-hidden="true" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="currentMember" class="card buttons header-buttons">
      <button
        v-if="
          newGalleryItems.length > 0 ||
          editGalleryIndexes.length > 0 ||
          deleteGalleryUrls.length > 0
        "
        class="action iconified-button"
        @click="resetGallery"
      >
        <CrossIcon />
        Cancel
      </button>
      <button
        v-if="
          newGalleryItems.length > 0 ||
          editGalleryIndexes.length > 0 ||
          deleteGalleryUrls.length > 0
        "
        class="action brand-button-colors iconified-button"
        @click="saveGallery"
      >
        <CheckIcon />
        Save changes
      </button>
      <button
        class="iconified-button"
        @click="
          newGalleryItems.push({
            title: '',
            description: '',
            featured: false,
            url: '',
          })
        "
      >
        <PlusIcon />
        Add an image
      </button>
    </div>
    <div class="items">
      <div
        v-for="(item, index) in gallery"
        :key="index"
        class="card gallery-item"
      >
        <a class="gallery-thumbnail" @click="expandImage(item, index)">
          <img
            :src="
              item.url
                ? item.url
                : 'https://cdn.modrinth.com/placeholder-banner.svg'
            "
            :alt="item.title ? item.title : 'gallery-image'"
          />
        </a>
        <div class="gallery-body">
          <div v-if="editGalleryIndexes.includes(index)" class="gallery-info">
            <input
              v-model="item.title"
              type="text"
              placeholder="Enter the title..."
            />
            <div class="textarea-wrapper">
              <textarea
                id="body"
                v-model="item.description"
                placeholder="Enter the description..."
              />
            </div>
            <Checkbox v-model="item.featured" label="Featured" />
          </div>
          <div v-else class="gallery-info">
            <h2 v-if="item.title">{{ item.title }}</h2>
            <p v-if="item.description">{{ item.description }}</p>
          </div>
        </div>
        <div class="gallery-bottom">
          <div class="gallery-created">
            <CalendarIcon />
            {{ $dayjs(item.created).format('MMMM D, YYYY') }}
          </div>
          <div v-if="currentMember" class="gallery-buttons">
            <button
              v-if="editGalleryIndexes.includes(index)"
              class="iconified-button"
              @click="
                editGalleryIndexes.splice(editGalleryIndexes.indexOf(index), 1)
                gallery[index] = JSON.parse(
                  JSON.stringify(project.gallery[index])
                )
              "
            >
              <CrossIcon />
              Cancel
            </button>
            <button
              v-else
              class="iconified-button"
              @click="editGalleryIndexes.push(index)"
            >
              <EditIcon />
              Edit
            </button>
            <button
              class="iconified-button"
              @click="
                deleteGalleryUrls.push(item.url)
                gallery.splice(index, 1)
              "
            >
              <TrashIcon />
              Remove
            </button>
          </div>
        </div>
      </div>
      <div
        v-for="(item, index) in newGalleryItems"
        :key="index + 'new'"
        class="card gallery-item"
      >
        <img
          :src="
            newGalleryItems[index].preview
              ? newGalleryItems[index].preview
              : 'https://cdn.modrinth.com/placeholder-banner.svg'
          "
          :alt="item.title ? item.title : 'gallery-image'"
        />
        <div class="gallery-body">
          <div class="gallery-info">
            <input
              v-model="item.title"
              type="text"
              placeholder="Enter the title..."
            />
            <div class="textarea-wrapper">
              <textarea
                id="body"
                v-model="item.description"
                placeholder="Enter the description..."
              />
            </div>
            <Checkbox v-model="item.featured" label="Featured" />
          </div>
        </div>
        <div class="gallery-bottom">
          <SmartFileInput
            :max-size="5242880"
            accept="image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp"
            prompt="Choose image or drag it here"
            @change="(files) => showPreviewImage(files, index)"
          />
          <div class="gallery-buttons">
            <div class="delete-button-container">
              <button
                class="iconified-button"
                @click="newGalleryItems.splice(index, 1)"
              >
                <TrashIcon />
                Remove
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'
import RightArrowIcon from '~/assets/images/utils/right-arrow.svg?inline'
import LeftArrowIcon from '~/assets/images/utils/left-arrow.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import ExternalIcon from '~/assets/images/utils/external.svg?inline'
import ExpandIcon from '~/assets/images/utils/expand.svg?inline'
import ContractIcon from '~/assets/images/utils/contract.svg?inline'

import SmartFileInput from '~/components/ui/SmartFileInput'
import Checkbox from '~/components/ui/Checkbox'

export default {
  components: {
    CalendarIcon,
    PlusIcon,
    Checkbox,
    EditIcon,
    TrashIcon,
    CheckIcon,
    SmartFileInput,
    CrossIcon,
    RightArrowIcon,
    LeftArrowIcon,
    ExternalIcon,
    ExpandIcon,
    ContractIcon,
  },
  auth: false,
  beforeRouteLeave(to, from, next) {
    this.resetGallery()

    next()
  },
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
  },
  data() {
    return {
      gallery: [],
      newGalleryItems: [],
      editGalleryIndexes: [],
      deleteGalleryUrls: [],
      expandedGalleryItem: null,
      expandedGalleryIndex: 0,
      zoomedIn: false,
    }
  },
  fetch() {
    this.gallery = JSON.parse(JSON.stringify(this.project.gallery))
  },
  mounted() {
    this._keyListener = function (e) {
      if (this.expandedGalleryItem) {
        e.preventDefault()
        if (e.key === 'Escape') {
          this.expandedGalleryItem = null
        } else if (e.key === 'ArrowLeft') {
          this.previousImage()
        } else if (e.key === 'ArrowRight') {
          this.nextImage()
        }
      }
    }

    document.addEventListener('keydown', this._keyListener.bind(this))
  },
  methods: {
    showPreviewImage(files, index) {
      const reader = new FileReader()
      this.newGalleryItems[index].icon = files[0]

      if (this.newGalleryItems[index].icon instanceof Blob) {
        reader.readAsDataURL(this.newGalleryItems[index].icon)

        reader.onload = (event) => {
          this.newGalleryItems[index].preview = event.target.result

          // TODO: Find an alternative for this!
          this.$forceUpdate()
        }
      }
    },
    async saveGallery() {
      this.$nuxt.$loading.start()

      try {
        for (const item of this.newGalleryItems) {
          let url = `project/${this.project.id}/gallery?ext=${
            item.icon
              ? item.icon.type.split('/')[item.icon.type.split('/').length - 1]
              : null
          }&featured=${item.featured}`

          if (item.title) url += `&title=${encodeURIComponent(item.title)}`
          if (item.description)
            url += `&description=${encodeURIComponent(item.description)}`

          await this.$axios.post(url, item.icon, this.$defaultHeaders())
        }

        for (const index of this.editGalleryIndexes) {
          const item = this.gallery[index]

          let url = `project/${
            this.project.id
          }/gallery?url=${encodeURIComponent(item.url)}&featured=${
            item.featured
          }`

          if (item.title) url += `&title=${encodeURIComponent(item.title)}`
          if (item.description)
            url += `&description=${encodeURIComponent(item.description)}`

          await this.$axios.patch(url, {}, this.$defaultHeaders())
        }

        for (const url of this.deleteGalleryUrls) {
          await this.$axios.delete(
            `project/${this.project.id}/gallery?url=${encodeURIComponent(url)}`,
            this.$defaultHeaders()
          )
        }

        const project = (
          await this.$axios.get(
            `project/${this.project.id}`,
            this.$defaultHeaders()
          )
        ).data
        this.$emit('update:project', project)
        this.gallery = JSON.parse(JSON.stringify(project.gallery))

        this.deleteGalleryUrls = []
        this.editGalleryIndexes = []
        this.newGalleryItems = []
      } catch (err) {
        const description = err.response.data.description

        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: description,
          type: 'error',
        })

        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()
    },
    resetGallery() {
      this.newGalleryItems = []
      this.editGalleryIndexes = []
      this.deleteGalleryUrls = []
      this.gallery = JSON.parse(JSON.stringify(this.project.gallery))
    },
    nextImage() {
      this.expandedGalleryIndex++
      if (this.expandedGalleryIndex >= this.gallery.length) {
        this.expandedGalleryIndex = 0
      }
      this.expandedGalleryItem = this.gallery[this.expandedGalleryIndex]
    },
    previousImage() {
      this.expandedGalleryIndex--
      if (this.expandedGalleryIndex < 0) {
        this.expandedGalleryIndex = this.gallery.length - 1
      }
      this.expandedGalleryItem = this.gallery[this.expandedGalleryIndex]
    },
    expandImage(item, index) {
      this.expandedGalleryItem = item
      this.expandedGalleryIndex = index
      this.zoomedIn = false
    },
  },
}
</script>

<style lang="scss" scoped>
.expanded-image-modal {
  position: fixed;
  z-index: 20;
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
    width: calc(100vw - 2 * var(--spacing-card-lg));
    height: calc(100vh - 2 * var(--spacing-card-lg));

    .circle-button {
      padding: 0.5rem;
      line-height: 1;
      display: flex;
      max-width: 2rem;
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
      max-width: calc(100vw - 2 * var(--spacing-card-lg));
      max-height: calc(100vh - 2 * var(--spacing-card-lg));
      border-radius: var(--size-rounded-card);

      &.zoomed-in {
        object-fit: cover;
        width: auto;
        height: calc(100vh - 2 * var(--spacing-card-lg));
        max-width: calc(100vw - 2 * var(--spacing-card-lg));
      }
    }
    .floating {
      position: absolute;
      left: 50%;
      transform: translateX(-50%);
      bottom: var(--spacing-card-md);
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: var(--spacing-card-sm);
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
          color: var(--dark-color-text-dark);
          font-size: 1.25rem;
          text-align: center;
          margin: 0;
        }

        p {
          color: var(--dark-color-text);
          margin: 0;
        }
      }
      .controls {
        background-color: var(--color-raised-bg);
        padding: var(--spacing-card-md);
        border-radius: var(--size-rounded-card);
        transition: opacity 0.25s ease-in-out, transform 0.25s ease-in-out;
      }
    }
  }
}

.buttons {
  display: flex;

  button {
    margin-right: 0.5rem;

    &.brand-button-colors {
      background-color: var(--color-brand);

      &:hover {
        background-color: var(--color-brand-hover);
      }

      &:active {
        background-color: var(--color-brand-active);
      }
    }
  }
}

.items {
  display: grid;
  grid-template-rows: 1fr;
  grid-template-columns: 1fr;
  grid-gap: var(--spacing-card-md);

  @media screen and (min-width: 1024px) {
    grid-template-columns: 1fr 1fr 1fr;
  }
}

.gallery-item {
  display: flex;
  flex-direction: column;
  padding: 0;

  img {
    width: 100%;
    margin-top: 0;
    margin-bottom: 0;
    border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;

    min-height: 10rem;
    object-fit: cover;
  }

  .gallery-body {
    flex-grow: 1;
    width: calc(100% - 2 * var(--spacing-card-md));
    padding: var(--spacing-card-sm) var(--spacing-card-md);

    textarea {
      border-radius: var(--size-rounded-sm);
    }

    .textarea-wrapper {
      width: 100%;
    }

    input {
      width: 100%;
      margin: 0 0 0.25rem;
    }

    .gallery-info {
      h2 {
        margin-bottom: 0.5rem;
      }

      p {
        margin: 0 0 0.5rem 0;
      }
    }
  }

  .gallery-thumbnail {
    cursor: pointer;

    img {
      transition: filter 0.25s ease-in-out;

      &:hover {
        filter: brightness(0.7);
      }
    }
  }

  .gallery-bottom {
    width: calc(100% - 2 * var(--spacing-card-md));
    padding: 0 var(--spacing-card-md) var(--spacing-card-sm)
      var(--spacing-card-md);

    .gallery-created {
      display: flex;
      align-items: center;
      margin-bottom: 0.5rem;
      color: var(--color-icon);

      svg {
        width: 1rem;
        height: 1rem;
        margin-right: 0.25rem;
      }
    }

    .gallery-buttons {
      display: flex;
    }

    .columns {
      margin-bottom: 0.5rem;
    }
  }
}

.header-buttons {
  display: flex;
  justify-content: right;
}
</style>
