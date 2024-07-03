<template>
  <div class="page">
    <Card>
      <div class="content">
        <div>
          <h1 class="card-title-adjustments">Submit a Report</h1>
          <div>
            <p>
              Modding should be safe for everyone, so we take abuse and malicious intent seriously
              at Modrinth. If you encounter content that violates our
              <nuxt-link class="text-link" to="/legal/terms">Terms of Service</nuxt-link> or our
              <nuxt-link class="text-link" to="/legal/rules">Rules</nuxt-link>, please report it to
              us here.
            </p>
            <p>
              This form is intended exclusively for reporting abuse or harmful content to Modrinth
              staff. For bugs related to specific projects, please use the project's designated
              Issues link or Discord channel.
            </p>
            <p>
              Your privacy is important to us; rest assured that your identifying information will
              be kept confidential.
            </p>
          </div>
        </div>
        <div class="report-info-section">
          <div class="report-info-item">
            <label for="report-item">Item type to report</label>
            <DropdownSelect
              id="report-item"
              v-model="reportItem"
              name="report-item"
              :options="reportItems"
              :display-name="capitalizeString"
              :multiple="false"
              :searchable="false"
              :show-no-results="false"
              :show-labels="false"
              placeholder="Choose report item"
            />
          </div>
          <div class="report-info-item">
            <label for="report-item-id">Item ID</label>
            <input
              id="report-item-id"
              v-model="reportItemID"
              type="text"
              placeholder="ex. project ID"
              autocomplete="off"
              :disabled="reportItem === ''"
            />
          </div>
          <div class="report-info-item">
            <label for="report-type">Reason for report</label>
            <DropdownSelect
              id="report-type"
              v-model="reportType"
              name="report-type"
              :options="reportTypes"
              :multiple="false"
              :searchable="false"
              :show-no-results="false"
              :show-labels="false"
              :display-name="capitalizeString"
              placeholder="Choose report type"
            />
          </div>
        </div>
        <div class="report-submission-section">
          <div>
            <p>
              Please provide additional context about your report. Include links and images if
              possible. <strong>Empty reports will be closed.</strong>
            </p>
          </div>
          <MarkdownEditor v-model="reportBody" placeholder="" :on-image-upload="onImageUpload" />
        </div>
        <div class="submit-button">
          <Button
            id="submit-button"
            color="primary"
            :disabled="submitLoading || !canSubmit"
            @click="submitReport"
          >
            <SaveIcon />
            Submit
          </Button>
        </div>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { Card, Button, MarkdownEditor, DropdownSelect, SaveIcon } from 'omorphia'
import { useImageUpload } from '~/composables/image-upload.ts'

const tags = useTags()
const route = useNativeRoute()

const accessQuery = (id: string): string => {
  return route.query?.[id]?.toString() || ''
}

const submitLoading = ref<boolean>(false)

const uploadedImageIDs = ref<string[]>([])

const reportBody = ref<string>(accessQuery('body'))
const reportItem = ref<string>(accessQuery('item'))
const reportItemID = ref<string>(accessQuery('itemID'))
const reportType = ref<string>('')

const reportItems = ['project', 'version', 'user']
const reportTypes = computed(() => tags.value.reportTypes)

const canSubmit = computed(() => {
  return (
    reportItem.value !== '' &&
    reportItemID.value !== '' &&
    reportType.value !== '' &&
    reportBody.value !== ''
  )
})

const submissionValidation = () => {
  if (!canSubmit.value) {
    throw new Error('Please fill out all required fields')
  }

  if (reportItem.value === '') {
    throw new Error('Please select a report item')
  }

  if (reportItemID.value === '') {
    throw new Error('Please enter a report item ID')
  }

  if (reportType.value === '') {
    throw new Error('Please select a report type')
  }

  if (reportBody.value === '') {
    throw new Error('Please enter a report body')
  }

  return true
}

const capitalizeString = (value?: string) => {
  if (!value) return ''
  return value?.charAt(0).toUpperCase() + value?.slice(1)
}

const submitReport = async () => {
  submitLoading.value = true

  let data: {
    [key: string]: unknown
  } = {
    report_type: reportType.value,
    item_type: reportItem.value,
    item_id: reportItemID.value,
    body: reportBody.value,
  }

  function takeNLast<T>(arr: T[], n: number): T[] {
    return arr.slice(Math.max(arr.length - n, 0))
  }

  if (uploadedImageIDs.value.length > 0) {
    data = {
      ...data,
      uploaded_images: takeNLast(uploadedImageIDs.value, 10),
    }
  }

  try {
    submissionValidation()
  } catch (error) {
    submitLoading.value = false

    if (error instanceof Error) {
      addNotification({
        group: 'main',
        title: 'An error occurred',
        text: error.message,
        type: 'error',
      })
    }

    return
  }

  try {
    const response = (await useBaseFetch('report', {
      method: 'POST',
      body: data,
    })) as { id: string }

    submitLoading.value = false

    if (response?.id) {
      navigateTo(`/dashboard/report/${response.id}`)
    }
  } catch (error) {
    submitLoading.value = false

    if (error instanceof Error) {
      addNotification({
        group: 'main',
        title: 'An error occurred',
        text: error.message,
        type: 'error',
      })
    }

    throw error
  }
}

const onImageUpload = async (file: File) => {
  const item = await useImageUpload(file, { context: 'report' })
  uploadedImageIDs.value.push(item.id)
  return item.url
}
</script>

<style scoped lang="scss">
.submit-button {
  display: flex;
  justify-content: flex-end;
  width: 100%;

  margin-top: var(--spacing-card-md);
}

.card-title-adjustments {
  margin-block: var(--spacing-card-md) var(--spacing-card-sm);
}

.page {
  padding: 0.5rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 56rem;
}

.content {
  // TODO: Get rid of this hack when removing global styles from the website.
  // Overflow decides the behavior of md editor but also clips the border.
  // In the future, we should use ring instead of block-shadow for the
  // green ring around the md editor
  padding-inline: var(--gap-md);
  padding-bottom: var(--gap-md);
  margin-inline: calc(var(--gap-md) * -1);

  display: grid;

  // Disable horizontal stretch
  grid-template-columns: minmax(0, 1fr);
  overflow: hidden;
}

.report-info-section {
  display: block;

  width: 100%;
  gap: var(--gap-md);

  :global(.animated-dropdown) {
    & > .selected {
      height: 40px;
    }
  }

  .report-info-item {
    display: block;

    width: 100%;
    max-width: 100%;

    label {
      display: block;
      margin-bottom: var(--gap-sm);
      color: var(--color-text-dark);
      font-size: var(--font-size-md);
      font-weight: var(--font-weight-bold);
      margin-block: var(--spacing-card-md) var(--spacing-card-sm);
    }
  }
}
</style>
