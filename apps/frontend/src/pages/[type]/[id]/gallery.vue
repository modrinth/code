<template>
  <div>
    <Modal
      v-if="currentMember"
      ref="modal_edit_item"
      :header="editIndex === -1 ? '上传渲染图' : '编辑渲染图'"
    >
      <div class="modal-gallery universal-labels">
        <div class="gallery-file-input">
          <div class="file-header">
            <ImageIcon aria-hidden="true" />
            <strong>{{ editFile ? editFile.name : "当前图片" }}</strong>
            <FileInput
              v-if="editIndex === -1"
              class="iconified-button raised-button"
              prompt="替换图片"
              :accept="acceptFileTypes"
              :max-size="524288000"
              should-always-reset
              aria-label="替换图片"
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
                  : 'https://cdn.bbsmc.net/raw/placeholder-banner.svg'
            "
            alt="gallery-preview"
          />
        </div>
        <label for="gallery-image-title">
          <span class="label__title">标题</span>
        </label>
        <input
          id="gallery-image-title"
          v-model="editTitle"
          type="text"
          maxlength="64"
          placeholder="输入标题.."
        />
        <label for="gallery-image-desc">
          <span class="label__title">简介</span>
        </label>
        <div class="textarea-wrapper">
          <textarea
            id="gallery-image-desc"
            v-model="editDescription"
            maxlength="255"
            placeholder="输入简介..."
          />
        </div>
        <label for="gallery-image-ordering">
          <span class="label__title">优先级</span>
        </label>
        <input
          id="gallery-image-ordering"
          v-model="editOrder"
          type="number"
          placeholder="优先级数字..."
        />
        <label for="gallery-image-featured">
          <span class="label__title">精选</span>
          <span class="label__description">
            精选图库图片会显示在搜索和资源卡中,只能有一张图库图片被精选
          </span>
        </label>
        <button
          v-if="!editFeatured"
          id="gallery-image-featured"
          class="iconified-button"
          @click="editFeatured = true"
        >
          <StarIcon aria-hidden="true" />
          未精选
        </button>
        <button
          v-else
          id="gallery-image-featured"
          class="iconified-button"
          @click="editFeatured = false"
        >
          <StarIcon fill="currentColor" aria-hidden="true" />
          已精选
        </button>
        <div class="button-group">
          <button class="iconified-button" @click="$refs.modal_edit_item.hide()">
            <XIcon aria-hidden="true" />
            取消
          </button>
          <button
            v-if="editIndex === -1"
            class="iconified-button brand-button"
            :disabled="shouldPreventActions"
            @click="createGalleryItem"
          >
            <PlusIcon aria-hidden="true" />
            上传
          </button>
          <button
            v-else
            class="iconified-button brand-button"
            :disabled="shouldPreventActions"
            @click="editGalleryItem"
          >
            <SaveIcon aria-hidden="true" />
            保存
          </button>
        </div>
      </div>
    </Modal>
    <ConfirmModal
      v-if="currentMember"
      ref="modal_confirm"
      title="请确认删除"
      description="这将永久删除此图库图像（真的永久删除）."
      :has-to-type="false"
      proceed-label="删除"
      @proceed="deleteGalleryImage"
    />
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
            expandedGalleryItem.raw_url
              ? expandedGalleryItem.raw_url
              : 'https://cdn.bbsmc.net/raw/placeholder-banner.svg'
          "
          :alt="expandedGalleryItem.title ? expandedGalleryItem.title : 'gallery-image'"
          @click.stop
        />

        <div class="floating" @click.stop>
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
              <button class="close circle-button" @click="expandedGalleryItem = null">
                <XIcon aria-hidden="true" />
              </button>
              <a
                class="open circle-button"
                target="_blank"
                :href="
                  expandedGalleryItem.raw_url
                    ? expandedGalleryItem.raw_url
                    : 'https://cdn.bbsmc.net/raw/placeholder-banner.svg'
                "
              >
                <ExternalIcon aria-hidden="true" />
              </a>
              <button class="circle-button" @click="zoomedIn = !zoomedIn">
                <ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
                <ContractIcon v-else aria-hidden="true" />
              </button>
              <button
                v-if="project.gallery.length > 1"
                class="previous circle-button"
                @click="previousImage()"
              >
                <LeftArrowIcon aria-hidden="true" />
              </button>
              <button
                v-if="project.gallery.length > 1"
                class="next circle-button"
                @click="nextImage()"
              >
                <RightArrowIcon aria-hidden="true" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="currentMember" class="card header-buttons">
      <FileInput
        :max-size="524288000"
        :accept="acceptFileTypes"
        prompt="上传渲染图"
        aria-label="上传渲染图"
        class="iconified-button brand-button"
        :disabled="!isPermission(currentMember?.permissions, 1 << 2)"
        @change="handleFiles"
      >
        <UploadIcon aria-hidden="true" />
      </FileInput>
      <span class="indicator">
        <InfoIcon aria-hidden="true" /> 单击选择图像或将其拖到此页面上
      </span>
      <DropArea
        :accept="acceptFileTypes"
        :disabled="!isPermission(currentMember?.permissions, 1 << 2)"
        @change="handleFiles"
      />
    </div>
    <div class="items">
      <div v-for="(item, index) in project.gallery" :key="index" class="card gallery-item">
        <a class="gallery-thumbnail" @click="expandImage(item, index)">
          <img
            :src="item.url ? item.url : 'https://cdn.bbsmc.net/raw/placeholder-banner.svg'"
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
              编辑
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
              删除
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
  RightArrowIcon,
  LeftArrowIcon,
  ExternalIcon,
  ExpandIcon,
  ContractIcon,
  UploadIcon,
  InfoIcon,
  ImageIcon,
  TransferIcon,
} from "@modrinth/assets";
import { ConfirmModal } from "@modrinth/ui";
import FileInput from "~/components/ui/FileInput.vue";
import DropArea from "~/components/ui/DropArea.vue";
import Modal from "~/components/ui/Modal.vue";

import { isPermission } from "~/utils/permissions.ts";
import dayjs from "dayjs";
import 'dayjs/locale/zh-cn';
dayjs.locale('zh-cn');
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

const title = `${props.project.title} - 渲染图`;
const description = `浏览 ${props.project.gallery.length} 个图片 ${props.project.title} 在 BBSMC.`;

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
    nextImage() {
      this.expandedGalleryIndex++;
      if (this.expandedGalleryIndex >= this.project.gallery.length) {
        this.expandedGalleryIndex = 0;
      }
      this.expandedGalleryItem = this.project.gallery[this.expandedGalleryIndex];
    },
    previousImage() {
      this.expandedGalleryIndex--;
      if (this.expandedGalleryIndex < 0) {
        this.expandedGalleryIndex = this.project.gallery.length - 1;
      }
      this.expandedGalleryItem = this.project.gallery[this.expandedGalleryIndex];
    },
    expandImage(item, index) {
      this.expandedGalleryItem = item;
      this.expandedGalleryIndex = index;
      this.zoomedIn = false;
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
          title: "发生错误",
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
          title: "发生错误",
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
          title: "发生错误",
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
        transition:
          opacity 0.25s ease-in-out,
          transform 0.25s ease-in-out;
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
        transition:
          opacity 0.25s ease-in-out,
          transform 0.25s ease-in-out;
      }
    }
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
