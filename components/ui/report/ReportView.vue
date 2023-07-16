<template>
  <div>
    <section class="universal-card">
      <Breadcrumbs
        v-if="breadcrumbsStack"
        :current-title="`Report ${reportId}`"
        :link-stack="breadcrumbsStack"
      />
      <h2>Report details</h2>
      <ReportInfo :report="report" :show-thread="false" :show-message="false" />
    </section>
    <section class="universal-card">
      <h2>Messages</h2>
      <ConversationThread :thread="thread" :report="report" :update-thread="updateThread" />
    </section>
  </div>
</template>
<script setup>
import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import ReportInfo from '~/components/ui/report/ReportInfo.vue'
import { addReportMessage } from '~/helpers/threads.js'

const props = defineProps({
  reportId: {
    type: String,
    required: true,
  },
  breadcrumbsStack: {
    type: Array,
    default: null,
  },
})

const app = useNuxtApp()

const report = ref(null)

await fetchReport().then((result) => {
  report.value = result
})

const { data: rawThread } = await useAsyncData(`thread/${report.value.thread_id}`, () =>
  useBaseFetch(`thread/${report.value.thread_id}`, app.$defaultHeaders())
)
const thread = computed(() => addReportMessage(rawThread.value, report.value))

async function updateThread(newThread) {
  rawThread.value = newThread
  report.value = await fetchReport()
}

async function fetchReport() {
  const { data: rawReport } = await useAsyncData(`report/${props.reportId}`, () =>
    useBaseFetch(`report/${props.reportId}`, app.$defaultHeaders())
  )
  rawReport.value.item_id = rawReport.value.item_id.replace(/"/g, '')

  const userIds = []
  userIds.push(rawReport.value.reporter)
  if (rawReport.value.item_type === 'user') {
    userIds.push(rawReport.value.item_id)
  }

  const versionId = rawReport.value.item_type === 'version' ? rawReport.value.item_id : null

  let users = []
  if (userIds.length > 0) {
    const { data: usersVal } = await useAsyncData(`users?ids=${JSON.stringify(userIds)}`, () =>
      useBaseFetch(`users?ids=${JSON.stringify(userIds)}`, app.$defaultHeaders())
    )
    users = usersVal.value
  }

  let version = null
  if (versionId) {
    const { data: versionVal } = await useAsyncData(`version/${versionId}`, () =>
      useBaseFetch(`version/${versionId}`, app.$defaultHeaders())
    )
    version = versionVal.value
  }

  const projectId = version
    ? version.project_id
    : rawReport.value.item_type === 'project'
    ? rawReport.value.item_id
    : null

  let project = null
  if (projectId) {
    const { data: projectVal } = await useAsyncData(`project/${projectId}`, () =>
      useBaseFetch(`project/${projectId}`, app.$defaultHeaders())
    )
    project = projectVal.value
  }

  const reportData = rawReport.value
  reportData.project = project
  reportData.version = version
  reportData.reporterUser = users.find((user) => user.id === rawReport.value.reporter)
  if (rawReport.value.item_type === 'user') {
    reportData.user = users.find((user) => user.id === rawReport.value.item_id)
  }
  return reportData
}
</script>
<style lang="scss" scoped>
.stacked {
  display: flex;
  flex-direction: column;
}
</style>
