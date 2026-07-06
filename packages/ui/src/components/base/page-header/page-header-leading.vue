<template>
	<div v-if="items.length" class="flex shrink-0 items-center gap-4">
		<template v-for="(item, index) in items" :key="item.id ?? index">
			<div v-if="item.type === 'button'" :class="item.wrapperClass ?? defaultButtonWrapperClass">
				<ButtonStyled
					circular
					:color="item.color ?? 'standard'"
					:size="item.size ?? 'large'"
					:type="item.buttonType ?? 'standard'"
				>
					<PageHeaderInteractiveWrapper
						:to="item.to"
						:clickable="!!item.onClick"
						:disabled="item.disabled"
						:tooltip="item.tooltip"
						:aria-label="leadingLabel(item)"
						base-class=""
						@click="(event) => item.onClick?.(event)"
					>
						<component
							:is="item.icon"
							v-if="item.icon"
							aria-hidden="true"
							v-bind="item.iconProps"
						/>
					</PageHeaderInteractiveWrapper>
				</ButtonStyled>
			</div>
			<Avatar
				v-else-if="item.type === 'avatar'"
				:src="item.src"
				:alt="item.alt ?? ''"
				:size="item.avatarSize ?? '64px'"
				:tint-by="item.tintBy ?? null"
				:circle="item.circle ?? false"
				:no-shadow="item.noShadow ?? false"
				:raised="item.raised ?? false"
				:loading="item.loading ?? 'eager'"
				:class="item.class"
			/>
			<component :is="item.component" v-else :class="item.class" v-bind="item.componentProps" />
		</template>
	</div>
</template>

<script setup lang="ts">
import Avatar from '../Avatar.vue'
import ButtonStyled from '../ButtonStyled.vue'
import PageHeaderInteractiveWrapper from './page-header-interactive-wrapper.vue'
import type { PageHeaderButtonLeading, PageHeaderLeading } from './types'

defineProps<{
	items: PageHeaderLeading[]
}>()

const defaultButtonWrapperClass = 'flex size-16 shrink-0 items-center justify-center'

function leadingLabel(item: PageHeaderButtonLeading) {
	return item.ariaLabel ?? item.tooltip ?? 'Header action'
}
</script>
