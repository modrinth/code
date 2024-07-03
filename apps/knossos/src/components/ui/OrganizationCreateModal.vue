<template>
  <Modal ref="modal" header="Create an organization">
    <div class="universal-modal modal-creation universal-labels">
      <div class="markdown-body">
        <p>
          Organizations can be found under your profile page. You will be set as its owner, but you
          can invite other members and transfer ownership at any time.
        </p>
      </div>
      <label for="name">
        <span class="label__title">Name<span class="required">*</span></span>
      </label>
      <input
        id="name"
        v-model="name"
        type="text"
        maxlength="64"
        :placeholder="`Enter organization name...`"
        autocomplete="off"
        @input="updateSlug()"
      />
      <label for="slug">
        <span class="label__title">URL<span class="required">*</span></span>
      </label>
      <div class="text-input-wrapper">
        <div class="text-input-wrapper__before">https://modrinth.com/organization/</div>
        <input
          id="slug"
          v-model="slug"
          type="text"
          maxlength="64"
          autocomplete="off"
          @input="manualSlug = true"
        />
      </div>
      <label for="additional-information">
        <span class="label__title">Summary<span class="required">*</span></span>
        <span class="label__description">This will appear on your organization's page.</span>
      </label>
      <div class="textarea-wrapper">
        <textarea id="additional-information" v-model="description" maxlength="256" />
      </div>
      <div class="push-right input-group">
        <Button @click="modal.hide()">
          <CrossIcon />
          Cancel
        </Button>
        <Button color="primary" @click="createProject">
          <CheckIcon />
          Continue
        </Button>
      </div>
    </div>
  </Modal>
</template>
<script setup>
import { XIcon as CrossIcon, CheckIcon, Modal, Button } from 'omorphia'

const router = useNativeRouter()

const name = ref('')
const slug = ref('')
const description = ref('')
const manualSlug = ref(false)

const modal = ref()

async function createProject() {
  startLoading()
  try {
    const value = {
      name: name.value.trim(),
      description: description.value.trim(),
      slug: slug.value.trim().replace(/ +/g, ''),
    }

    const result = await useBaseFetch('organization', {
      method: 'POST',
      body: JSON.stringify(value),
      apiVersion: 3,
    })

    modal.value.hide()

    await router.push(`/organization/${result.slug}`)
  } catch (err) {
    console.error(err)
    addNotification({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}
function show() {
  name.value = ''
  description.value = ''
  modal.value.show()
}

function updateSlug() {
  if (!manualSlug.value) {
    slug.value = name.value
      .trim()
      .toLowerCase()
      .replaceAll(' ', '-')
      .replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '')
      .replaceAll(/--+/gm, '-')
  }
}

defineExpose({
  show,
})
</script>

<style scoped lang="scss">
.modal-creation {
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
    margin-top: var(--gap-md);
  }
}
</style>
