<template>
	<div
		class="flex flex-col gap-2"
		:class="[
			props.divider ? 'border-0 border-b border-solid border-divider' : '',
			props.bottomPadding ? 'pb-4' : '',
			props.headerClass,
		]"
	>
		<div class="flex flex-wrap items-start gap-4 max-md:flex-col" :class="props.rowClass">
			<div class="flex min-w-0 flex-1 gap-4" :class="props.mainClass">
				<div v-if="leadingItems.length" class="contents">
					<template v-for="(leadingItem, index) in leadingItems" :key="leadingItem.id ?? index">
						<div
							v-if="leadingItem.type === 'button'"
							:class="
								leadingWrapperClass(
									leadingItem,
									'flex size-16 shrink-0 items-center justify-center',
								)
							"
						>
							<ButtonStyled
								circular
								:color="leadingItem.color ?? 'standard'"
								:size="leadingItem.size ?? 'large'"
								:type="leadingItem.buttonType ?? 'standard'"
							>
								<AutoLink
									v-if="leadingItem.to"
									v-tooltip="leadingItem.tooltip"
									:to="leadingItem.to"
									:aria-label="leadingLabel(leadingItem)"
								>
									<component
										:is="leadingItem.icon"
										v-if="leadingItem.icon"
										aria-hidden="true"
										v-bind="leadingItem.iconProps"
									/>
								</AutoLink>
								<button
									v-else
									v-tooltip="leadingItem.tooltip"
									type="button"
									:aria-label="leadingLabel(leadingItem)"
									@click="leadingItem.onClick"
								>
									<component
										:is="leadingItem.icon"
										v-if="leadingItem.icon"
										aria-hidden="true"
										v-bind="leadingItem.iconProps"
									/>
								</button>
							</ButtonStyled>
						</div>
						<Avatar
							v-else-if="leadingItem.type === 'avatar'"
							:src="leadingItem.src"
							:alt="leadingItem.alt ?? ''"
							:size="leadingItem.avatarSize ?? '64px'"
							:tint-by="leadingItem.tintBy ?? null"
							:circle="leadingItem.circle ?? false"
							:no-shadow="leadingItem.noShadow ?? false"
							:raised="leadingItem.raised ?? false"
							:loading="leadingItem.loading ?? 'eager'"
							:class="leadingItem.class"
						/>
						<component
							:is="leadingItem.component"
							v-else-if="leadingItem.component"
							:class="leadingItem.class"
							v-bind="leadingItem.componentProps"
						/>
					</template>
				</div>

				<div class="flex min-w-0 flex-col gap-2 justify-center">
					<div class="flex flex-col gap-1.5 justify-center">
						<div class="flex flex-wrap items-center gap-2">
							<h1
								class="m-0 min-w-0 max-w-full text-2xl font-semibold leading-none text-contrast"
								:class="[props.truncateTitle ? 'truncate' : '', props.titleClass]"
							>
								{{ props.header }}
							</h1>
						</div>
						<p
							v-if="props.summary"
							class="m-0 max-w-[44rem] empty:hidden"
							:class="[props.disableLineClamp ? '' : 'line-clamp-2']"
						>
							{{ props.summary }}
						</p>
					</div>

					<div v-if="props.metadata.length" class="flex flex-wrap gap-3 empty:hidden max-md:hidden">
						<div class="flex min-w-0 flex-wrap items-center gap-2">
							<template v-for="(item, index) in props.metadata" :key="item.id">
								<BulletDivider v-if="index > 0" class="shrink-0" />
								<AutoLink
									v-if="item.to"
									v-tooltip="item.tooltip"
									:to="item.to"
									:aria-label="item.ariaLabel ?? item.label"
									:class="metadataClass(item, true)"
								>
									<component
										:is="item.icon"
										v-if="item.icon"
										class="flex size-5 shrink-0"
										aria-hidden="true"
										v-bind="item.iconProps"
									/>
									<span class="truncate">{{ item.label }}</span>
								</AutoLink>
								<button
									v-else-if="item.onClick"
									v-tooltip="item.tooltip"
									type="button"
									:aria-label="item.ariaLabel ?? item.label"
									:class="metadataClass(item, true)"
									@click="item.onClick"
								>
									<component
										:is="item.icon"
										v-if="item.icon"
										class="flex size-5 shrink-0"
										aria-hidden="true"
										v-bind="item.iconProps"
									/>
									<span class="truncate">{{ item.label }}</span>
								</button>
								<div
									v-else
									v-tooltip="item.tooltip"
									:aria-label="item.ariaLabel"
									:class="metadataClass(item)"
								>
									<component
										:is="item.icon"
										v-if="item.icon"
										class="flex size-5 shrink-0"
										aria-hidden="true"
										v-bind="item.iconProps"
									/>
									<span class="truncate">{{ item.label }}</span>
								</div>
							</template>
						</div>
					</div>
				</div>
			</div>

			<div v-if="props.actions.length" class="flex flex-wrap gap-2 items-center">
				<ButtonStyled
					v-for="action in props.actions"
					:key="action.id"
					:color="action.color ?? 'standard'"
					:size="action.size ?? 'large'"
					:type="action.type ?? 'standard'"
					:circular="action.circular ?? action.labelHidden ?? false"
				>
					<AutoLink
						v-if="action.to"
						v-tooltip="action.tooltip"
						:to="action.to"
						:aria-label="actionLabel(action)"
					>
						<component
							:is="action.icon"
							v-if="action.icon"
							aria-hidden="true"
							v-bind="action.iconProps"
						/>
						<span v-if="!action.labelHidden && !action.circular">{{ action.label }}</span>
					</AutoLink>
					<button
						v-else
						v-tooltip="action.tooltip"
						type="button"
						:disabled="action.disabled"
						:aria-label="actionLabel(action)"
						@click="action.onClick"
					>
						<component
							:is="action.icon"
							v-if="action.icon"
							aria-hidden="true"
							v-bind="action.iconProps"
						/>
						<span v-if="!action.labelHidden && !action.circular">{{ action.label }}</span>
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="props.metadata.length" class="flex justify-between md:hidden">
			<div class="flex flex-wrap gap-3 empty:hidden">
				<div class="flex min-w-0 flex-wrap items-center gap-2">
					<template v-for="(item, index) in props.metadata" :key="item.id">
						<BulletDivider v-if="index > 0" class="shrink-0" />
						<AutoLink
							v-if="item.to"
							v-tooltip="item.tooltip"
							:to="item.to"
							:aria-label="item.ariaLabel ?? item.label"
							:class="metadataClass(item, true)"
						>
							<component
								:is="item.icon"
								v-if="item.icon"
								class="flex size-5 shrink-0"
								aria-hidden="true"
								v-bind="item.iconProps"
							/>
							<span class="truncate">{{ item.label }}</span>
						</AutoLink>
						<button
							v-else-if="item.onClick"
							v-tooltip="item.tooltip"
							type="button"
							:aria-label="item.ariaLabel ?? item.label"
							:class="metadataClass(item, true)"
							@click="item.onClick"
						>
							<component
								:is="item.icon"
								v-if="item.icon"
								class="flex size-5 shrink-0"
								aria-hidden="true"
								v-bind="item.iconProps"
							/>
							<span class="truncate">{{ item.label }}</span>
						</button>
						<div
							v-else
							v-tooltip="item.tooltip"
							:aria-label="item.ariaLabel"
							:class="metadataClass(item)"
						>
							<component
								:is="item.icon"
								v-if="item.icon"
								class="flex size-5 shrink-0"
								aria-hidden="true"
								v-bind="item.iconProps"
							/>
							<span class="truncate">{{ item.label }}</span>
						</div>
					</template>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from './AutoLink.vue'
