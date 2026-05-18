<template>
	<div class="w-full flex flex-col gap-4" :class="{ 'mt-4': isNuxt }">
		<ContentPageHeader :class="props.headerClass">
			<template #icon>
				<div class="flex size-16 shrink-0 items-center justify-center">
					<ButtonStyled circular size="large">
						<button
							type="button"
							:aria-label="props.backLabel"
							@click="router.push(props.backHref)"
						>
							<LeftArrowIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</template>
			<template #title>
				{{ props.name || props.fallbackName }}
			</template>
			<template #stats>
				<div v-if="headerMetadata.length" class="flex min-w-0 flex-wrap items-center gap-2">
					<template v-for="(item, index) in headerMetadata" :key="item.id">
						<BulletDivider v-if="index > 0" />
						<div class="flex min-w-0 items-center gap-2 font-medium text-secondary text-nowrap">
							<component
								:is="item.icon"
								class="flex size-5 shrink-0"
								aria-hidden="true"
								v-bind="item.iconProps"
							/>
							<span class="truncate">{{ item.label }}</span>
						</div>
					</template>
				</div>
			</template>
			<template #actions>
				<slot name="actions" />
			</template>
		</ContentPageHeader>
	</div>
</template>

<script setup lang="ts">
import { NuxtModrinthClient } from '@modrinth/api-client'
import { LeftArrowIcon, TagCategoryGamepad2Icon as Gamepad2Icon, TimerIcon } from '@modrinth/assets'
import { type Component, computed } from 'vue'
import { useRouter } from 'vue-router'

import BulletDivider from '#ui/components/base/BulletDivider.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import ContentPageHeader from '#ui/components/base/ContentPageHeader.vue'
import LoaderIcon from '#ui/components/servers/icons/LoaderIcon.vue'
import { injectModrinthClient } from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

type MetadataItem = {
	id: string
	label: string
	icon: Component
	iconProps?: Record<string, unknown>
}

const props = withDefaults(
	defineProps<{
		name?: string | null
		metadataItems?: MetadataItem[]
		gameVersion?: string | null
		loader?: string | null
		loaderVersion?: string | null
		lastActive?: string | null
		backHref: string
		backLabel: string
		fallbackName?: string
		headerClass?: string
	}>(),
	{
		name: null,
		metadataItems: () => [],
		gameVersion: null,
		loader: null,
		loaderVersion: null,
		lastActive: null,
		fallbackName: 'World',
		headerClass: '',
	},
)

const client = injectModrinthClient()
const router = useRouter()
const isNuxt = computed(() => client instanceof NuxtModrinthClient)
const loaderLabel = computed(() => {
	if (!props.loader) return null
	const label = formatLoaderLabel(props.loader.toLowerCase())
	return [label, props.loaderVersion].filter(Boolean).join(' ')
})
const headerMetadata = computed<MetadataItem[]>(() => {
	if (props.metadataItems.length) return props.metadataItems

	const items: MetadataItem[] = []
	if (props.gameVersion) {
		items.push({
			id: 'game-version',
			label: props.gameVersion,
			icon: Gamepad2Icon,
		})
	}
	if (props.loader && loaderLabel.value) {
		items.push({
			id: 'loader',
			label: loaderLabel.value,
			icon: LoaderIcon,
			iconProps: {
				loader: formatLoaderLabel(props.loader.toLowerCase()),
			},
		})
	}
	if (props.lastActive) {
		items.push({
			id: 'last-active',
			label: props.lastActive,
			icon: TimerIcon,
		})
	}

	return items
})
</script>
