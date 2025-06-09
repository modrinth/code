<template>
  <NewModal
    ref="modal"
    :header="mode === 'edit' ? 'Edit Schedule' : 'Create Schedule'"
    class="schedule-modal"
  >
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-2">
        <label for="title">
          <span class="text-lg font-semibold text-contrast">
            Title
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          id="title"
          v-model="data.title"
          type="text"
          placeholder="Enter schedule title"
          autocomplete="off"
        />
      </div>

      <div class="flex flex-col gap-2">
        <label for="action_kind">
          <span class="text-lg font-semibold text-contrast">
            Action Type
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <DropdownSelect
          id="action_kind"
          v-model="data.action_kind"
          :options="actionTypes"
          name="Action Type"
        />
        <div class="text-sm text-secondary action-type-description">
          {{ getActionTypeDescription(data.action_kind) }}
        </div>
      </div>

      <div v-if="data.action_kind === 'game-command'" class="flex flex-col gap-2">
        <label for="command">
          <span class="text-lg font-semibold text-contrast">
            Command
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          id="command"
          v-model="data.options.command"
          type="text"
          placeholder="/give @a diamond 64"
          autocomplete="off"
        />
      </div>

      <div class="flex flex-col gap-4">
        <label class="text-lg font-semibold text-contrast">Schedule</label>

        <RadioButtons v-model="scheduleType" :items="['daily', 'custom']" force-selection>
          <template #default="{ item }">
            <div class="flex items-center gap-2">
              <ClockIcon v-if="item === 'daily'" class="h-4 w-4" />
              <CodeIcon v-if="item === 'custom'" class="h-4 w-4" />
              {{ item === 'daily' ? 'Every X day(s) at specific time' : 'Custom cron expression' }}
            </div>
          </template>
        </RadioButtons>

        <!-- Fixed width container for schedule cards -->
        <div class="schedule-card-container">
          <div v-if="scheduleType === 'daily'" class="card p-4 space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="flex flex-col gap-2">
                <label for="dayInterval">
                  <span class="text-lg font-semibold text-contrast">Every X day(s)</span>
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="dayInterval"
                    v-model="dayInterval"
                    type="text"
                    inputmode="numeric"
                    placeholder="1"
                    class="w-20"
                    maxlength="3"
                    @input="handleDayIntervalChange"
                  />
                  <span class="text-sm text-secondary">
                    day{{ parseInt(dayInterval || '1') > 1 ? 's' : '' }}
                  </span>
                </div>
                <div class="text-xs text-secondary">1-365 days</div>
              </div>
              <div class="flex flex-col gap-2">
                <label class="text-lg font-semibold text-contrast">At time</label>
                <TimePicker v-model="selectedTime" placeholder="Select time" />
              </div>
            </div>
            <div class="pt-2 border-t border-divider">
              <p class="text-sm text-secondary">{{ getCronDescription(cronString) }}</p>
            </div>
          </div>

          <div v-if="scheduleType === 'custom'" class="card p-4 space-y-4">
            <div class="flex flex-col gap-2">
              <label for="customCron">
                <span class="text-lg font-semibold text-contrast">Cron Expression</span>
              </label>
              <div class="textarea-wrapper">
                <textarea
                  id="customCron"
                  v-model="customCron"
                  placeholder="0 0 9 * * *"
                  class="font-mono"
                  rows="2"
                />
              </div>
              <div class="text-xs text-secondary">
                Format: seconds minutes hours dayOfMonth month dayOfWeek
              </div>
              <div v-if="!isValidCron" class="text-xs text-brand-red">
                Invalid cron format. Please use 6 space-separated values.
              </div>
            </div>
            <div class="pt-2 border-t border-divider">
              <p class="text-sm text-secondary">{{ getCronDescription(customCron) }}</p>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-2">
        <label for="warn_msg">
          <span class="text-lg font-semibold text-contrast">Warning Message (Optional)</span>
        </label>
        <div class="textarea-wrapper">
          <textarea
            id="warn_msg"
            v-model="data.warn_msg"
            placeholder="/tellraw Warning message with {} placeholder"
            rows="2"
          />
        </div>
      </div>

      <div v-if="data.warn_msg && data.warn_msg.trim() !== ''" class="flex flex-col gap-2">
        <label class="text-lg font-semibold text-contrast">Warning Intervals</label>
        <div class="flex gap-2">
          <input v-model="newInterval" type="number" placeholder="Seconds" min="1" class="flex-1" />
          <ButtonStyled
            :disabled="!newInterval || (data.warn_intervals?.length || 0) >= 6"
            @click="addWarningInterval"
          >
            <PlusIcon class="h-4 w-4" />
          </ButtonStyled>
        </div>
        <!-- Fixed height container for warning intervals -->
        <div class="warning-intervals-container">
          <div
            v-if="data.warn_intervals && data.warn_intervals.length > 0"
            class="warning-intervals-content"
          >
            <RaisedBadge
              v-for="(interval, index) in data.warn_intervals"
              :key="index"
              :text="formatInterval(interval)"
              class="flex items-center gap-1"
            >
              <button
                type="button"
                class="ml-1 hover:text-brand-red"
                @click="removeWarningInterval(index)"
              >
                <XIcon class="h-3 w-3" />
              </button>
            </RaisedBadge>
          </div>
        </div>
        <p class="text-xs text-secondary">Maximum 6 intervals. Values in seconds.</p>
      </div>

      <div class="flex gap-2">
        <ButtonStyled
          color="brand"
          :disabled="isLoading || !isValid || !hasChanges"
          @click="handleSubmit"
        >
          <button>
            <PlusIcon v-if="mode === 'new'" class="h-4 w-4" />
            <EditIcon v-else class="h-4 w-4" />
            {{ isLoading ? 'Saving...' : mode === 'edit' ? 'Update' : 'Create' }}
          </button>
        </ButtonStyled>
        <ButtonStyled :disabled="isLoading" @click="modal?.hide()">
          <button>
            <XIcon class="h-4 w-4" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NewModal,
  ButtonStyled,
  DropdownSelect,
  RaisedBadge,
  TimePicker,
  RadioButtons,
} from '@modrinth/ui'
import { XIcon, PlusIcon, EditIcon, ClockIcon, CodeIcon } from '@modrinth/assets'
import cronstrue from 'cronstrue'
import type { ScheduledTask } from '@modrinth/utils'

