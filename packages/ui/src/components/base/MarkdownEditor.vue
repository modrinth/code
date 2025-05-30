<template>
  <Modal ref="linkModal" header="Insert link">
    <div class="modal-insert">
      <label class="label" for="insert-link-label">
        <span class="label__title">Label</span>
      </label>
      <div class="iconified-input">
        <AlignLeftIcon />
        <input id="insert-link-label" v-model="linkText" type="text" placeholder="Enter label..." />
        <Button class="r-btn" @click="() => (linkText = '')">
          <XIcon />
        </Button>
      </div>
      <label class="label" for="insert-link-url">
        <span class="label__title">URL<span class="required">*</span></span>
      </label>
      <div class="iconified-input">
        <LinkIcon />
        <input
          id="insert-link-url"
          v-model="linkUrl"
          type="text"
          placeholder="Enter the link's URL..."
          @input="validateURL"
        />
        <Button class="r-btn" @click="() => (linkUrl = '')">
          <XIcon />
        </Button>
      </div>
      <template v-if="linkValidationErrorMessage">
        <span class="label">
          <span class="label__title">Error</span>
          <span class="label__description">{{ linkValidationErrorMessage }}</span>
        </span>
      </template>
      <span class="label">
        <span class="label__title">Preview</span>
        <span class="label__description"></span>
      </span>
      <div class="markdown-body-wrapper">
        <div
          style="width: 100%"
          class="markdown-body"
          v-html="renderHighlightedString(linkMarkdown)"
        />
      </div>
      <div class="input-group push-right">
        <Button :action="() => linkModal?.hide()"><XIcon /> Cancel</Button>
        <Button
          color="primary"
          :disabled="linkValidationErrorMessage || !linkUrl"
          :action="
            () => {
              if (editor) markdownCommands.replaceSelection(editor, linkMarkdown)
              linkModal?.hide()
            }
          "
          ><PlusIcon /> Insert</Button
        >
      </div>
    </div>
  </Modal>
  <Modal ref="imageModal" header="Insert image">
    <div class="modal-insert">
      <label class="label" for="insert-image-alt">
        <span class="label__title">Description (alt text)<span class="required">*</span></span>
        <span class="label__description">
          Describe the image completely as you would to someone who could not see the image.
        </span>
      </label>
      <div class="iconified-input">
        <AlignLeftIcon />
        <input
          id="insert-image-alt"
          v-model="linkText"
          type="text"
          placeholder="Describe the image..."
        />
        <Button class="r-btn" @click="() => (linkText = '')">
          <XIcon />
        </Button>
      </div>
      <label class="label" for="insert-link-url">
        <span class="label__title">URL<span class="required">*</span></span>
      </label>
      <div v-if="props.onImageUpload" class="image-strategy-chips">
        <Chips v-model="imageUploadOption" :items="['upload', 'link']" />
      </div>
      <div
        v-if="props.onImageUpload && imageUploadOption === 'upload'"
        class="btn-input-alternative"
      >
        <FileInput
          accept="image/png,image/jpeg,image/gif,image/webp"
          prompt="Drag and drop to upload or click to select file"
          long-style
          should-always-reset
          class="file-input"
          @change="handleImageUpload"
        >
          <UploadIcon />
        </FileInput>
      </div>
      <div v-if="!props.onImageUpload || imageUploadOption === 'link'" class="iconified-input">
        <ImageIcon />
        <input
          id="insert-link-url"
          v-model="linkUrl"
          type="text"
          placeholder="Enter the image URL..."
          @input="validateURL"
        />
        <Button class="r-btn" @click="() => (linkUrl = '')">
          <XIcon />
        </Button>
      </div>
      <template v-if="linkValidationErrorMessage">
        <span class="label">
          <span class="label__title">Error</span>
          <span class="label__description">{{ linkValidationErrorMessage }}</span>
        </span>
      </template>
      <span class="label">
        <span class="label__title">Preview</span>
        <span class="label__description"></span>
      </span>
      <div class="markdown-body-wrapper">
        <div
          style="width: 100%"
          class="markdown-body"
          v-html="renderHighlightedString(imageMarkdown)"
        />
      </div>
      <div class="input-group push-right">
        <Button :action="() => imageModal?.hide()"><XIcon /> Cancel</Button>
        <Button
          color="primary"
          :disabled="!canInsertImage"
          :action="
            () => {
              if (editor) markdownCommands.replaceSelection(editor, imageMarkdown)
              imageModal?.hide()
            }
          "
        >
          <PlusIcon /> Insert
        </Button>
      </div>
    </div>
  </Modal>
  <Modal ref="videoModal" header="Insert YouTube video">
    <div class="modal-insert">
      <label class="label" for="insert-video-url">
        <span class="label__title">YouTube video URL<span class="required">*</span></span>
        <span class="label__description"> Enter a valid link to a YouTube video. </span>
      </label>
      <div class="iconified-input">
        <YouTubeIcon />
        <input
          id="insert-video-url"
          v-model="linkUrl"
          type="text"
          placeholder="Enter YouTube video URL"
          @input="validateURL"
        />
        <Button class="r-btn" @click="() => (linkUrl = '')">
          <XIcon />
        </Button>
      </div>
      <template v-if="linkValidationErrorMessage">
        <span class="label">
          <span class="label__title">Error</span>
          <span class="label__description">{{ linkValidationErrorMessage }}</span>
        </span>
      </template>
      <span class="label">
        <span class="label__title">Preview</span>
        <span class="label__description"></span>
      </span>

      <div class="markdown-body-wrapper">
        <div
          style="width: 100%"
          class="markdown-body"
          v-html="renderHighlightedString(videoMarkdown)"
        />
      </div>
      <div class="input-group push-right">
        <Button :action="() => videoModal?.hide()"><XIcon /> Cancel</Button>
        <Button
          color="primary"
          :disabled="linkValidationErrorMessage || !linkUrl"
          :action="
            () => {
              if (editor) markdownCommands.replaceSelection(editor, videoMarkdown)
              videoModal?.hide()
            }
          "
        >
          <PlusIcon /> Insert
        </Button>
      </div>
    </div>
  </Modal>
  <div class="resizable-textarea-wrapper">
    <div class="editor-action-row">
      <div class="editor-actions">
        <template
          v-for="(buttonGroup, _i) in Object.values(BUTTONS).filter((bg) => bg.display)"
          :key="_i"
        >
          <div class="divider"></div>
          <template v-for="button in buttonGroup.buttons" :key="button.label">
            <Button
              v-tooltip="button.label"
              icon-only
              :aria-label="button.label"
              :class="{ 'mobile-hidden-group': !!buttonGroup.hideOnMobile }"
              :action="() => button.action(editor)"
              :disabled="previewMode || disabled"
            >
              <component :is="button.icon" />
            </Button>
          </template>
        </template>
      </div>
      <div class="preview">
        <Toggle id="preview" v-model="previewMode" />
        <label class="label" for="preview"> Preview </label>
      </div>
    </div>
    <div ref="editorRef" :class="{ hide: previewMode }" />
    <div v-if="!previewMode" class="info-blurb">
      <div class="info-blurb">
        <InfoIcon />
        <span
          >This editor supports
          <a
            class="markdown-resource-link"
            href="https://support.modrinth.com/en/articles/8801962-advanced-markdown-formatting"
            target="_blank"
            >Markdown formatting</a
          >.</span
        >
      </div>
      <div :class="{ hide: !props.maxLength }" class="max-length-label">
        <span>Max length: </span>
        <span>
          {{ props.maxLength ? `${currentValue?.length || 0}/${props.maxLength}` : 'Unlimited' }}
        </span>
      </div>
    </div>
    <div v-else>
      <div class="markdown-body-wrapper">
        <div
          style="width: 100%"
          class="markdown-body"
          v-html="renderHighlightedString(currentValue ?? '')"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { type Component, computed, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue'
import { Compartment, EditorState } from '@codemirror/state'
import { EditorView, keymap, placeholder as cm_placeholder } from '@codemirror/view'
import { markdown } from '@codemirror/lang-markdown'
import { history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
  AlignLeftIcon,
  BoldIcon,
  CodeIcon,
  Heading1Icon,
  Heading2Icon,
  Heading3Icon,
  ImageIcon,
  InfoIcon,
  ItalicIcon,
  LinkIcon,
  ListBulletedIcon,
  ListOrderedIcon,
  PlusIcon,
  ScanEyeIcon,
  StrikethroughIcon,
  TextQuoteIcon,
  UploadIcon,
  XIcon,
  YouTubeIcon,
} from '@modrinth/assets'
import { markdownCommands, modrinthMarkdownEditorKeymap } from '@modrinth/utils/codemirror'
import { renderHighlightedString } from '@modrinth/utils/highlight'
import Modal from '../modal/Modal.vue'
import Button from './Button.vue'
import Toggle from './Toggle.vue'
import FileInput from './FileInput.vue'
import Chips from './Chips.vue'

const props = withDefaults(
  defineProps<{
    modelValue: string
    disabled?: boolean
    headingButtons?: boolean
    /**
     * @param file The file to upload
     * @throws If the file is invalid or the upload fails
     */
    onImageUpload?: (file: File) => Promise<string>
    placeholder?: string
    maxLength?: number
    maxHeight?: number
  }>(),
  {
    modelValue: '',
    disabled: false,
    headingButtons: true,
    onImageUpload: undefined,
    placeholder: 'Write something...',
    maxLength: undefined,
    maxHeight: undefined,
  },
)

const editorRef = ref<HTMLDivElement>()
let editor: EditorView | null = null
let isDisabledCompartment: Compartment | null = null
let editorThemeCompartment: Compartment | null = null

const emit = defineEmits(['update:modelValue'])

onMounted(() => {
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      updateCurrentValue(update.state.doc.toString())
    }
  })

  editorThemeCompartment = new Compartment()

  const theme = EditorView.theme({
    // in defaults.scss there's references to .cm-content and such to inherit global styles
    '.cm-content': {
      marginBlockEnd: '0.5rem',
      padding: '0.5rem',
      minHeight: '200px',
      caretColor: 'var(--color-contrast)',
      width: '100%',
      overflowX: 'scroll',
      maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
      overflowY: 'scroll',
    },
    '.cm-scroller': {
      height: '100%',
      overflow: 'visible',
    },
  })

  isDisabledCompartment = new Compartment()

  const disabledCompartment = EditorState.readOnly.of(props.disabled)

  const eventHandlers = EditorView.domEventHandlers({
    paste: (ev, view) => {
      const { clipboardData } = ev
      if (!clipboardData) return

      if (clipboardData.files && clipboardData.files.length > 0 && props.onImageUpload) {
        // If the user is pasting a file, upload it if there's an included handler and insert the link.
        uploadImagesFromList(clipboardData.files)
          .then(function (url) {
            const selection = markdownCommands.yankSelection(view)
            const altText = selection || 'Replace this with a description'
            const linkMarkdown = `![${altText}](${url})`
            return markdownCommands.replaceSelection(view, linkMarkdown)
          })
          .catch((error) => {
            if (error instanceof Error) {
              console.error('Problem with handling image.', error)
            }
          })

        return false
      }

      // If the user's pasting a url, automatically convert it to a link with the selection as the text or the url itself if no selection content.
      const url = ev.clipboardData?.getData('text/plain')

      if (url) {
        try {
          cleanUrl(url)
        } catch (error: unknown) {
          if (error instanceof Error) {
            return
          }
        }

        const selection = view.state.selection.main
        const selectionText = view.state.doc.sliceString(selection.from, selection.to)
        const linkText = selectionText ? selectionText : url
        const linkMarkdown = `[${linkText}](${url})`
        return markdownCommands.replaceSelection(view, linkMarkdown)
      }

      // Check if the length of the document is greater than the max length. If it is, prevent the paste.
      if (props.maxLength && view.state.doc.length > props.maxLength) {
        ev.preventDefault()
        return false
      }
    },
    blur: (_, view) => {
      if (props.maxLength && view.state.doc.length > props.maxLength) {
        // Calculate how many characters to remove from the end
        const excessLength = view.state.doc.length - props.maxLength
        // Dispatch transaction to remove excess characters
        view.dispatch({
          changes: { from: view.state.doc.length - excessLength, to: view.state.doc.length },
          selection: { anchor: props.maxLength, head: props.maxLength }, // Place cursor at the end
        })
      }
    },
  })

  const inputFilter = EditorState.changeFilter.of((transaction) => {
    if (props.maxLength && transaction.newDoc.length > props.maxLength) {
      return false
    }
    return true
  })

  const editorState = EditorState.create({
    extensions: [
      eventHandlers,
      updateListener,
      keymap.of([indentWithTab]),
      keymap.of(modrinthMarkdownEditorKeymap),
      history(),
      markdown({
        addKeymap: false,
      }),
      keymap.of(historyKeymap),
      cm_placeholder(props.placeholder || ''),
      inputFilter,
      isDisabledCompartment.of(disabledCompartment),
      editorThemeCompartment.of(theme),
    ],
  })

  editor = new EditorView({
    state: editorState,
    parent: editorRef.value,
    doc: props.modelValue ?? '', // This doesn't work for some reason
  })

  // set editor content to props.modelValue
  editor?.dispatch({
    changes: {
      from: 0,
      to: editor.state.doc.length,
      insert: props.modelValue,
    },
  })
})

