<script setup lang="ts">
import { defineMessage, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'
import { Checkbox } from '@modrinth/ui'

const { formatMessage } = useVIntl()
const value = defineModel<boolean>({ required: true })

const labelMessage = defineMessage({
  id: 'instance.edit-world.hide-from-home',
  defaultMessage: `Hide from the Home page`,
})

const label = computed(() => formatMessage(labelMessage))
</script>
<template>
  <Checkbox v-model="value" :label="label" />
</template>