interface TimeValue {
  hour: string
  minute: string
}

const emit = defineEmits<{
  save: [data: ScheduledTask]
}>()

const modal = ref()
const isLoading = ref(false)
const mode = ref<'new' | 'edit'>('new')
const originalData = ref<ScheduledTask | null>(null)

const data = ref<ScheduledTask>({
  title: '',
  action_kind: 'game-command',
  options: {},
  enabled: true,
  warn_msg: '',
  warn_intervals: [],
  every: '0 0 9 * * *',
})

const newInterval = ref('')

const scheduleType = ref<'daily' | 'custom'>('daily')
const dayInterval = ref('1')
const selectedTime = ref<TimeValue>({ hour: '9', minute: '0' })
const customCron = ref('0 0 9 * * *')

const actionTypes = ref(['game-command', 'restart'])

const cronString = computed(() => generateCronString())

const isValidCron = computed(() => {
  return scheduleType.value === 'custom' ? validateCustomCron(customCron.value) : true
})

const isValid = computed(() => {
  if (!data.value.title.trim()) return false

  if (!isValidCron.value) return false

  if (
    data.value.action_kind === 'game-command' &&
    (!data.value.options.command || !data.value.options.command.trim())
  ) {
    return false
  }

  return true
})

const hasChanges = computed(() => {
  if (mode.value === 'new') return true

  if (!originalData.value) return false

  if (data.value.title !== originalData.value.title) return true
  if (data.value.action_kind !== originalData.value.action_kind) return true
  if (data.value.every !== originalData.value.every) return true
  if (data.value.warn_msg !== originalData.value.warn_msg) return true

  if (
    data.value.action_kind === 'game-command' &&
    data.value.options.command !== originalData.value.options?.command
  ) {
    return true
  }

  const originalIntervals = originalData.value.warn_intervals || []
  const currentIntervals = data.value.warn_intervals || []

  if (originalIntervals.length !== currentIntervals.length) return true

  for (let i = 0; i < originalIntervals.length; i++) {
    if (originalIntervals[i] !== currentIntervals[i]) return true
  }

  return false
})

watch(
  [scheduleType, dayInterval, selectedTime, customCron],
  () => {
    const newCron = generateCronString()
    data.value.every = newCron
  },
  { deep: true },
)

function resetData() {
  data.value = {
    title: '',
    action_kind: 'game-command',
    options: {},
    enabled: true,
    warn_msg: '',
    warn_intervals: [],
    every: '0 0 9 * * *',
  }
  scheduleType.value = 'daily'
  dayInterval.value = '1'
  selectedTime.value = { hour: '9', minute: '0' }
  customCron.value = '0 0 9 * * *'
  originalData.value = null
}

function parseCronExpression(cron: string) {
  const parts = cron.trim().split(/\s+/)
  if (parts.length === 6) {
    const [_seconds, minutes, hours, dayOfMonth, month, dayOfWeek] = parts

    if (dayOfMonth.startsWith('*/') && month === '*' && dayOfWeek === '*') {
      scheduleType.value = 'daily'
      dayInterval.value = dayOfMonth.substring(2)
      selectedTime.value = { hour: hours, minute: minutes }
    } else if (dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
      scheduleType.value = 'daily'
      dayInterval.value = '1'
      selectedTime.value = { hour: hours, minute: minutes }
    } else {
      scheduleType.value = 'custom'
      customCron.value = cron
    }
  } else {
    scheduleType.value = 'custom'
    customCron.value = cron
  }
}

