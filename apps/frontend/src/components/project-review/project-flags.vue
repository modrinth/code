<script setup lang="ts">
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

const props = defineProps<{ projectId: string }>()
const api = injectModrinthClient().labrinth.project_flags_internal
const { data: flags, isError } = useQuery({
	queryKey: computed(() => ['moderation', 'project-flags', props.projectId]),
	queryFn: () => api.getProjectFlags(props.projectId),
	enabled: computed(() => !!props.projectId),
	refetchInterval: 10_000,
})
</script>

<template>
	<section v-if="flags?.length || isError" class="flex flex-col gap-2 rounded-xl bg-surface-3 p-3">
		<h3 class="m-0 text-sm font-semibold">Project flag violations ({{ flags?.length ?? 0 }})</h3>
		<p v-if="isError" class="m-0 text-sm text-red">Could not load project flag violations.</p>
		<article v-for="flag in flags" :key="flag.key" class="flex flex-col gap-1 text-sm">
			<div class="flex flex-wrap gap-2 font-semibold">
				<span>{{ flag.key }}</span>
				<span>{{ flag.active ? 'Active' : 'Resolved · Retained' }}</span>
			</div>
			<p v-if="!flag.active" class="m-0 text-secondary">
				This flag is no longer current but is retained by the rule. It remains a violation and
				blocks automatic approval.
			</p>
			<p v-if="flag.reason" class="m-0 whitespace-pre-wrap">{{ flag.reason }}</p>
			<details>
				<summary class="cursor-pointer">Flag evidence</summary>
				<pre class="m-0 overflow-auto whitespace-pre-wrap">{{ JSON.stringify(flag, null, 2) }}</pre>
			</details>
		</article>
	</section>
</template>
