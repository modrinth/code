<template>
	<PopoutMenu
		ref="dropdown"
		v-bind="$attrs"
		:disabled="disabled"
		:dropdown-id="dropdownId"
		:tooltip="tooltip"
		:placement="placement"
	>
		<slot></slot>
		<template #menu>
			<slot name="menu-header" />
			<div class="p-[2px]">
				<template
					v-for="(option, index) in options.filter((x) => x.shown === undefined || x.shown)"
				>
					<div
						v-if="isDivider(option)"
						:key="`divider-${index}`"
						class="h-px my-1 mx-[-2px] bg-[#979797]"
					></div>
					<AutoLink
						v-else
						:key="`option-${option.id}`"
						v-tooltip="option.tooltip"
						class="flex w-full items-center gap-3 rounded-[2px] px-4 py-1 border !border-solid border-transparent [&>svg]:size-5"
						:class="{
							'hover:bg-[#E9EFF7] hover:border-[#AECFF7]': !option.color,
							'hover:bg-[--color-red-100] hover:border-[--color-red-300]':
								option.color === 'red' || option.color === 'danger',
							'hover:bg-[--color-orange-100] hover:border-[--color-orange-200]':
								option.color === 'orange',
							'hover:bg-[--color-green-100] hover:border-[--color-green-400]':
								option.color === 'green' || option.color === 'primary',
						}"
						:v-close-popper="!option.remainOnClick"
						:to="
							option.action
								? (event: MouseEvent) => {
										option.action?.(event)
										if (!option.remainOnClick) {
											close()
										}
									}
								: option.link
									? option.link
									: undefined
						"
						:external="option.external ? option.external : false"
						:disabled="option.disabled"
						@click="
							() => {
								if (option.link && !option.remainOnClick) {
									close()
								}
							}
						"
					>
						<template v-if="!$slots[option.id]">
							<component :is="option.icon" v-if="option.icon" class="size-5" />
							{{ option.id }}
						</template>
						<slot :name="option.id"></slot>
					</AutoLink>
				</template>
			</div>
		</template>
	</PopoutMenu>
</template>

<script setup lang="ts">
import { type Component, type Ref, ref } from 'vue'

import { AutoLink } from '#ui/components'

import PopoutMenu from './PopoutMenu.vue'

interface BaseOption {
	shown?: boolean
}

interface Divider extends BaseOption {
	divider?: boolean
}

interface Item extends BaseOption {
	id: string
	icon?: Component
	action?: (event?: MouseEvent) => void
	link?: string
	external?: boolean
	color?:
		| 'primary'
		| 'danger'
		| 'secondary'
		| 'highlight'
		| 'red'
		| 'orange'
		| 'green'
		| 'blue'
		| 'purple'
	hoverFilled?: boolean
	hoverFilledOnly?: boolean
	remainOnClick?: boolean
	disabled?: boolean
	tooltip?: string
}

export type Option = Divider | Item

withDefaults(
	defineProps<{
		options: Option[]
		disabled?: boolean
		dropdownId?: string
		tooltip?: string
		placement?: string
	}>(),
	{
		options: () => [],
		disabled: false,
		dropdownId: undefined,
		tooltip: undefined,
		placement: 'bottom-end',
	},
)

defineOptions({
	inheritAttrs: false,
})

const dropdown: Ref<InstanceType<typeof PopoutMenu> | null> = ref(null)

const close = () => {
	dropdown.value?.hide()
}

const open = () => {
	dropdown.value?.show()
}

function isDivider(option: BaseOption): option is Divider {
	return 'divider' in option
}

defineExpose({ open, close })
</script>

<style lang="scss" scoped>
.btn {
	white-space: nowrap;
	width: 100%;
	box-shadow: none;
	--text-color: var(--color-base);
	--background-color: transparent;
	justify-content: flex-start;

	&:not(:last-child) {
		margin-bottom: var(--gap-xs);
	}
}
</style>
