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
							<template v-for="badge in props.badges" :key="badge.id">
								<component
									:is="badge.component"
									v-if="badge.component"
									:class="badge.class"
									v-bind="badge.componentProps"
								/>
								<AutoLink
									v-else-if="badge.to"
									v-tooltip="badge.tooltip"
									:to="badge.to"
									:aria-label="badge.ariaLabel ?? badge.label"
									:class="badgeClass(badge, true)"
									:style="badge.style"
								>
									<component
										:is="badge.icon"
										v-if="badge.icon"
										aria-hidden="true"
										v-bind="badge.iconProps"
									/>
									<span>{{ badge.label }}</span>
								</AutoLink>
								<button
									v-else-if="badge.onClick"
									v-tooltip="badge.tooltip"
									type="button"
									:aria-label="badge.ariaLabel ?? badge.label"
									:class="badgeClass(badge, true)"
									:style="badge.style"
									@click="badge.onClick"
								>
									<component
										:is="badge.icon"
										v-if="badge.icon"
										aria-hidden="true"
										v-bind="badge.iconProps"
									/>
									<span>{{ badge.label }}</span>
								</button>
								<div
									v-else
									v-tooltip="badge.tooltip"
									:aria-label="badge.ariaLabel"
									:class="badgeClass(badge)"
									:style="badge.style"
								>
									<component
										:is="badge.icon"
										v-if="badge.icon"
										aria-hidden="true"
										v-bind="badge.iconProps"
									/>
									<span>{{ badge.label }}</span>
								</div>
							</template>
						</div>
						<p
							v-if="props.summary || $slots.summary"
							class="m-0 max-w-[44rem] empty:hidden"
							:class="[props.disableLineClamp ? '' : 'line-clamp-2']"
						>
							<slot name="summary">{{ props.summary }}</slot>
						</p>
					</div>

					<div v-if="props.metadata.length" class="flex flex-wrap gap-3 empty:hidden max-md:hidden">
						<div class="flex min-w-0 flex-wrap items-center gap-2">
							<template v-for="(item, index) in props.metadata" :key="item.id">
								<BulletDivider v-if="index > 0" class="shrink-0" />
								<div v-if="item.type === 'custom'" :class="item.class">
									<slot :name="metadataSlotName(item)" :item="item" />
								</div>
								<AutoLink
									v-else-if="item.to"
									v-tooltip="item.tooltip"
									:to="item.to"
									:aria-label="item.ariaLabel ?? item.label ?? ''"
									:class="metadataClass(item, true)"
								>
									<component
										:is="item.icon"
										v-if="item.icon"
										class="flex size-5 shrink-0 text-current"
										aria-hidden="true"
										v-bind="item.iconProps"
									/>
									<span class="truncate">{{ item.label }}</span>
								</AutoLink>
								<button
									v-else-if="item.onClick"
									v-tooltip="item.tooltip"
									type="button"
									:aria-label="item.ariaLabel ?? item.label ?? ''"
									:class="metadataClass(item, true)"
									@click="item.onClick"
								>
									<component
										:is="item.icon"
										v-if="item.icon"
										class="flex size-5 shrink-0 text-current"
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
										class="flex size-5 shrink-0 text-current"
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
				<template v-for="action in props.actions" :key="action.id">
					<component
						:is="action.component"
						v-if="action.component"
						:class="action.class"
						v-bind="action.componentProps"
					/>
					<Tooltip
						v-else-if="action.prompt"
						theme="dismissable-prompt"
						:triggers="[]"
						:shown="action.prompt.shown"
						:auto-hide="false"
						:placement="action.prompt.placement ?? 'bottom'"
					>
						<JoinedButtons
							v-if="action.joinedActions?.length"
							:actions="action.joinedActions"
							:color="joinedActionColor(action.color)"
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
							<TeleportOverflowMenu
								v-if="action.menuActions?.length"
								:options="action.menuActions"
								:tooltip="action.tooltip"
								:aria-label="actionLabel(action)"
								:disabled="action.disabled"
								@open="dismissActionPrompt(action)"
							>
								<component
									:is="action.icon"
									v-if="action.icon"
									:class="action.iconClass"
									aria-hidden="true"
									v-bind="action.iconProps"
								/>
								<span v-if="!action.labelHidden && !action.circular">{{ action.label }}</span>
							</TeleportOverflowMenu>
							<AutoLink
								v-else-if="action.to"
								v-tooltip="action.tooltip"
								:to="action.to"
								:aria-label="actionLabel(action)"
								@click="(event) => handleActionClick(action, event)"
							>
								<component
									:is="action.icon"
									v-if="action.icon"
									:class="action.iconClass"
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
								@click="(event) => handleActionClick(action, event)"
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
						<template #popper>
							<div class="grid grid-cols-[min-content] gap-1">
								<div class="flex min-w-48 items-center justify-between gap-8">
									<h3
										class="m-0 flex items-center gap-2 whitespace-nowrap text-base font-bold text-contrast"
									>
										{{ action.prompt.title }}
										<span
											v-if="action.prompt.badge"
											class="inline-flex items-center rounded-full border border-solid border-brand-highlight bg-brand-highlight px-2 py-1 text-sm font-semibold leading-none text-brand"
										>
											{{ action.prompt.badge }}
										</span>
									</h3>
									<ButtonStyled size="small" circular>
										<button
											v-tooltip="action.prompt.dismissLabel"
											@click="action.prompt.onDismiss?.()"
										>
											<XIcon aria-hidden="true" />
										</button>
									</ButtonStyled>
								</div>
								<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
									{{ action.prompt.description }}
								</p>
								<p v-if="action.prompt.footer" class="m-0 text-wrap text-sm font-bold text-primary">
									{{ action.prompt.footer }}
								</p>
							</div>
						</template>
					</Tooltip>
					<template v-else>
						<JoinedButtons
							v-if="action.joinedActions?.length"
							:actions="action.joinedActions"
							:color="joinedActionColor(action.color)"
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
							<TeleportOverflowMenu
								v-if="action.menuActions?.length"
								:options="action.menuActions"
								:tooltip="action.tooltip"
								:aria-label="actionLabel(action)"
								:disabled="action.disabled"
							>
								<component
									:is="action.icon"
									v-if="action.icon"
									:class="action.iconClass"
									aria-hidden="true"
									v-bind="action.iconProps"
								/>
								<span v-if="!action.labelHidden && !action.circular">{{ action.label }}</span>
							</TeleportOverflowMenu>
							<AutoLink
								v-else-if="action.to"
								v-tooltip="action.tooltip"
								:to="action.to"
								:aria-label="actionLabel(action)"
								@click="(event) => handleActionClick(action, event)"
							>
								<component
									:is="action.icon"
									v-if="action.icon"
									:class="action.iconClass"
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
								@click="(event) => handleActionClick(action, event)"
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
				</template>
			</div>
		</div>

		<div v-if="props.metadata.length" class="flex justify-between md:hidden">
			<div class="flex flex-wrap gap-3 empty:hidden">
				<div class="flex min-w-0 flex-wrap items-center gap-2">
					<template v-for="(item, index) in props.metadata" :key="item.id">
						<BulletDivider v-if="index > 0" class="shrink-0" />
						<div v-if="item.type === 'custom'" :class="item.class">
							<slot :name="metadataSlotName(item)" :item="item" />
						</div>
						<AutoLink
							v-else-if="item.to"
							v-tooltip="item.tooltip"
							:to="item.to"
							:aria-label="item.ariaLabel ?? item.label ?? ''"
							:class="metadataClass(item, true)"
						>
							<component
								:is="item.icon"
								v-if="item.icon"
								class="flex size-5 shrink-0 text-current"
								aria-hidden="true"
								v-bind="item.iconProps"
							/>
							<span class="truncate">{{ item.label }}</span>
						</AutoLink>
						<button
							v-else-if="item.onClick"
							v-tooltip="item.tooltip"
							type="button"
							:aria-label="item.ariaLabel ?? item.label ?? ''"
							:class="metadataClass(item, true)"
							@click="item.onClick"
						>
							<component
								:is="item.icon"
								v-if="item.icon"
								class="flex size-5 shrink-0 text-current"
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
								class="flex size-5 shrink-0 text-current"
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
import { XIcon } from '@modrinth/assets'
import { Tooltip } from 'floating-vue'
import type { Component } from 'vue'
import { computed } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from './AutoLink.vue'
import Avatar from './Avatar.vue'
import BulletDivider from './BulletDivider.vue'
import ButtonStyled from './ButtonStyled.vue'
import JoinedButtons, { type JoinedButtonAction } from './JoinedButtons.vue'
import TeleportOverflowMenu, {
	type Item as TeleportOverflowMenuItem,
} from './TeleportOverflowMenu.vue'

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
	onClick?: (event?: MouseEvent) => void | Promise<void>
	tooltip?: string
	ariaLabel?: string
	color?: ButtonColor
	size?: ButtonSize
	buttonType?: ButtonType
}