onBeforeUnmount(() => {
  editor?.destroy()
})

type ButtonAction = {
  label: string
  icon: Component
  action: (editor: EditorView | null) => void
}
type ButtonGroup = {
  display: boolean
  hideOnMobile: boolean
  buttons: ButtonAction[]
}
type ButtonGroupMap = {
  [key: string]: ButtonGroup
}

function runEditorCommand(command: (view: EditorView) => boolean, editor: EditorView | null) {
  if (editor) {
    command(editor)
    editor.focus()
  }
}

const composeCommandButton = (
  name: string,
  icon: Component,
  command: (view: EditorView) => boolean,
) => {
  return {
    label: name,
    icon,
    action: (e: EditorView | null) => runEditorCommand(command, e),
  }
}

const BUTTONS: ButtonGroupMap = {
  headings: {
    display: props.headingButtons,
    hideOnMobile: false,
    buttons: [
      composeCommandButton('Heading 1', Heading1Icon, markdownCommands.toggleHeader),
      composeCommandButton('Heading 2', Heading2Icon, markdownCommands.toggleHeader2),
      composeCommandButton('Heading 3', Heading3Icon, markdownCommands.toggleHeader3),
    ],
  },
  stylizing: {
    display: true,
    hideOnMobile: false,
    buttons: [
      composeCommandButton('Bold', BoldIcon, markdownCommands.toggleBold),
      composeCommandButton('Italic', ItalicIcon, markdownCommands.toggleItalic),
      composeCommandButton(
        'Strikethrough',
        StrikethroughIcon,
        markdownCommands.toggleStrikethrough,
      ),
      composeCommandButton('Code', CodeIcon, markdownCommands.toggleCodeBlock),
      composeCommandButton('Spoiler', ScanEyeIcon, markdownCommands.toggleSpoiler),
    ],
  },
  lists: {
    display: true,
    hideOnMobile: false,
    buttons: [
      composeCommandButton('Bulleted list', ListBulletedIcon, markdownCommands.toggleBulletList),
      composeCommandButton('Ordered list', ListOrderedIcon, markdownCommands.toggleOrderedList),
      composeCommandButton('Quote', TextQuoteIcon, markdownCommands.toggleQuote),
    ],
  },
  components: {
    display: true,
    hideOnMobile: false,
    buttons: [
      {
        label: 'Link',
        icon: LinkIcon,
        action: () => openLinkModal(),
      },
      {
        label: 'Image',
        icon: ImageIcon,
        action: () => openImageModal(),
      },
      {
        label: 'Video',
        icon: YouTubeIcon,
        action: () => openVideoModal(),
      },
    ],
  },
}

