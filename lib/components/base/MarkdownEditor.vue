<template>
  <Modal ref="linkModal" header="Insert link">
    <div class="modal-insert">
      <label class="label" for="insert-link-label">
        <span class="label__title">Label</span>
      </label>
      <div class="iconified-input">
        <AlignLeftIcon />
        <input id="insert-link-label" v-model="linkText" type="text" placeholder="Enter label..." />
        <Button @click="() => (linkText = '')">
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
        <Button @click="() => (linkUrl = '')">
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
      <div
        style="width: 100%"
        class="markdown-body"
        v-html="renderHighlightedString(linkMarkdown)"
      />
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
        <Button @click="() => (linkText = '')">
          <XIcon />
        </Button>
      </div>
      <label class="label" for="insert-link-url">
        <span class="label__title">URL<span class="required">*</span></span>
      </label>
      <div class="iconified-input">
        <ImageIcon />
        <input
          id="insert-link-url"
          v-model="linkUrl"
          type="text"
          placeholder="Enter the image URL..."
          @input="validateURL"
        />
        <Button @click="() => (linkUrl = '')">
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
      <div
        style="width: 100%"
        class="markdown-body"
        v-html="renderHighlightedString(imageMarkdown)"
      />
      <div class="input-group push-right">
        <Button :action="() => imageModal?.hide()"><XIcon /> Cancel</Button>
        <Button
          color="primary"
          :disabled="linkValidationErrorMessage || !linkUrl"
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
        <Button @click="() => (linkUrl = '')">
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
      <div
        style="width: 100%"
        class="markdown-body"
        v-html="renderHighlightedString(videoMarkdown)"
      />
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
      <InfoIcon />
      <span>
        This editor supports
        <a href="https://docs.modrinth.com/docs/markdown" target="_blank">Markdown formatting</a>.
      </span>
    </div>
    <div
      v-if="previewMode"
      style="width: 100%"
      class="markdown-body"
      v-html="renderHighlightedString(currentValue ?? '')"
    />
  </div>
</template>

<script setup lang="ts">
import { type Component, computed, ref, onMounted, onBeforeUnmount } from 'vue'

import { EditorState } from '@codemirror/state'
import { EditorView, keymap } from '@codemirror/view'
import { markdown } from '@codemirror/lang-markdown'
import { indentWithTab, historyKeymap, history } from '@codemirror/commands'

import {
  Heading1Icon,
  Heading2Icon,
  Heading3Icon,
  BoldIcon,
  ItalicIcon,
  StrikethroughIcon,
  CodeIcon,
  ListBulletedIcon,
  ListOrderedIcon,
  TextQuoteIcon,
  LinkIcon,
  ImageIcon,
  YouTubeIcon,
  AlignLeftIcon,
  PlusIcon,
  XIcon,
  Button,
  Modal,
  Toggle,
} from '@/components'
import { markdownCommands, modrinthMarkdownEditorKeymap } from '@/helpers/codemirror'
import { renderHighlightedString } from '@/helpers/highlight'

const props = defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  headingButtons: {
    type: Boolean,
    default: true,
  },
})

const editorRef = ref<HTMLDivElement>()
let editor: EditorView | null = null

const emit = defineEmits(['update:modelValue'])

onMounted(() => {
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      currentValue.value = update.state.doc.toString()
      emit('update:modelValue', currentValue.value)
    }
  })

  const theme = EditorView.theme({
    // in defualts.scss there's references to .cm-content and such to inherit global styles
    '.cm-content, .cm-gutter': {
      marginBlockEnd: '0.5rem',
      padding: '0.5rem',
      minHeight: '200px',
      caretColor: 'var(--color-contrast)',
      width: '100%',
      overflowX: 'scroll',
    },
    '.cm-scroller': {
      height: '100%',
      overflow: 'visible',
    },
  })

  const eventHandlers = EditorView.domEventHandlers({
    paste: (ev, view) => {
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
    },
  })

  const editorState = EditorState.create({
    doc: props.modelValue,
    extensions: [
      theme,
      eventHandlers,
      updateListener,
      keymap.of([indentWithTab]),
      keymap.of(modrinthMarkdownEditorKeymap),
      history(),
      markdown({
        addKeymap: false,
      }),
      keymap.of(historyKeymap),
    ],
  })

  editor = new EditorView({
    state: editorState,
    parent: editorRef.value,
    doc: props.modelValue,
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
  command: (view: EditorView) => boolean
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
        markdownCommands.toggleStrikethrough
      ),
      composeCommandButton('Code', CodeIcon, markdownCommands.toggleCodeBlock),
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

const currentValue = ref(props.modelValue)
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
  let url

  // Attempt to validate and parse the URL
  try {
    url = new URL(input)
  } catch (e) {
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

const imageMarkdown = computed(() => (linkMarkdown.value.length ? `!${linkMarkdown.value}` : ''))

const youtubeRegex =
  /^(?:https?:)?(?:\/\/)?(?:youtu\.be\/|(?:www\.|m\.)?youtube\.com\/(?:watch|v|embed)(?:\.php)?(?:\?.*v=|\/))([a-zA-Z0-9_-]{7,15})(?:[?&][a-zA-Z0-9_-]+=[a-zA-Z0-9_-]+)*$/

const videoMarkdown = computed(() => {
  const match = youtubeRegex.exec(linkUrl.value)
  if (match) {
    return `<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/${match[1]}" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>`
  } else {
    return ''
  }
})

const linkModal = ref<InstanceType<typeof Modal> | null>(null)
const imageModal = ref<InstanceType<typeof Modal> | null>(null)
const videoModal = ref<InstanceType<typeof Modal> | null>(null)

function openLinkModal() {
  if (editor) linkText.value = markdownCommands.yankSelection(editor)
  linkUrl.value = ''
  linkModal.value?.show()
}

function openImageModal() {
  linkText.value = ''
  linkUrl.value = ''
  imageModal.value?.show()
}

function openVideoModal() {
  linkText.value = ''
  linkUrl.value = ''
  videoModal.value?.show()
}
</script>

<style scoped>
.display-options {
  margin-bottom: var(--gap-sm);
}

.editor-action-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  overflow: hidden;
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

.markdown-body {
  border: 1px solid var(--color-button-bg);
  border-radius: var(--radius-md);
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
</style>
