<template>
	<section class="flex min-w-0 flex-col gap-3" aria-label="Input schema">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">Input schema</h3>
			<ProjectCombobox
				v-model="projectId"
				class="ml-auto max-w-60"
				dropdown-width="26rem"
				search-placeholder="Search project name/ID..."
				:search-input-attrs="{ 'aria-label': 'Project for input schema' }"
				lookup-project-ids
				:show-chevron="false"
				clearable
			>
				<template #option-suffix="{ item }">
					<TagItem class="shrink-0 font-mono">{{ item.value }}</TagItem>
				</template>
			</ProjectCombobox>
		</div>
		<div
			class="min-h-0 flex-1 overflow-auto rounded-xl border border-solid border-surface-4 bg-surface-2"
		>
			<div
				v-if="isPending"
				role="status"
				class="flex items-center gap-2 p-4 text-sm text-secondary"
			>
				<LoaderCircleIcon class="size-4 animate-spin" />
				Loading schema…
			</div>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2 p-4">
				<p class="m-0 text-sm text-red">Could not load the input schema: {{ error?.message }}</p>
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<div
				v-else-if="data"
				class="p-4 font-mono text-xs leading-relaxed text-primary [overflow-wrap:anywhere]"
			>
				<code>{{ '{' }}</code>
				<div class="pl-3">
					<SchemaField
						v-for="([name, value], index) in fields"
						:key="name"
						:name="name"
						:value="value"
						:trailing-comma="index < fields.length - 1"
					/>
				</div>
				<code>{{ '}' }}</code>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import type { ProjectFlagValue } from '@modrinth/api-client'
import { LoaderCircleIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, ProjectCombobox, TagItem } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import SchemaField from './field.vue'

const projectId = defineModel<string | undefined>('projectId', { required: true })
const client = injectModrinthClient()
const { data, isPending, isError, error, isFetching, refetch } = useQuery({
	queryKey: computed(
		() => ['moderation', 'project-flag', 'input-schema', projectId.value ?? ''] as const,
	),
	queryFn: async ({ queryKey: [, , , projectId] }) => {
		const example = client.labrinth.project_flags_internal.getRuleSchema()
		if (!projectId) return example

		// TODO: Use a dedicated backend route to preview the full flag rule input for a project.
		const [schema, project] = await Promise.all([
			example,
			client.labrinth.projects_v3.get(projectId),
		])
		return { ...schema, project: project as Record<string, ProjectFlagValue> }
	},
	staleTime: 60_000,
})
const fields = computed(() => Object.entries(data.value ?? {}))
</script>
