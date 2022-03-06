<template>
  <div>
    <div
      v-if="expandedGalleryItem != null"
      class="expanded-image-modal"
      @click="expandedGalleryItem = null"
    >
      <div class="content" @click.stop="">
        <button class="close circle-button" @click="expandedGalleryItem = null">
          <CrossIcon aria-hidden="true" />
        </button>

        <img
          class="image"
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
        />

        <div class="footer">
          <div class="description">
            <h2 v-if="expandedGalleryItem.title">
              {{ expandedGalleryItem.title }}
            </h2>
            <p v-if="expandedGalleryItem.description">
              {{ expandedGalleryItem.description }}
            </p>
          </div>

          <div v-if="gallery.length > 1" class="buttons">
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
    <div v-if="currentMember" class="card buttons">
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
        <UploadIcon />
        Upload
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
        Save
      </button>
      <button
        v-if="
          newGalleryItems.length > 0 ||
          editGalleryIndexes.length > 0 ||
          deleteGalleryUrls.length > 0
        "
        class="action iconified-button"
        @click="resetGallery"
      >
        <TrashIcon />
        Discard Changes
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
              Delete
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
          <div class="gallery-buttons">
            <SmartFileInput
              accept="image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp"
              prompt="Upload"
              @change="(files) => showPreviewImage(files, index)"
            />
            <button
              class="iconified-button"
              @click="newGalleryItems.splice(index, 1)"
            >
              <TrashIcon />
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import UploadIcon from '~/assets/images/utils/upload.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'
import RightArrowIcon from '~/assets/images/utils/right-arrow.svg?inline'
import LeftArrowIcon from '~/assets/images/utils/left-arrow.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'

import SmartFileInput from '~/components/ui/SmartFileInput'
import Checkbox from '~/components/ui/Checkbox'

export default {
  components: {
    CalendarIcon,
    UploadIcon,
    Checkbox,
    EditIcon,
    TrashIcon,
    CheckIcon,
    SmartFileInput,
    CrossIcon,
    RightArrowIcon,
    LeftArrowIcon,
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
    }
  },
  fetch() {
    this.gallery = JSON.parse(JSON.stringify(this.project.gallery))
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

          if (item.title) url += `&title=${item.title}`
          if (item.description) url += `&description=${item.description}`

          await this.$axios.post(url, item.icon, this.$auth.headers)
        }

        for (const index of this.editGalleryIndexes) {
          const item = this.gallery[index]

          let url = `project/${
            this.project.id
          }/gallery?url=${encodeURIComponent(item.url)}&featured=${
            item.featured
          }`

          if (item.title) url += `&title=${item.title}`
          if (item.description) url += `&description=${item.description}`

          await this.$axios.patch(url, {}, this.$auth.headers)
        }

        for (const url of this.deleteGalleryUrls) {
          await this.$axios.delete(
            `project/${this.project.id}/gallery?url=${encodeURIComponent(url)}`,
            this.$auth.headers
          )
        }

        const project = (
          await this.$axios.get(
            `project/${this.project.id}`,
            this.$auth.headers
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
    width: auto;
    height: auto;
    max-height: 96vh;
    max-width: 96vw;
    background-color: var(--color-raised-bg);
    overflow: auto;
    border-radius: var(--size-rounded-card);
    display: flex;
    flex-direction: column;

    .close {
      position: absolute;
      top: 0.5rem;
      right: 0.5rem;
    }

    .next {
      top: 20rem;
      right: 0.5rem;
    }

    .previous {
      top: 20rem;
      left: 0.5rem;
    }

    .circle-button {
      padding: 0.5rem;
      line-height: 1;
      display: flex;
      max-width: 2rem;
      background-color: var(--color-raised-bg);
      border-radius: var(--size-rounded-max);
      margin: 0 0.5rem 0 0;
      box-shadow: inset 0px -1px 1px rgb(17 24 39 / 10%);

      &:hover,
      &:active {
        background-color: var(--color-button-bg-hover) !important;

        svg {
          color: var(--color-button-text-hover) !important;
        }
      }

      svg {
        height: 1rem;
        width: 1rem;
      }
    }

    .image {
      object-fit: contain;
      max-height: 80vh;
      max-width: 80vw;
    }

    .footer {
      display: flex;
      flex-direction: row;
      margin: 0.5rem 0.75rem 0.75rem 0.75rem;

      .buttons {
        display: flex;
        flex-direction: row;
        flex-grow: 0;
        align-items: center;

        .circle-button {
          background-color: var(--color-button-bg);
        }
      }

      .description {
        flex-grow: 1;
        width: min-content;

        h2 {
          margin-bottom: 0.25rem;
          font-size: 1.25rem;
        }

        p {
          margin: 0;
          font-size: 1rem;
        }
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
      width: 14rem;
    }

    input {
      width: calc(14rem - 2rem - 4px);
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
  }
}
</style>
