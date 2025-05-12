<template>
  <div>
    <Modal
      v-if="currentMember"
      ref="modal_edit_item"
      :header="editIndex === -1 ? 'Upload gallery image' : 'Edit gallery item'"
    >
      <div class="modal-gallery universal-labels">
        <div class="gallery-file-input">
          <div class="file-header">
            <ImageIcon aria-hidden="true" />
            <strong>{{ editFile ? editFile.name : "Current image" }}</strong>
            <FileInput
              v-if="editIndex === -1"
              class="iconified-button raised-button"
              prompt="Replace"
              :accept="acceptFileTypes"
              :max-size="524288000"
              should-always-reset
              aria-label="Replace image"
              @change="
                (x) => {
                  editFile = x[0];
                  showPreviewImage();
                }
              "
            >
              <TransferIcon aria-hidden="true" />
            </FileInput>
          </div>
          <img
            :src="
              previewImage
                ? previewImage
                : project.gallery[editIndex] && project.gallery[editIndex].url
                  ? project.gallery[editIndex].url
                  : 'https://cdn.modrinth.com/placeholder-banner.svg'
            "
            alt="gallery-preview"
          />
        </div>
        <label for="gallery-image-title">
          <span class="label__title">Title</span>
        </label>
        <input
          id="gallery-image-title"
          v-model="editTitle"
          type="text"
          maxlength="64"
          placeholder="Enter title..."
        />
        <label for="gallery-image-desc">
          <span class="label__title">Description</span>
        </label>
        <div class="textarea-wrapper">
          <textarea
            id="gallery-image-desc"
            v-model="editDescription"
            maxlength="255"
            placeholder="Enter description..."
          />
        </div>
        <label for="gallery-image-ordering">
          <span class="label__title">Order Index</span>
        </label>
        <input
          id="gallery-image-ordering"
          v-model="editOrder"
          type="number"
          placeholder="Enter order index..."
        />
        <label for="gallery-image-featured">
          <span class="label__title">Featured</span>
          <span class="label__description">
            A featured gallery image shows up in search and your project card. Only one gallery
            image can be featured.
          </span>
        </label>
        <button
          v-if="!editFeatured"
          id="gallery-image-featured"
          class="iconified-button"
          @click="editFeatured = true"
        >
          <StarIcon aria-hidden="true" />
          Feature image
        </button>
        <button
          v-else
          id="gallery-image-featured"
          class="iconified-button"
          @click="editFeatured = false"
        >
          <StarIcon fill="currentColor" aria-hidden="true" />
          Unfeature image
        </button>
        <div class="button-group">
          <button class="iconified-button" @click="$refs.modal_edit_item.hide()">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
          <button
            v-if="editIndex === -1"
            class="iconified-button brand-button"
            :disabled="shouldPreventActions"
            @click="createGalleryItem"
          >
            <PlusIcon aria-hidden="true" />
            Add gallery image
          </button>
          <button
            v-else
            class="iconified-button brand-button"
            :disabled="shouldPreventActions"
            @click="editGalleryItem"
          >
            <SaveIcon aria-hidden="true" />
            Save changes
          </button>
        </div>
      </div>
    </Modal>
    <ConfirmModal
      v-if="currentMember"
      ref="modal_confirm"
      title="Are you sure you want to delete this gallery image?"
      description="This will remove this gallery image forever (like really forever)."
      :has-to-type="false"
      proceed-label="Delete"
      @proceed="deleteGalleryImage"
    />
    <ImagePreviewModal
      ref="imageModal"
      :next="getNextEntry"
      :prev="getPrevEntry"
      :open-externally="openExternally"
    />
    <div v-if="currentMember" class="card header-buttons">
      <FileInput
        :max-size="524288000"
        :accept="acceptFileTypes"
        prompt="Upload an image"
        aria-label="Upload an image"
        class="iconified-button brand-button"
        :disabled="!isPermission(currentMember?.permissions, 1 << 2)"
        @change="handleFiles"
      >
        <UploadIcon aria-hidden="true" />
      </FileInput>
      <span class="indicator">
        <InfoIcon aria-hidden="true" /> Click to choose an image or drag one onto this page
      </span>
      <DropArea
        :accept="acceptFileTypes"
        :disabled="!isPermission(currentMember?.permissions, 1 << 2)"
        @change="handleFiles"
      />
    </div>
    <div class="items">
      <div v-for="(item, index) in project.gallery" :key="index" class="card gallery-item">
        <a class="gallery-thumbnail" @click="openImage(item, index)">
          <img
            :src="item.url ? item.url : 'https://cdn.modrinth.com/placeholder-banner.svg'"
            :alt="item.title ? item.title : 'gallery-image'"
          />
        </a>
        <div class="gallery-body">
          <div class="gallery-info">
            <h2 v-if="item.title">
              {{ item.title }}
            </h2>
            <p v-if="item.description">
              {{ item.description }}
            </p>
          </div>
        </div>
        <div class="gallery-bottom">
          <div class="gallery-created">
            <CalendarIcon aria-hidden="true" aria-label="Date created" />
            {{ $dayjs(item.created).format("MMMM D, YYYY") }}
          </div>
          <div v-if="currentMember" class="gallery-buttons input-group">
            <button
              class="iconified-button"
              @click="
                () => {
                  resetEdit();
                  editIndex = index;
                  editTitle = item.title;
                  editDescription = item.description;
                  editFeatured = item.featured;
                  editOrder = item.ordering;
                  $refs.modal_edit_item.show();
                }
              "
            >
              <EditIcon aria-hidden="true" />
              Edit
            </button>
            <button
              class="iconified-button"
              @click="
                () => {
                  deleteIndex = index;
                  $refs.modal_confirm.show();
                }
              "
            >
              <TrashIcon aria-hidden="true" />
              Remove
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  PlusIcon,
  CalendarIcon,
  EditIcon,
  TrashIcon,
  SaveIcon,
  StarIcon,
  XIcon,
  UploadIcon,
  InfoIcon,
  ImageIcon,
  TransferIcon,
} from "@modrinth/assets";
import { ConfirmModal, ImagePreviewModal } from "@modrinth/ui";
import FileInput from "~/components/ui/FileInput.vue";
import DropArea from "~/components/ui/DropArea.vue";
import Modal from "~/components/ui/Modal.vue";

