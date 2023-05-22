<template>
  <Card class="log-card">
    <div class="button-row">
      <DropdownSelect
        name="Log date"
        :model-value="logs[selectedLogIndex]"
        :options="logs"
        :display-name="(option) => option?.name"
        :disabled="logs.length === 0"
        @change="(value) => (selectedLogIndex = value.index)"
      />
      <div class="button-group">
        <Button :disabled="!logs[selectedLogIndex]" @click="copyLog()">
          <ClipboardCopyIcon v-if="!copied" />
          <CheckIcon v-else />
          {{ copied ? 'Copied' : 'Copy' }}
        </Button>
        <Button disabled color="primary">
          <SendIcon />
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
      <!--      {{ logs[1] }}-->
      <div v-for="line in logs[selectedLogIndex]?.stdout.split('\n')" :key="line" class="no-wrap">
        {{ line }}
      </div>
    </div>
  </Card>
</template>

<script setup>
import {
  Button,
  Card,
  CheckIcon,
  ClipboardCopyIcon,
  DropdownSelect,
  SendIcon,
  TrashIcon,
} from 'omorphia'
import { delete_logs_by_datetime, get_logs, get_stdout_by_datetime } from '@/helpers/logs.js'
import { nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import dayjs from 'dayjs'
import calendar from 'dayjs/plugin/calendar'
import { get_stdout_by_uuid, get_uuids_by_profile_path } from '@/helpers/process.js'
import { useRoute } from 'vue-router'
import { process_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'

dayjs.extend(calendar)

const route = useRoute()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

async function getLiveLog() {
  const uuids = await get_uuids_by_profile_path(route.params.id).catch(handleError)
  let returnValue
  if (uuids.length === 0) {
    returnValue = 'No live game detected. \nStart your game to proceed'
  } else {
    returnValue = await get_stdout_by_uuid(uuids[0]).catch(handleError)
  }

  return { name: 'Live Log', stdout: returnValue, live: true }
}

async function getLogs() {
  return (await get_logs(props.instance.uuid, true).catch(handleError)).reverse().map((log) => {
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

const logs = ref([])
await setLogs()

const selectedLogIndex = ref(0)
const copied = ref(false)

const copyLog = () => {
  if (logs.value[selectedLogIndex.value]) {
    navigator.clipboard.writeText(logs.value[selectedLogIndex.value].stdout)
    copied.value = true
  }
}

watch(selectedLogIndex, async (newIndex) => {
  copied.value = false
  userScrolled.value = false

  if (newIndex !== 0) {
    logs.value[newIndex].stdout = 'Loading...'
    logs.value[newIndex].stdout = await get_stdout_by_datetime(
      props.instance.uuid,
      logs.value[newIndex].datetime_string
    ).catch(handleError)
  }
})

const deleteLog = async () => {
  if (logs.value[selectedLogIndex.value] && selectedLogIndex.value !== 0) {
    let deleteIndex = selectedLogIndex.value
    selectedLogIndex.value = deleteIndex - 1
    await delete_logs_by_datetime(
      props.instance.uuid,
      logs.value[deleteIndex].datetime_string
    ).catch(handleError)
    await setLogs()
  }
}

const logContainer = ref(null)
const interval = ref(null)
const userScrolled = ref(false)
const isAutoScrolling = ref(false)

function handleUserScroll() {
  if (!isAutoScrolling.value) {
    userScrolled.value = true
  }
}

interval.value = setInterval(async () => {
  if (logs.value.length > 0) {
    logs.value[0] = await getLiveLog()

    if (selectedLogIndex.value === 0 && !userScrolled.value) {
      await nextTick()
      isAutoScrolling.value = true
      logContainer.value.scrollTop =
        logContainer.value.scrollHeight - logContainer.value.offsetHeight
      setTimeout(() => (isAutoScrolling.value = false), 50)
    }
  }
}, 250)

const unlistenProcesses = await process_listener(async (e) => {
  if (e.event === 'finished') {
    userScrolled.value = false
    await setLogs()
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
}
</style>
