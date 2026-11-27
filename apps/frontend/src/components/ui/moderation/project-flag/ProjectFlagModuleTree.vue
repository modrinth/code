<template>
	<ul class="m-0 list-none space-y-1 pl-3">
		<li v-for="node in nodes" :key="node.path" class="min-w-0">
			<button
				v-if="names.includes(node.path)"
				type="button"
				class="w-full rounded-lg px-2 py-1 text-left font-mono text-sm hover:bg-bg-raised"
				:class="{ 'bg-bg-raised text-contrast': selected === node.path }"
				:aria-current="selected === node.path ? 'true' : undefined"
				:disabled="disabled"
				:title="node.path"
				@click="$emit('select', node.path)"
			>
				<span class="break-all">{{ node.label }}</span>
			</button>
			<details v-if="names.some((name) => name.startsWith(`${node.path}/`))" open>
				<summary class="cursor-pointer break-all py-1 font-mono text-sm">{{ node.label }}/</summary>
				<ProjectFlagModuleTree
					:names="names"
					:prefix="`${node.path}/`"
					:selected="selected"
					:disabled="disabled"
					@select="$emit('select', $event)"
				/>
			</details>
		</li>
	</ul>
</template>

<script setup lang="ts">
const props = withDefaults(
	defineProps<{
		names: string[]
		prefix?: string
		selected: string | null
		disabled?: boolean
	}>(),
	{ prefix: '' },
)
defineEmits<{ select: [name: string] }>()
const nodes = computed(() =>
	[
		...new Set(
			props.names
				.filter((name) => name.startsWith(props.prefix))
				.map((name) => name.slice(props.prefix.length).split('/')[0]),
		),
	]
		.sort()
		.map((label) => ({ label, path: props.prefix + label })),
)
</script>