watch(
  () => props.disabled,
  (newValue) => {
    if (editor) {
      if (isDisabledCompartment) {
        editor.dispatch({
          effects: [isDisabledCompartment.reconfigure(EditorState.readOnly.of(newValue))],
        })
      }

      if (editorThemeCompartment) {
        editor.dispatch({
          effects: [
            editorThemeCompartment.reconfigure(
              EditorView.theme({
                // in defaults.scss there's references to .cm-content and such to inherit global styles
                '.cm-content': {
                  marginBlockEnd: '0.5rem',
                  padding: '0.5rem',
                  minHeight: '200px',
                  caretColor: 'var(--color-contrast)',
                  width: '100%',
                  overflowX: 'scroll',
                  maxHeight: props.maxHeight ? `${props.maxHeight}px` : 'unset',
                  overflowY: 'scroll',

                  opacity: newValue ? 0.6 : 1,
                  pointerEvents: newValue ? 'none' : 'all',
                  cursor: newValue ? 'not-allowed' : 'auto',
                },
                '.cm-scroller': {
                  height: '100%',
                  overflow: 'visible',
                },
              }),
            ),
          ],
        })
      }
    }
  },
  {
    immediate: true,
  },
)

const currentValue = toRef(props, 'modelValue')
watch(
  currentValue,
  (newValue) => {
    if (editor && newValue !== editor.state.doc.toString()) {
      editor.dispatch({
        changes: {
          from: 0,
          to: editor.state.doc.length,
          insert: newValue,
        },
      })
    }
  },
  {
    immediate: true,
  },
)