function generateCronString(): string {
  if (scheduleType.value === 'custom') {
    return customCron.value
  }

  const minute = selectedTime.value.minute === '' ? '0' : selectedTime.value.minute
  const hour = selectedTime.value.hour === '' ? '0' : selectedTime.value.hour
  const days = dayInterval.value === '' ? '1' : dayInterval.value

  if (days === '1') {
    return `0 ${minute} ${hour} * * *`
  } else {
    return `0 ${minute} ${hour} */${days} * *`
  }
}

function getCronDescription(cronExpression: string): string {
  try {
    const parts = cronExpression.trim().split(/\s+/)
    if (parts.length === 6) {
      const fiveFieldCron = parts.slice(1).join(' ')
      return cronstrue.toString(fiveFieldCron, {
        throwExceptionOnParseError: false,
        verbose: false,
        use24HourTimeFormat: true,
      })
    }
    return cronstrue.toString(cronExpression, {
      throwExceptionOnParseError: false,
      verbose: false,
      use24HourTimeFormat: true,
    })
  } catch {
    return 'Invalid cron expression'
  }
}

function handleDayIntervalChange(event: Event) {
  const target = event.target as HTMLInputElement
  const cleanValue = target.value.replace(/\D/g, '')
  if (cleanValue === '' || (parseInt(cleanValue) > 0 && parseInt(cleanValue) <= 365)) {
    dayInterval.value = cleanValue
  }
}

function validateCustomCron(cron: string): boolean {
  const parts = cron.trim().split(/\s+/)
  return parts.length === 6
}

function addWarningInterval() {
  const interval = parseInt(newInterval.value)
  if (interval > 0 && data.value.warn_intervals && data.value.warn_intervals.length < 6) {
    data.value.warn_intervals = [...(data.value.warn_intervals || []), interval].sort(
      (a, b) => b - a,
    )
    newInterval.value = ''
  }
}

function removeWarningInterval(index: number) {
  data.value.warn_intervals = data.value.warn_intervals?.filter((_, i) => i !== index)
}

function formatInterval(seconds: number) {
  if (seconds >= 60) {
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = seconds % 60
    return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`
  }
  return `${seconds}s`
}

function getActionTypeDescription(actionType: string): string {
  switch (actionType) {
    case 'game-command':
      return 'Execute a custom command in the game server'
    case 'restart':
      return 'Restart the game server'
    default:
      return ''
  }
}

async function handleSubmit() {
  if (!isValid.value) return

  isLoading.value = true

  try {
    const cleanData = {
      ...data.value,
      options: data.value.action_kind === 'restart' ? {} : data.value.options,
      warn_msg: data.value.warn_msg || undefined,
      warn_intervals: data.value.warn_intervals?.length ? data.value.warn_intervals : undefined,
    }

    emit('save', cleanData)
    modal.value?.hide()
  } catch (error) {
    console.error('Failed to save schedule:', error)
  } finally {
    isLoading.value = false
  }
}

function showNew(event?: Event) {
  mode.value = 'new'
  resetData()
  modal.value?.show(event)
}

function show(task: ScheduledTask, event?: Event) {
  mode.value = 'edit'

  originalData.value = JSON.parse(JSON.stringify(task))

  data.value = {
    title: task.title,
    action_kind: task.action_kind,
    options: { ...task.options },
    enabled: task.enabled,
    warn_msg: task.warn_msg || '',
    warn_intervals: task.warn_intervals || [],
    every: task.every,
  }

  parseCronExpression(task.every)

  modal.value?.show(event)
}

defineExpose({
  show,
  showNew,
})
</script>

<style scoped lang="scss">
.schedule-modal {
  :deep(.modal-content) {
    min-width: 600px;
    max-width: 800px;
    width: 90vw;
  }
}

.card {
  background-color: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  border-radius: var(--radius-card);
}

.schedule-card-container {
  min-height: 200px;

  .card {
    width: 100%;
  }
}

.warning-intervals-container {
  min-height: 40px;
  max-height: 120px;
  overflow-y: auto;

  .warning-intervals-content {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }
}

.action-type-description {
  font-style: italic;
  min-height: 1.25rem;
}

input {
  width: 100%;
}

.textarea-wrapper {
  width: 100%;
}

textarea {
  min-height: 5rem;
  width: 100%;
}

.flex.flex-col.gap-2,
.flex.flex-col.gap-3,
.flex.flex-col.gap-4 {
  min-height: fit-content;
}
</style>
