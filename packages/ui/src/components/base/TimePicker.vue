<template>
  <Dropdown
    ref="dropdown"
    :disabled="disabled"
    placement="bottom-start"
    theme="ribbit-popout"
    :distance="8"
    no-arrow
    @apply-show="onDropdownShow"
  >
    <ButtonStyled
      :class="['w-full justify-start text-left font-normal', !modelValue && 'text-secondary']"
      :disabled="disabled"
    >
      <button>
        <ClockIcon class="h-4 w-4" />
        {{ modelValue ? formatTime(modelValue) : placeholder }} {{ useUtcValues ? 'UTC' : '' }}
      </button>
    </ButtonStyled>

    <template #popper>
      <div class="flex flex-col gap-4 p-4 w-64">
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-2">
            <label for="hour" class="text-sm font-medium text-contrast"> Hour </label>
            <input
              id="hour"
              v-model="currentTime.hour"
              type="text"
              inputmode="numeric"
              placeholder="00"
              class="bg-bg-input w-full rounded-lg p-2 text-center"
              maxlength="2"
              @input="handleHourChange"
            />
            <div class="text-xs text-secondary text-center">0-23</div>
          </div>
          <div class="flex flex-col gap-2">
            <label for="minute" class="text-sm font-medium text-contrast"> Minute </label>
            <input
              id="minute"
              v-model="currentTime.minute"
              type="text"
              inputmode="numeric"
              placeholder="00"
              class="bg-bg-input w-full rounded-lg p-2 text-center"
              maxlength="2"
              @input="handleMinuteChange"
            />
            <div class="text-xs text-secondary text-center">0-59</div>
          </div>
        </div>

        <div class="text-center p-3 bg-bg-raised rounded-lg">
          <div class="text-sm text-secondary">Selected Time (Local)</div>
          <div class="text-2xl font-bold text-contrast">
            {{ formatTime(currentTime) }}
          </div>
          <div class="text-xs text-secondary mt-1">
            {{ getUTCTime(currentTime) }}
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <div class="text-sm font-medium text-contrast">Quick Select</div>
          <div class="grid grid-cols-2 gap-2">
            <ButtonStyled v-for="preset in quickSelects" :key="preset.label">
              <button class="" @click="handleQuickSelect(preset.hour, preset.minute)">
                {{ preset.label }}
              </button>
            </ButtonStyled>
          </div>
        </div>

        <div class="flex gap-2 pt-2">
          <ButtonStyled class="flex-1">
            <button @click="handleCancel">Cancel</button>
          </ButtonStyled>
          <ButtonStyled color="brand" class="flex-1">
            <button @click="handleDone">Done</button>
          </ButtonStyled>
        </div>
      </div>
    </template>
  </Dropdown>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { Dropdown } from 'floating-vue'
import { ButtonStyled } from '@modrinth/ui'
import { ClockIcon } from '@modrinth/assets'

interface TimeValue {
  hour: string
  minute: string
}

interface Props {
  modelValue?: TimeValue
  disabled?: boolean
  placeholder?: string
  useUtcValues?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => ({ hour: '12', minute: '00' }),
  disabled: false,
  placeholder: 'Select time',
  useUtcValues: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: TimeValue]
}>()

function utcToLocal(utcTime: TimeValue): TimeValue {
  const today = new Date()
  const utcHour = utcTime.hour === '' ? 0 : parseInt(utcTime.hour)
  const utcMinute = utcTime.minute === '' ? 0 : parseInt(utcTime.minute)

  const utcDate = new Date(
    Date.UTC(today.getUTCFullYear(), today.getUTCMonth(), today.getUTCDate(), utcHour, utcMinute),
  )

  return {
    hour: utcDate.getHours().toString(),
    minute: utcDate.getMinutes().toString(),
  }
}

