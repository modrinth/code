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
          v-if="logs[selectedLogIndex] && logs[selectedLogIndex].live === true"
          @click="clearLiveLog()"
        >
          <TrashIcon />
          Clear
        </Button>

        <Button
          v-else
          :disabled="!logs[selectedLogIndex] || logs[selectedLogIndex].live === true"
          color="danger"
          @click="deleteLog()"
        >
          <TrashIcon />
          Delete
        </Button>
      </div>
    </div>
    <div class="button-row">
      <input
        id="text-filter"
        v-model="searchFilter"
        autocomplete="off"
        type="text"
        class="text-filter"
        placeholder="Type to filter logs..."
      />
      <div class="filter-group">
        <Checkbox
          v-for="level in levels"
          :key="level.toLowerCase()"
          v-model="levelFilters[level.toLowerCase()]"
          class="filter-checkbox"
        >
          {{ level }}</Checkbox
        >
      </div>
    </div>
    <div ref="logContainer" class="log-text">
      <span
        v-for="(processedLine, index) in processedLogs.filter((l) => shouldDisplay(l))"
        :key="index"
        :class="processedLine.class"
      >
        {{ processedLine.line }}<br />
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
  Checkbox,
  TrashIcon,
  ShareModal,
} from 'omorphia'
import {
  delete_logs_by_filename,
  get_logs,
  get_output_by_filename,
  get_latest_log_cursor,
} from '@/helpers/logs.js'
import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import dayjs from 'dayjs'
import isToday from 'dayjs/plugin/isToday'
import isYesterday from 'dayjs/plugin/isYesterday'
import { get_uuids_by_profile_path } from '@/helpers/process.js'
import { useRoute } from 'vue-router'
import { process_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'
import { ofetch } from 'ofetch'

dayjs.extend(isToday)
dayjs.extend(isYesterday)

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
  playing: {
    type: Boolean,
    default: false,
  },
})

const currentLiveLog = ref(null)
const currentLiveLogCursor = ref(0)
const emptyText = ['No live game detected.', 'Start your game to proceed']

const logs = ref([])
await setLogs()

const selectedLogIndex = ref(0)
const copied = ref(false)
const logContainer = ref(null)
const interval = ref(null)
const userScrolled = ref(false)
const isAutoScrolling = ref(false)
const shareModal = ref(null)

const levels = ['Fatal', 'Error', 'Warn', 'Info', 'Debug', 'Trace']
const levelFilters = ref({})
levels.forEach((level) => {
  levelFilters.value[level.toLowerCase()] = true
})
const searchFilter = ref('')

function shouldDisplay(line) {
  if (line.forceShow) {
    return true
  }
  if (!levelFilters.value[line.level.toLowerCase()]) {
    return false
  }
  if (searchFilter.value !== '') {
    if (!line.line.toLowerCase().includes(searchFilter.value.toLowerCase())) {
      return false
    }
  }
  return true
}

const processedLogs = computed(() => {
  const lines = logs.value[selectedLogIndex.value]?.stdout.split('\n') || []
  const processed = []
  let currentClass = ''
  const regex = /\[(\d{2}:\d{2}:\d{2})\] \[.*\/([a-zA-Z]+)\]/

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const match = line.match(regex)
    let level = ''
    let timestamp = ''
    if (match) {
      timestamp = match[1] // Captured timestamp
      level = match[2] // Captured log level, could be any alphabetic string
      if (level === 'WARN') {
        currentClass = 'yellow-color'
      } else if (level === 'ERROR' || level === 'FATAL') {
        currentClass = 'red-color'
      } else if (level === 'INFO' || level === 'DEBUG' || level === 'TRACE') {
        currentClass = ''
      }
    }

    let forceShow = false
    if (!match && emptyText.includes(line) && selectedLogIndex.value === 0) {
      forceShow = true
    }

    processed.push({ line, class: currentClass + ' no-wrap', timestamp, level, forceShow })
  }

  return processed
})

async function getLiveLog() {
  if (route.params.id) {
    const uuids = await get_uuids_by_profile_path(route.params.id).catch(handleError)
    let returnValue
    if (uuids.length === 0) {
      returnValue = emptyText.join('\n')
    } else {
      const logCursor = await get_latest_log_cursor(
        props.instance.path,
        currentLiveLogCursor.value
      ).catch(handleError)
      if (logCursor.new_file) {
        currentLiveLog.value = ''
      }
      currentLiveLog.value = currentLiveLog.value + logCursor.output
      currentLiveLogCursor.value = logCursor.cursor
      returnValue = currentLiveLog.value
    }
    return { name: 'Live Log', stdout: returnValue, live: true }
  }
  return null
}

async function getLogs() {
  return (await get_logs(props.instance.path, true).catch(handleError)).reverse().map((log) => {
    if (log.filename == 'latest.log') {
      log.name = 'Latest Log'
    } else {
      let filename = log.filename.split('.')[0]
      let day = dayjs(filename.slice(0, 10))
      if (day.isValid()) {
        if (day.isToday()) {
          log.name = 'Today'
        } else if (day.isYesterday()) {
          log.name = 'Yesterday'
        } else {
          log.name = day.format('MMMM D, YYYY')
        }
        // Displays as "Today-1", "Today-2", etc, matching minecraft log naming but with the date
        log.name = log.name + filename.slice(10)
      } else {
        log.name = filename
      }
    }
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
    logs.value[newIndex].stdout = await get_output_by_filename(
      props.instance.path,
      logs.value[newIndex].filename
    ).catch(handleError)
  }
})

if (logs.value.length > 1 && !props.playing) {
  selectedLogIndex.value = 1
} else {
  selectedLogIndex.value = 0
}

const deleteLog = async () => {
  if (logs.value[selectedLogIndex.value] && selectedLogIndex.value !== 0) {
    let deleteIndex = selectedLogIndex.value
    selectedLogIndex.value = deleteIndex - 1
    await delete_logs_by_filename(props.instance.path, logs.value[deleteIndex].filename).catch(
      handleError
    )
    await setLogs()
  }
}

const clearLiveLog = async () => {
  currentLiveLog.value = ''
  // does not reset cursor
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
    currentLiveLog.value = ''
    currentLiveLogCursor.value = 0
    selectedLogIndex.value = 0
  }
  if (e.event === 'finished') {
    currentLiveLog.value = ''
    currentLiveLogCursor.value = 0
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

.red-color {
  color: red;
}
.yellow-color {
  color: yellow;
}
.filter-checkbox {
  margin-bottom: 0.3rem;
  font-size: 1rem;

  svg {
    display: flex;
    align-self: center;
    justify-self: center;
  }
}
.filter-group {
  display: flex;
  padding: 0.6rem;
  flex-direction: row;
  gap: 0.5rem;
}
</style>