import { isPermission } from "~/utils/permissions.ts";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
  resetProject: {
    type: Function,
    required: true,
    default: () => {},
  },
});

const title = `${props.project.title} - Gallery`;
const description = `View ${props.project.gallery.length} images of ${props.project.title} on Modrinth.`;

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});
</script>

<script>
export default defineNuxtComponent({
  data() {
    return {
      expandedGalleryItem: null,
      expandedGalleryIndex: 0,
      zoomedIn: false,

      deleteIndex: -1,

      editIndex: -1,
      editTitle: "",
      editDescription: "",
      editFeatured: false,
      editOrder: null,
      editFile: null,
      previewImage: null,
      shouldPreventActions: false,
    };
  },
  computed: {
    acceptFileTypes() {
      return "image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp";
    },
  },
  mounted() {
    this._keyListener = function (e) {
      if (this.expandedGalleryItem) {
        e.preventDefault();
        if (e.key === "Escape") {
          this.expandedGalleryItem = null;
        } else if (e.key === "ArrowLeft") {
          this.previousImage();
        } else if (e.key === "ArrowRight") {
          this.nextImage();
        }
      }
    };

    document.addEventListener("keydown", this._keyListener.bind(this));
  },
  methods: {
    openImage(item, index) {
      const src = item.raw_url || item.url;
      const alt = item.title || "gallery-image";
      this.$refs.imageModal.show(src, alt, {
        index,
        title: item.title,
        description: item.description,
      });
    },
    getGalleryEntry(key, offset) {
      const gallery = this.project.gallery;
      const len = gallery.length;
      const i = (key.index + offset + len) % len;
      const item = gallery[i];

      return {
        src: item.raw_url || item.url,
        alt: item.title || "gallery-image",
        key: {
          index: i,
          title: item.title,
          description: item.description,
        },
      };
    },
    getNextEntry(key) {
      return this.getGalleryEntry(key, 1);
    },
    getPrevEntry(key) {
      return this.getGalleryEntry(key, -1);
    },
    openExternally(src) {
      window.open(src, "_blank");
    },
    resetEdit() {
      this.editIndex = -1;
      this.editTitle = "";
      this.editDescription = "";
      this.editFeatured = false;
      this.editOrder = null;
      this.editFile = null;
      this.previewImage = null;
    },
    handleFiles(files) {
      this.resetEdit();
      this.editFile = files[0];

      this.showPreviewImage();
      this.$refs.modal_edit_item.show();
    },
    showPreviewImage() {
      const reader = new FileReader();
      if (this.editFile instanceof Blob) {
        reader.readAsDataURL(this.editFile);
        reader.onload = (event) => {
          this.previewImage = event.target.result;
        };
      }
    },
    async createGalleryItem() {
      this.shouldPreventActions = true;
      startLoading();

      try {
        let url = `project/${this.project.id}/gallery?ext=${
          this.editFile
            ? this.editFile.type.split("/")[this.editFile.type.split("/").length - 1]
            : null
        }&featured=${this.editFeatured}`;

        if (this.editTitle) {
          url += `&title=${encodeURIComponent(this.editTitle)}`;
        }
        if (this.editDescription) {
          url += `&description=${encodeURIComponent(this.editDescription)}`;
        }
        if (this.editOrder) {
          url += `&ordering=${this.editOrder}`;
        }

        await useBaseFetch(url, {
          method: "POST",
          body: this.editFile,
        });
        await this.resetProject();

        this.$refs.modal_edit_item.hide();
      } catch (err) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: err.data ? err.data.description : err,
          type: "error",
        });
      }

      stopLoading();
      this.shouldPreventActions = false;
    },
    async editGalleryItem() {
      this.shouldPreventActions = true;
      startLoading();

      try {
        let url = `project/${this.project.id}/gallery?url=${encodeURIComponent(
          this.project.gallery[this.editIndex].url,
        )}&featured=${this.editFeatured}`;

        if (this.editTitle) {
          url += `&title=${encodeURIComponent(this.editTitle)}`;
        }
        if (this.editDescription) {
          url += `&description=${encodeURIComponent(this.editDescription)}`;
        }
        if (this.editOrder) {
          url += `&ordering=${this.editOrder}`;
        }

        await useBaseFetch(url, {
          method: "PATCH",
        });

        await this.resetProject();
        this.$refs.modal_edit_item.hide();
      } catch (err) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: err.data ? err.data.description : err,
          type: "error",
        });
      }

      stopLoading();
      this.shouldPreventActions = false;
    },
    async deleteGalleryImage() {
      startLoading();

      try {
        await useBaseFetch(
          `project/${this.project.id}/gallery?url=${encodeURIComponent(
            this.project.gallery[this.deleteIndex].url,
          )}`,
          {
            method: "DELETE",
          },
        );

        await this.resetProject();
      } catch (err) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: err.data ? err.data.description : err,
          type: "error",
        });
      }

      stopLoading();
    },
  },
});
</script>

<style lang="scss" scoped>
.header-buttons {
  display: flex;
  align-items: center;
  gap: 1rem;

  .indicator {
    display: flex;
    gap: 0.5ch;
    align-items: center;
    color: var(--color-text-inactive);
  }
}

.buttons {
  display: flex;

  button {
    margin-right: 0.5rem;
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
    padding: 0 var(--spacing-card-md) var(--spacing-card-sm) var(--spacing-card-md);

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

.modal-gallery {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;

  .gallery-file-input {
    .file-header {
      border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;

      display: flex;
      align-items: center;
      gap: 0.5rem;
      background-color: var(--color-button-bg);
      padding: var(--spacing-card-md);

      svg {
        min-width: 1rem;
      }
      strong {
        word-wrap: anywhere;
      }

      .iconified-button {
        margin-left: auto;
      }
    }

    img {
      border-radius: 0 0 var(--size-rounded-card) var(--size-rounded-card);
      width: 100%;
      height: auto;
      max-height: 15rem;
      object-fit: contain;
      background-color: #000000;
    }
  }
}

.brand-button {
  color: var(--color-accent-contrast);
}
</style>