const updateCurrentValue = (newValue: string) => {
  emit('update:modelValue', newValue)
}

const previewMode = ref(false)

const linkText = ref('')
const linkUrl = ref('')
const linkValidationErrorMessage = ref<string | undefined>()

function validateURL() {
  if (!linkUrl.value || linkUrl.value === '') {
    linkValidationErrorMessage.value = undefined
    return
  }

  try {
    linkValidationErrorMessage.value = undefined
    cleanUrl(linkUrl.value)
  } catch (e: unknown) {
    if (e instanceof Error) {
      linkValidationErrorMessage.value = e.message
    }
  }
}

function cleanUrl(input: string): string {
  let url: URL

  // Attempt to validate and parse the URL
  try {
    url = new URL(input)
  } catch {
    throw new Error('Invalid URL. Make sure the URL is well-formed.')
  }

  // Check for unsupported protocols
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error('Unsupported protocol. Use http or https.')
  }

  // If the scheme is "http", automatically upgrade it to "https"
  if (url.protocol === 'http:') {
    url.protocol = 'https:'
  }

  // Block certain domains for compliance
  const blockedDomains = ['forgecdn', 'cdn.discordapp', 'media.discordapp']
  if (blockedDomains.some((domain) => url.hostname.includes(domain))) {
    throw new Error('Invalid URL. This domain is not allowed.')
  }

  return url.toString()
}

