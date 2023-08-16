<template>
  <Card class="log-card">
    <div class="button-row">
      <DropdownSelect
        v-model="selectedLogIndex"
        :default-value="0"
        name="Log date"
        :options="logs.map((_, index) => index)"
        :display-name="(option) => logs[option]?.name"
        :disabled="logs.length === 0"
      />
      <div class="button-group">
        <Button :disabled="!logs[selectedLogIndex]" @click="copyLog()">
          <ClipboardCopyIcon v-if="!copied" />
          <CheckIcon v-else />
          {{ copied ? 'Copied' : 'Copy' }}
        </Button>
        <Button color="primary" :disabled="offline || !logs[selectedLogIndex]" @click="share">
          <ShareIcon />
          Share
        </Button>
        <Button
          :disabled="!logs[selectedLogIndex] || logs[selectedLogIndex].live === true"
          color="danger"
          @click="deleteLog()"
        >
          <TrashIcon />
          Delete
        </Button>
      </div>
    </div>
    <div ref="logContainer" class="log-text">
      <span
        v-for="(line, index) in logs[selectedLogIndex]?.stdout.split('\n')"
        :key="index"
        class="no-wrap"
      >
        <span :style="{ color: getLineColor(line, true), 'font-weight': getLineWeight(line) }">{{
          getLinePrefix(line)
        }}</span>
        <span :style="{ color: getLineColor(line, false) }">{{ getLineText(line) }}</span>
        <br />
      </span>
    </div>
    <ShareModal
      ref="shareModal"
      header="Share Log"
      share-title="Instance Log"
      share-text="Check out this log from an instance on the Modrinth App"
      link
    />
  </Card>
</template>

<script setup>
import {
  Button,
  Card,
  CheckIcon,
  ClipboardCopyIcon,
  DropdownSelect,
  ShareIcon,
  TrashIcon,
  ShareModal,
} from 'omorphia'
import { delete_logs_by_datetime, get_logs, get_output_by_datetime } from '@/helpers/logs.js'
import { nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import dayjs from 'dayjs'
import calendar from 'dayjs/plugin/calendar'
import { get_output_by_uuid, get_uuids_by_profile_path } from '@/helpers/process.js'
import { useRoute } from 'vue-router'
import { useLogs } from '@/store/logs'
import { process_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'
import { ofetch } from 'ofetch'

dayjs.extend(calendar)

const route = useRoute()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
  offline: {
    type: Boolean,
    default: false,
  },
})

const logsColored = useLogs().logsColored
const logs = ref([])
await setLogs()

const selectedLogIndex = ref(0)
const copied = ref(false)
const logContainer = ref(null)
const interval = ref(null)
const userScrolled = ref(false)
const isAutoScrolling = ref(false)
const shareModal = ref(null)

async function getLiveLog() {
  if (route.params.id) {
    const uuids = await get_uuids_by_profile_path(route.params.id).catch(handleError)
    let returnValue
    if (uuids.length === 0) {
      returnValue = 'No live game detected. \nStart your game to proceed'
    } else {
      returnValue = await get_output_by_uuid(uuids[0]).catch(handleError)
    }

    return { name: 'Live Log', stdout: returnValue, live: true }
  }
  return null
}

async function getLogs() {
  return (await get_logs(props.instance.path, true).catch(handleError)).reverse().map((log) => {
    log.name = dayjs(
      log.datetime_string.slice(0, 8) + 'T' + log.datetime_string.slice(9)
    ).calendar()
    log.stdout = 'Loading...'
    return log
  })
}

async function setLogs() {
  const [liveLog, allLogs] = await Promise.all([getLiveLog(), getLogs()])
  logs.value = [liveLog, ...allLogs]
}

const copyLog = () => {
  if (logs.value.length > 0 && logs.value[selectedLogIndex.value]) {
    navigator.clipboard.writeText(logs.value[selectedLogIndex.value].stdout)
    copied.value = true
  }
}

const share = async () => {
  if (logs.value.length > 0 && logs.value[selectedLogIndex.value]) {
    const url = await ofetch('https://api.mclo.gs/1/log', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: `content=${encodeURIComponent(logs.value[selectedLogIndex.value].stdout)}`,
    }).catch(handleError)

    shareModal.value.show(url.url)
  }
}

