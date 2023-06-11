<template>
  <div>
    <section class="universal-card">
      <label for="project-description">
        <span class="label__title size-card-header">Description</span>
        <span class="label__description">
          You can type an extended description of your mod here. This editor supports
          <a
            class="text-link"
            href="https://guides.github.com/features/mastering-markdown/"
            target="_blank"
            rel="noopener"
            >Markdown</a
          >. HTML can also be used inside your description, not including styles, scripts, and
          iframes (though YouTube iframes are allowed).
          <span class="label__subdescription">
            The description must clearly and honestly describe the purpose and function of the
            project. See section 2.1 of the
            <nuxt-link to="/legal/rules" class="text-link" target="_blank">Content Rules</nuxt-link>
            for the full requirements.
          </span>
        </span>
      </label>
      <Chips v-model="bodyViewMode" :items="['source', 'preview']" />
      <div v-if="bodyViewMode === 'source'" class="resizable-textarea-wrapper">
        <textarea
          id="project-description"
          v-model="description"
          :disabled="(currentMember.permissions & EDIT_BODY) !== EDIT_BODY"
        />
      </div>
      <div
        v-else-if="bodyViewMode === 'preview'"
        class="markdown-body"
        v-html="description ? renderHighlightedString(description) : 'No body specified.'"
      />
      <div class="input-group">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="!hasChanges"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </section>
  </div>
</template>

<script>
import Chips from '~/components/ui/Chips.vue'
import SaveIcon from '~/assets/images/utils/save.svg'
import { renderHighlightedString } from '~/helpers/highlight.js'

export default defineNuxtComponent({
  components: {
    Chips,
    SaveIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
    allMembers: {
      type: Array,
      default() {
        return []
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
    patchProject: {
      type: Function,
      default() {
        return () => {
          this.$notify({
            group: 'main',
            title: 'An error occurred',
            text: 'Patch project function not found',
            type: 'error',
          })
        }
      },
    },
  },
  data() {
    return {
      description: this.project.body,
      bodyViewMode: 'source',
    }
  },
  computed: {
    patchData() {
      const data = {}

      if (this.description !== this.project.body) {
        data.body = this.description
      }

      return data
    },
    hasChanges() {
      return Object.keys(this.patchData).length > 0
    },
  },
  created() {
    this.EDIT_BODY = 1 << 3
  },
  methods: {
    renderHighlightedString,
    saveChanges() {
      if (this.hasChanges) {
        this.patchProject(this.patchData)
      }
    },
  },
})
</script>
<style lang="scss" scoped>
.resizable-textarea-wrapper textarea {
  min-height: 40rem;
}

.markdown-body {
  margin-bottom: var(--spacing-card-md);
}
</style>