const linkMarkdown = computed(() => {
  if (!linkUrl.value) {
    return ''
  }
  try {
    const url = cleanUrl(linkUrl.value)
    return url ? `[${linkText.value ? linkText.value : url}](${url})` : ''
  } catch (error: unknown) {
    if (error instanceof Error) {
      console.error(error.message)
    }
  }
  return ''
})

const uploadImagesFromList = async (files: FileList): Promise<string> => {
  const file = files[0]
  if (!props.onImageUpload) {
    throw new Error('No image upload handler provided')
  }
  if (file) {
    try {
      const url = await props.onImageUpload(file)
      return url
    } catch (error) {
      if (error instanceof Error) {
        console.error('Unable to upload image using handler.', error.message)
        throw new Error(error.message)
      }
    }
  }
  throw new Error('No file provided')
}

const handleImageUpload = async (files: FileList) => {
  if (props.onImageUpload) {
    try {
      const uploadedURL = await uploadImagesFromList(files)
      linkUrl.value = uploadedURL
      validateURL()
    } catch (error) {
      if (error instanceof Error) {
        linkValidationErrorMessage.value = error.message
      }
      console.error(error)
    }
  }
}

const imageUploadOption = ref<string>('upload')
const imageMarkdown = computed(() => (linkMarkdown.value.length ? `!${linkMarkdown.value}` : ''))

const canInsertImage = computed(() => {
  // Make sure the image url is valid, there is an image url, and there is alt text
  // They need to be valid, and not empty
  return (
    !linkValidationErrorMessage.value && linkUrl.value?.length > 0 && linkText.value?.length > 0
  )
})