watch(selectedLogIndex, async (newIndex) => {
  copied.value = false
  userScrolled.value = false

  if (logs.value.length > 1 && newIndex !== 0) {
    logs.value[newIndex].stdout = 'Loading...'
    logs.value[newIndex].stdout = await get_output_by_datetime(
      props.instance.path,
      logs.value[newIndex].datetime_string
    ).catch(handleError)
  }
})

if (logs.value.length >= 1) {
  selectedLogIndex.value = 1
}

const deleteLog = async () => {
  if (logs.value[selectedLogIndex.value] && selectedLogIndex.value !== 0) {
    let deleteIndex = selectedLogIndex.value
    selectedLogIndex.value = deleteIndex - 1
    await delete_logs_by_datetime(
      props.instance.path,
      logs.value[deleteIndex].datetime_string
    ).catch(handleError)
    await setLogs()
  }
}

const isLineLevel = (text, level) => {
  if ((text.includes('/INFO') || text.includes('[System] [CHAT]')) && level === 'info') {
    return true
  }

  if (text.includes('/WARN') && level === 'warn') {
    return true
  }

  const errorTriggers = ['/ERROR', 'Exception', ':?]', 'Error', '[thread']
  if (level === 'error') {
    for (const trigger of errorTriggers) {
      if (text.includes(trigger)) return true
    }
  }

  if (text.trim()[0] === '#' && level === 'comment') {
    return true
  }
  return false
}

const getLineWeight = (text) => {
  if (!logsColored || isLineLevel(text, 'info')) {
    return 'normal'
  }

  if (isLineLevel(text, 'error') || isLineLevel(text, 'warn')) {
    return 'bold'
  }
}

const getLineColor = (text, prefix) => {
  if (!logsColored || text.includes('[System] [CHAT]')) {
    return 'white'
  }
  if (isLineLevel(text, 'info') && prefix) {
    return '#00a8ff'
  }
  if (isLineLevel(text, 'warn')) {
    return '#ff6625'
  }
  if (isLineLevel(text, 'error')) {
    return '#f62451'
  }
  if (isLineLevel(text, 'comment')) {
    return '#1dd1a1'
  }
}

const getLinePrefix = (text) => {
  if (text.includes(']:')) {
    return text.split(']:')[0] + ']:'
  }
}

const getLineText = (text) => {
  if (text.includes(']:')) {
    if (text.split(']:').length > 2) {
      return text.split(']:').slice(1).join(']:')
    }
    return text.split(']:')[1]
  } else {
    return text
  }
}

function handleUserScroll() {
  if (!isAutoScrolling.value) {
    userScrolled.value = true
  }
}

interval.value = setInterval(async () => {
  if (logs.value.length > 0) {
    logs.value[0] = await getLiveLog()

    // Allow resetting of userScrolled if the user scrolls to the bottom
    if (selectedLogIndex.value === 0) {
      if (
        logContainer.value.scrollTop + logContainer.value.offsetHeight >=
        logContainer.value.scrollHeight - 10
      )
        userScrolled.value = false

      if (!userScrolled.value) {
        await nextTick()
        isAutoScrolling.value = true
        logContainer.value.scrollTop =
          logContainer.value.scrollHeight - logContainer.value.offsetHeight
        setTimeout(() => (isAutoScrolling.value = false), 50)
      }
    }
  }
}, 250)

const unlistenProcesses = await process_listener(async (e) => {
  if (e.event === 'launched') {
    selectedLogIndex.value = 0
  }
  if (e.event === 'finished') {
    userScrolled.value = false
    await setLogs()
    selectedLogIndex.value = 1
  }
})

onMounted(() => {
  logContainer.value.addEventListener('scroll', handleUserScroll)
})

onBeforeUnmount(() => {
  logContainer.value.removeEventListener('scroll', handleUserScroll)
})
onUnmounted(() => {
  clearInterval(interval.value)
  unlistenProcesses()
})
</script>

<style scoped lang="scss">
.log-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: calc(100vh - 11rem);
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: 0.5rem;
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.log-text {
  width: 100%;
  height: 100%;
  font-family: var(--mono-font);
  background-color: var(--color-accent-contrast);
  color: var(--color-contrast);
  border-radius: var(--radius-lg);
  padding: 1.5rem;
  overflow: auto;
  white-space: normal;
  color-scheme: dark;

  .no-wrap {
    white-space: pre;
  }
}

:deep(button.checkbox) {
  border: none;
}
</style>
