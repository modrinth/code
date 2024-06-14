<template>
  <Modal ref="modal" header="Create a project">
    <div class="modal-creation universal-labels">
      <div class="markdown-body">
        <p>New projects are created as drafts and can be found under your profile page.</p>
      </div>
      <label for="name">
        <span class="label__title">Name<span class="required">*</span></span>
      </label>
      <input
        id="name"
        v-model="name"
        type="text"
        maxlength="64"
        placeholder="Enter project name..."
        autocomplete="off"
        @input="updatedName()"
      />
      <label for="slug">
        <span class="label__title">URL<span class="required">*</span></span>
      </label>
      <div class="text-input-wrapper">
        <div class="text-input-wrapper__before">https://modrinth.com/project/</div>
        <input
          id="slug"
          v-model="slug"
          type="text"
          maxlength="64"
          autocomplete="off"
          @input="manualSlug = true"
        />
      </div>
      <label for="visibility">
        <span class="label__title">Visibility<span class="required">*</span></span>
        <span class="label__description">
          The visibility of your project after it has been approved.
        </span>
      </label>
      <multiselect
        id="visibility"
        v-model="visibility"
        :options="visibilities"
        track-by="actual"
        label="display"
        :multiple="false"
        :searchable="false"
        :show-no-results="false"
        :show-labels="false"
        placeholder="Choose visibility.."
        open-direction="bottom"
      />
      <label for="additional-information">
        <span class="label__title">Summary<span class="required">*</span></span>
        <span class="label__description"
          >This appears in search and on the sidebar of your project's page.</span
        >
      </label>
      <div class="textarea-wrapper">
        <textarea id="additional-information" v-model="description" maxlength="256" />
      </div>
      <div class="push-right input-group">
        <button class="iconified-button" @click="cancel">
          <CrossIcon />
          Cancel
        </button>
        <button class="iconified-button brand-button" @click="createProject">
          <CheckIcon />
          Continue
        </button>
      </div>
    </div>
  </Modal>
</template>

<script>
import { Multiselect } from 'vue-multiselect'
import CrossIcon from '~/assets/images/utils/x.svg?component'
import CheckIcon from '~/assets/images/utils/right-arrow.svg?component'
import Modal from '~/components/ui/Modal.vue'

export default {
  components: {
    CrossIcon,
    CheckIcon,
    Modal,
    Multiselect,
  },
  props: {
    organizationId: {
      type: String,
      required: false,
      default: null,
    },
  },
  setup() {
    const tags = useTags()

    return { tags }
  },
  data() {
    return {
      name: '',
      slug: '',
      description: '',
      manualSlug: false,
      visibilities: [
        {
          actual: 'approved',
          display: 'Public',
        },
        {
          actual: 'private',
          display: 'Private',
        },
        {
          actual: 'unlisted',
          display: 'Unlisted',
        },
      ],
      visibility: {
        actual: 'approved',
        display: 'Public',
      },
    }
  },
  methods: {
    cancel() {
      this.$refs.modal.hide()
    },
    async createProject() {
      startLoading()

      const formData = new FormData()

      const auth = await useAuth()

      const projectData = {
        title: this.name.trim(),
        project_type: 'mod',
        slug: this.slug,
        description: this.description.trim(),
        body: '',
        requested_status: this.visibility.actual,
        initial_versions: [],
        team_members: [
          {
            user_id: auth.value.user.id,
            name: auth.value.user.username,
            role: 'Owner',
          },
        ],
        categories: [],
        client_side: 'required',
        server_side: 'required',
        license_id: 'LicenseRef-Unknown',
        is_draft: true,
      }

      if (this.organizationId) {
        projectData.organization_id = this.organizationId
      }

      formData.append('data', JSON.stringify(projectData))

      try {
        await useBaseFetch('project', {
          method: 'POST',
          body: formData,
          headers: {
            'Content-Disposition': formData,
          },
        })

        this.$refs.modal.hide()
        await this.$router.push({
          name: 'type-id',
          params: {
            type: 'project',
            id: this.slug,
          },
        })
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data.description,
          type: 'error',
        })
      }
      stopLoading()
    },
    show() {
      this.projectType = this.tags.projectTypes[0].display
      this.name = ''
      this.slug = ''
      this.description = ''
      this.manualSlug = false
      this.$refs.modal.show()
    },
    updatedName() {
      if (!this.manualSlug) {
        this.slug = this.name
          .trim()
          .toLowerCase()
          .replaceAll(' ', '-')
          .replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '')
          .replaceAll(/--+/gm, '-')
      }
    },
  },
}
</script>

<style scoped lang="scss">
.modal-creation {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;

  .markdown-body {
    margin-bottom: 0.5rem;
  }

  input {
    width: 20rem;
    max-width: 100%;
  }

  .text-input-wrapper {
    width: 100%;
  }

  textarea {
    min-height: 5rem;
  }

  .input-group {
    margin-top: var(--spacing-card-md);
  }
}
</style>