const youtubeRegex =
  /^(?:https?:)?(?:\/\/)?(?:youtu\.be\/|(?:www\.|m\.)?youtube\.com\/(?:watch|v|embed)(?:\.php)?(?:\?.*v=|\/))(?<temp1>[a-zA-Z0-9_-]{7,15})(?:[?&][a-zA-Z0-9_-]+=[a-zA-Z0-9_-]+)*$/

const videoMarkdown = computed(() => {
  const match = youtubeRegex.exec(linkUrl.value)
  if (match) {
    return `<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/${match[1]}" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>`
  }
  return ''
})

const linkModal = ref<InstanceType<typeof Modal> | null>(null)
const imageModal = ref<InstanceType<typeof Modal> | null>(null)
const videoModal = ref<InstanceType<typeof Modal> | null>(null)

function resetModalStates() {
  linkText.value = ''
  linkUrl.value = ''
  linkValidationErrorMessage.value = undefined
}

function openLinkModal() {
  if (editor) linkText.value = markdownCommands.yankSelection(editor)
  resetModalStates()
  linkModal.value?.show()
}

function openImageModal() {
  resetModalStates()
  imageModal.value?.show()
}

function openVideoModal() {
  resetModalStates()
  videoModal.value?.show()
}
</script>

<style lang="scss" scoped>
.file-input {
  width: 100%;
  padding: 1.5rem;
  padding-left: 2.5rem;
  background: var(--color-button-bg);
  border: 2px dashed var(--color-gray);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition:
    opacity 0.5s ease-in-out,
    filter 0.2s ease-in-out,
    scale 0.05s ease-in-out,
    outline 0.2s ease-in-out;

  &:hover {
    filter: brightness(0.85);
  }
}

.markdown-resource-link {
  cursor: pointer;
  color: var(--color-blue);

  &:focus-visible,
  &:hover {
    filter: brightness(1.2);
    text-decoration: none;
  }

  &:active {
    filter: brightness(1.1);
    text-decoration: none;
  }
}

.display-options {
  margin-bottom: var(--gap-sm);
}

.editor-action-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: space-between;
  margin-bottom: var(--gap-sm);
  gap: var(--gap-xs);

  @media (max-width: 768px) {
    flex-direction: column;
    align-items: start;
  }
}

.editor-actions {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--gap-xs);

  @media (max-width: 768px) {
    .divider {
      display: none;
    }

    .mobile-hidden-group {
      display: none;
    }
  }

  .divider {
    width: 0.125rem;
    height: 1.8rem;
    background-color: var(--color-button-bg);
    border-radius: var(--radius-max);
    margin-inline: var(--gap-xs);
  }

  .divider:first-child {
    display: none;
  }
}

.resizable-textarea-wrapper textarea {
  min-height: 10rem;
  width: 100%;
}

.info-blurb {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--gap-xs);
}

.hide {
  display: none;
}

.preview {
  display: flex;
  align-items: center;
  justify-items: end;
  gap: var(--gap-xs);
}

.markdown-body-wrapper {
  border: 1px solid var(--color-button-bg);
  border-radius: var(--radius-md);
  width: 100%;
  padding: var(--radius-md);
  min-height: 6rem;
}

.modal-insert {
  padding: var(--gap-lg);

  .iconified-input {
    width: 100%;
  }

  .label {
    margin-block: var(--gap-lg) var(--gap-sm);
    display: block;
  }

  .label__title {
    color: var(--color-contrast);
    display: block;
    font-size: 1.17rem;
    font-weight: 700;

    .required {
      color: var(--color-red);
    }
  }

  .input-group {
    margin-top: var(--gap-lg);
  }
}

.image-strategy-chips {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--gap-xs);
  padding-bottom: var(--gap-md);
}

.btn-input-alternative {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  gap: var(--gap-xs);
  padding-bottom: var(--gap-xs);

  .btn {
    width: 100%;
    padding-left: 2.5rem;
    min-height: 4rem;
    display: flex;
    align-items: center;
    justify-content: start;
  }
}

.cm-disabled {
  opacity: 0.6;
  pointer-events: none;
  cursor: not-allowed;
}

:deep(.cm-content) {
  overflow: auto;
}
</style>
