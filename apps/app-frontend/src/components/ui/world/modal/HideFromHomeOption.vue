<script setup lang="ts">
import { Checkbox } from '@modrinth/ui'
import { computed } from 'vue'

import { defineMessage, useVIntl } from '@/utils/i18n-vintl'

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