import Avatar from './Avatar.vue'
import BulletDivider from './BulletDivider.vue'
import ButtonStyled from './ButtonStyled.vue'

type PageHeaderTarget = string | RouteLocationRaw
type ButtonColor =
	| 'standard'
	| 'brand'
	| 'red'
	| 'orange'
	| 'green'
	| 'blue'
	| 'purple'
	| 'medal-promo'
type ButtonSize = 'standard' | 'large' | 'small'
type ButtonType =
	| 'standard'
	| 'outlined'
	| 'transparent'
	| 'highlight'
	| 'highlight-colored-text'
	| 'chip'

type PageHeaderLeading = {
	id?: string
	type: 'avatar' | 'button' | 'component'
	icon?: Component
	iconProps?: Record<string, unknown>
	component?: Component
	componentProps?: Record<string, unknown>
	src?: string | null
	alt?: string
	tintBy?: string | null
	avatarSize?: string
	circle?: boolean
	noShadow?: boolean
	raised?: boolean
	loading?: 'eager' | 'lazy'
	class?: string
	wrapperClass?: string
	to?: PageHeaderTarget
	onClick?: () => void | Promise<void>
	tooltip?: string
	ariaLabel?: string
	color?: ButtonColor
	size?: ButtonSize
	buttonType?: ButtonType
}

type PageHeaderMetadataItem = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	onClick?: () => void | Promise<void>
	class?: string
}

type PageHeaderAction = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	onClick?: () => void | Promise<void>
	disabled?: boolean
	labelHidden?: boolean
	circular?: boolean
	color?: ButtonColor
	size?: ButtonSize
	type?: ButtonType
}

const props = withDefaults(
	defineProps<{
		header: string
		summary?: string | null
		leading?: PageHeaderLeading | PageHeaderLeading[] | null
		metadata?: PageHeaderMetadataItem[]
		actions?: PageHeaderAction[]
		headerClass?: string
		rowClass?: string
		mainClass?: string
		titleClass?: string
		truncateTitle?: boolean
		divider?: boolean
		bottomPadding?: boolean
		disableLineClamp?: boolean
	}>(),
	{
		summary: null,
		leading: null,
		metadata: () => [],
		actions: () => [],
		headerClass: '',
		rowClass: '',
		mainClass: '',
		titleClass: '',
		truncateTitle: false,
		divider: true,
		bottomPadding: true,
		disableLineClamp: false,
	},
)

const leadingItems = computed(() => {
	if (!props.leading) return []
	return Array.isArray(props.leading) ? props.leading : [props.leading]
})

function leadingLabel(item: PageHeaderLeading) {
	return item.ariaLabel ?? item.tooltip ?? 'Header action'
}

function leadingWrapperClass(item: PageHeaderLeading, defaultClass: string) {
	return [item.wrapperClass ?? defaultClass]
}

function metadataClass(item: PageHeaderMetadataItem, interactive = false) {
	return [
		'flex min-w-0 items-center gap-2 font-medium text-secondary text-nowrap',
		interactive ? 'm-0 cursor-pointer border-0 bg-transparent p-0 hover:underline' : '',
		item.class,
	]
}

function actionLabel(action: PageHeaderAction) {
	return action.ariaLabel ?? action.tooltip ?? action.label
}
</script>
