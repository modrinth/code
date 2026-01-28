<script setup lang="ts">
import { computed } from 'vue'

import { useVIntl } from '../../composables'
import { getTagMessageOrDefault } from '../../utils/tag-messages.ts'

const { formatMessage } = useVIntl()

const props = defineProps<{
	tag: string
	enforceType?: 'loader' | 'category'
}>()

const message = computed(() => getTagMessageOrDefault(props.tag, props.enforceType))
</script>

<template>
	{{ typeof message === 'string' ? message : formatMessage(message) }}
</template>