function localToUtc(localTime: TimeValue): TimeValue {
  const today = new Date()
  const localHour = localTime.hour === '' ? 0 : parseInt(localTime.hour)
  const localMinute = localTime.minute === '' ? 0 : parseInt(localTime.minute)

  const localDate = new Date(
    today.getFullYear(),
    today.getMonth(),
    today.getDate(),
    localHour,
    localMinute,
  )

  return {
    hour: localDate.getUTCHours().toString(),
    minute: localDate.getUTCMinutes().toString(),
  }
}

function emitTime(localTime: TimeValue) {
  const timeToEmit = props.useUtcValues ? localToUtc(localTime) : localTime
  emit('update:modelValue', timeToEmit)
}

const dropdown = ref()
const originalTime = ref<TimeValue>({ hour: '12', minute: '00' })

const getInitialTime = (): TimeValue => {
  const defaultTime = { hour: '12', minute: '00' }
  if (!props.modelValue) return defaultTime

  return props.useUtcValues ? utcToLocal(props.modelValue) : props.modelValue
}

const currentTime = reactive<TimeValue>(getInitialTime())

watch(
  () => props.modelValue,
  (newValue) => {
    if (newValue) {
      const displayTime = props.useUtcValues ? utcToLocal(newValue) : newValue
      currentTime.hour = displayTime.hour
      currentTime.minute = displayTime.minute
    }
  },
  { deep: true },
)

function formatTime(time: TimeValue): string {
  const hour = (time.hour === '' ? '0' : time.hour).padStart(2, '0')
  const minute = (time.minute === '' ? '0' : time.minute).padStart(2, '0')
  return `${hour}:${minute}`
}

function getUTCTime(time: TimeValue): string {
  const today = new Date()
  const hourValue = time.hour === '' ? 0 : parseInt(time.hour)
  const minuteValue = time.minute === '' ? 0 : parseInt(time.minute)

  const localDate = new Date(
    today.getFullYear(),
    today.getMonth(),
    today.getDate(),
    hourValue,
    minuteValue,
  )

  const utcHour = localDate.getUTCHours().toString().padStart(2, '0')
  const utcMinute = localDate.getUTCMinutes().toString().padStart(2, '0')

  return `${utcHour}:${utcMinute} UTC`
}

function handleHourChange(event: Event) {
  const target = event.target as HTMLInputElement
  const cleanValue = target.value.replace(/\D/g, '').slice(0, 2)
  let hour = cleanValue

  if (cleanValue !== '' && (parseInt(cleanValue) > 23 || parseInt(cleanValue) < 0)) {
    hour = Math.min(23, Math.max(0, parseInt(cleanValue))).toString()
  }

  currentTime.hour = hour
  emitTime(currentTime)
}

function handleMinuteChange(event: Event) {
  const target = event.target as HTMLInputElement
  const cleanValue = target.value.replace(/\D/g, '').slice(0, 2)
  let minute = cleanValue

  if (cleanValue !== '' && (parseInt(cleanValue) > 59 || parseInt(cleanValue) < 0)) {
    minute = Math.min(59, Math.max(0, parseInt(cleanValue))).toString()
  }

  currentTime.minute = minute
  emitTime(currentTime)
}

function handleQuickSelect(hour: string, minute: string) {
  currentTime.hour = hour
  currentTime.minute = minute
  emitTime(currentTime)
}

function handleCancel() {
  currentTime.hour = originalTime.value.hour
  currentTime.minute = originalTime.value.minute
  emitTime(originalTime.value)
  dropdown.value?.hide()
}

function handleDone() {
  dropdown.value?.hide()
}

function onDropdownShow() {
  originalTime.value = { ...currentTime }
}

const quickSelects = [
  { label: '00:00', hour: '0', minute: '0' },
  { label: '04:00', hour: '4', minute: '0' },
  { label: '12:00', hour: '12', minute: '0' },
  { label: '21:00', hour: '21', minute: '0' },
]
</script>

<style lang="scss">
.v-popper--theme-dropdown .v-popper__arrow-container {
  display: none;
}
</style>
