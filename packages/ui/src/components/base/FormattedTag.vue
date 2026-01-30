<script setup lang="ts">
import { computed } from 'vue'

import { useVIntl } from '../../composables'
import { formatTag } from '../../utils/tag-messages.ts'

const { formatMessage } = useVIntl()

const props = defineProps<{
	tag: string
	enforceType?: 'loader' | 'category'
}>()

const message = computed(() => formatTag(formatMessage, props.tag, props.enforceType))
</script>

<template>
	{{ message }}
</template>
