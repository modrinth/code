<template>
	<div class="w-full flex flex-col gap-4" :class="{ 'mt-4': isNuxt }">
		<PageHeader :title="props.name || props.fallbackName" :header-class="props.headerClass">
			<template #leading>
				<div class="flex size-16 shrink-0 items-center justify-center">
					<ButtonStyled circular size="large">
						<button
							v-tooltip="props.backLabel"
							type="button"
							:aria-label="props.backLabel"
							@click="router.push(props.backHref)"
						>
							<LeftArrowIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</template>

			<template #metadata>
				<PageHeaderMetadata>
					<PageHeaderMetadataItem
						v-for="item in headerMetadata"
						:key="item.id"
						:icon="item.icon"
						:icon-props="item.iconProps"
					>
						{{ item.label }}
					</PageHeaderMetadataItem>
				</PageHeaderMetadata>
			</template>

			<template #actions>
				<PageHeaderActions>
					<template v-for="action in props.actions" :key="action.id">
						<JoinedButtons
							v-if="action.joinedActions?.length"
							:actions="action.joinedActions"
							:color="action.color ?? 'standard'"
							:size="action.size ?? 'large'"
							:disabled="action.disabled"
							:primary-disabled="action.primaryDisabled"
							:dropdown-disabled="action.dropdownDisabled"
							:primary-muted="action.primaryMuted"
						/>
						<ButtonStyled
							v-else
							:color="action.color ?? 'standard'"
							:size="action.size ?? 'large'"
							:type="action.type ?? 'standard'"
							:circular="action.circular ?? action.labelHidden ?? false"
						>
							<button
								v-tooltip="action.tooltip"
								type="button"
								:disabled="action.disabled"
								:aria-label="action.ariaLabel ?? action.tooltip ?? action.label"
								@click="action.onClick"
							>
								<component
									:is="action.icon"
									v-if="action.icon"
									:class="action.iconClass"
									aria-hidden="true"
									v-bind="action.iconProps"
								/>
								<span v-if="!action.labelHidden && !action.circular">{{ action.label }}</span>
							</button>
						</ButtonStyled>
					</template>
				</PageHeaderActions>
			</template>
		</PageHeader>
	</div>
</template>

<script setup lang="ts">
import { NuxtModrinthClient } from '@modrinth/api-client'
import { LeftArrowIcon, TagCategoryGamepad2Icon as Gamepad2Icon, TimerIcon } from '@modrinth/assets'
import { type Component, computed } from 'vue'
import { useRouter } from 'vue-router'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import JoinedButtons, { type JoinedButtonAction } from '#ui/components/base/JoinedButtons.vue'
import PageHeader from '#ui/components/base/page-header/index.vue'
import PageHeaderMetadata from '#ui/components/base/page-header/metadata/index.vue'
import PageHeaderMetadataItem from '#ui/components/base/page-header/metadata/page-header-metadata-item.vue'
import PageHeaderActions from '#ui/components/base/page-header/page-header-actions.vue'
import LoaderIcon from '#ui/components/servers/icons/LoaderIcon.vue'
import { injectModrinthClient } from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

type MetadataItem = {
	id: string
	label: string
	icon: Component
	iconProps?: Record<string, unknown>
}
type HeaderAction = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: string
	tooltip?: string
	ariaLabel?: string
	onClick?: () => void | Promise<void>
	disabled?: boolean
	labelHidden?: boolean
	circular?: boolean
	color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
	size?: 'standard' | 'large' | 'small'
	type?: 'standard' | 'outlined' | 'transparent' | 'highlight' | 'highlight-colored-text' | 'chip'
	joinedActions?: JoinedButtonAction[]
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
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
		actions?: HeaderAction[]
	}>(),
	{
		name: null,
		metadataItems: () => [],
		gameVersion: null,
		loader: null,
		loaderVersion: null,
		lastActive: null,
		fallbackName: 'Instance',
		headerClass: '',
		actions: () => [],
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
