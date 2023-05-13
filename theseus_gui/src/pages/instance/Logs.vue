<template>
  <Card class="log-card">
    <div class="button-row">
      <DropdownSelect
        v-model="selectedLog"
        :options="logs"
        :display-name="option => option?.name"
        :disabled="logs.length === 0"
      />
      <div class="button-group">
        <Button :disabled="!selectedLog" @click="copyLog()">
          <ClipboardCopyIcon v-if="!copied"/>
          <CheckIcon v-else />
          {{ copied ? 'Copied' : 'Copy'}}
        </Button>
        <Button disabled color="primary" @click="shareLog()">
          <SendIcon />
          Share
        </Button>
        <Button :disabled="!selectedLog || selectedLog === logs[0]" color="danger" @click="deleteLog()">
          <TrashIcon />
          Delete
        </Button>
      </div>
    </div>
    <div class="log-text">
      <div v-for="line in selectedLog?.stdout.split('\n')" :key="line" class="no-wrap">
        {{ line }}
      </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, TrashIcon, SendIcon, ClipboardCopyIcon, DropdownSelect, CheckIcon } from 'omorphia'
import {delete_logs_by_datetime, get_logs} from "@/helpers/logs.js";
import {onMounted, ref, watch} from "vue";
import dayjs from "dayjs";
import {
  get_stdout_by_uuid, get_uuids_by_profile_path
} from "@/helpers/process.js";
import {useRoute} from "vue-router";

const route = useRoute()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

const getLiveLog = async () => {
  const uuids = await get_uuids_by_profile_path(route.params.id);
  if (uuids.length === 0) return ""
  return await get_stdout_by_uuid(uuids[0])
}

const logs = ref([])

try {
  logs.value = ref([
    {
      name: 'Live Log',
      stdout: await getLiveLog(),
    },
    ... await get_logs(props.instance.uuid).then(log => log.reverse()).then(log => log.map(log => {
    log.name = dayjs(log.datetime_string.slice(0, 8) + 'T' + log.datetime_string.slice(9))
    return log
  }))])
} catch (e) {
  console.error(e)
}

const selectedLog = ref(null)
const copied = ref(false)

const shareLog = () => {
  console.log("share")
}

const copyLog = () => {
  if (selectedLog.value) {
    navigator.clipboard.writeText(selectedLog.value.stdout)
    copied.value = true
  }
}

const deleteLog = async () => {
  if (selectedLog.value && selectedLog.value !== logs.value[0]) {
    await delete_logs_by_datetime(props.instance.uuid, selectedLog.value.datetime_string)
    logs.value = [
      getLiveLog(),
      ...await get_logs(props.instance.uuid).then(log => log.reverse()).then(log => log.map(log => {
      log.name = dayjs(log.datetime_string.slice(0, 8) + 'T' + log.datetime_string.slice(9))
      return log
    }))]
  }
}

watch(selectedLog, () => {
  copied.value = false
})

onMounted(() => {
  setInterval(async () => {
    if (logs.value.length > 0) {
      logs.value[0].stdout = await getLiveLog()
    }
  }, 1000)
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