type PageHeaderBadge = {
	id: string
	label?: string
	icon?: Component
	iconProps?: Record<string, unknown>
	component?: Component
	componentProps?: Record<string, unknown>
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	onClick?: () => void | Promise<void>
	class?: string
	style?: Record<string, string>
}

type PageHeaderMetadataItem = {
	id: string
	type?: 'text' | 'custom'
	label?: string
	icon?: Component
	iconProps?: Record<string, unknown>
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	onClick?: () => void | Promise<void>
	class?: string
}

type PageHeaderActionPrompt = {
	title: string
	description: string
	badge?: string
	footer?: string
	dismissLabel?: string
	shown?: boolean
	placement?: string
	onDismiss?: () => void
}

type PageHeaderAction = {
	id: string
	label: string
	component?: Component
	componentProps?: Record<string, unknown>
	class?: string
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: string
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
	joinedActions?: JoinedButtonAction[]
	menuActions?: TeleportOverflowMenuItem[]
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
	prompt?: PageHeaderActionPrompt
}

const props = withDefaults(
	defineProps<{
		header: string
		summary?: string | null
		leading?: PageHeaderLeading | PageHeaderLeading[] | null
		badges?: PageHeaderBadge[]
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
		badges: () => [],
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

function metadataSlotName(item: PageHeaderMetadataItem) {
	return `metadata-${item.id}`
}

function badgeClass(badge: PageHeaderBadge, interactive = false) {
	return [
		'inline-flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-button-bg px-2 py-1 text-sm font-semibold leading-none text-secondary text-nowrap [&>svg]:size-4 [&>svg]:shrink-0',
		interactive ? 'm-0 cursor-pointer hover:underline' : '',
		badge.class,
	]
}

function actionLabel(action: PageHeaderAction) {
	return action.ariaLabel ?? action.tooltip ?? action.label
}

function dismissActionPrompt(action: PageHeaderAction) {
	if (action.prompt?.shown) {
		action.prompt.onDismiss?.()
	}
}

function handleActionClick(action: PageHeaderAction, event?: MouseEvent) {
	dismissActionPrompt(action)
	void action.onClick?.(event)
}

function joinedActionColor(color?: ButtonColor) {
	return color === 'medal-promo' ? 'standard' : color
}
</script>
