<template>
	<Teleport to="#teleports">
		<transition name="fade">
			<div
				v-show="shown && !isMobileActiveSubmenu"
				ref="contextMenu"
				class="context-menu"
				:style="menuStyle"
				role="menu"
				@keydown="handleMenuKeydown"
				@mousemove="(event) => handleMenuMouseMove(event, 'menu')"
			>
				<template v-for="(option, index) in options" :key="index">
					<hr v-if="isDivider(option)" class="divider" />
					<button
						v-else-if="isOptionVisible(option)"
						:ref="(element) => setOptionButtonRef(index, element)"
						type="button"
						class="item clickable"
						:class="[
							option.color ?? 'base',
							{
								active: index === activeOptionIndex,
								'safe-triangle-hover': index === pendingOptionIndex,
							},
						]"
						role="menuitem"
						:aria-haspopup="hasChildren(option) ? 'menu' : undefined"
						:aria-expanded="hasChildren(option) ? index === activeOptionIndex : undefined"
						@click.stop="handleOptionClick(option, index)"
						@focus="handleOptionFocus(option, index)"
						@mouseenter="handleOptionMouseEnter(option, index)"
					>
						<span class="item-content"><slot :name="option.name" /></span>
						<ChevronRightIcon v-if="hasChildren(option)" class="submenu-chevron" />
					</button>
				</template>
			</div>
		</transition>
	</Teleport>

	<Teleport to="#teleports">
		<transition name="fade">
			<div
				v-if="shown && activeOption && hasSubmenuPosition"
				ref="submenu"
				class="context-menu submenu"
				:style="submenuStyle"
				role="menu"
				@keydown="handleSubmenuKeydown"
				@mouseenter="handleSubmenuMouseEnter"
				@mouseleave="isCursorInsideSubmenu = false"
				@mousemove="(event) => handleMenuMouseMove(event, 'submenu')"
			>
				<button
					v-if="isMobileSubmenuLayout"
					type="button"
					class="item clickable base mobile-back"
					@click.stop="returnToMenu"
				>
					<ChevronLeftIcon class="submenu-chevron" />
					<span class="item-content"><slot :name="activeOption.name" /></span>
				</button>
				<template v-for="(option, index) in activeOption.children" :key="index">
					<hr v-if="isDivider(option)" class="divider" />
					<button
						v-else-if="isOptionVisible(option)"
						:ref="(element) => setSubmenuButtonRef(index, element)"
						type="button"
						class="item clickable"
						:class="option.color ?? 'base'"
						role="menuitem"
						@click.stop="optionClicked(option.name)"
					>
						<span class="item-content"><slot :name="option.name" /></span>
					</button>
				</template>
			</div>
		</transition>
	</Teleport>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'

import type { ContextMenuEmit } from './types'
import { useContextMenu } from './use-context-menu'
import { hasChildren, isDivider } from './utils'

const emit = defineEmits<ContextMenuEmit>()

const {
	shown,
	contextMenu,
	submenu,
	options,
	menuStyle,
	submenuStyle,
	activeOption,
	activeOptionIndex,
	pendingOptionIndex,
	hasSubmenuPosition,
	isMobileSubmenuLayout,
	isMobileActiveSubmenu,
	isCursorInsideSubmenu,
	isOptionVisible,
	setOptionButtonRef,
	setSubmenuButtonRef,
	showMenu,
	hideMenu,
	handleOptionClick,
	optionClicked,
	handleOptionFocus,
	handleOptionMouseEnter,
	returnToMenu,
	handleSubmenuMouseEnter,
	handleMenuMouseMove,
	handleMenuKeydown,
	handleSubmenuKeydown,
} = useContextMenu(emit)

defineExpose({ showMenu, hideMenu })
</script>

<style lang="scss" scoped>
.context-menu {
	background-color: var(--color-raised-bg);
	border-radius: var(--radius-md);
	box-shadow: var(--shadow-floating);
	border: 1px solid var(--color-divider);
	margin: 0;
	position: fixed;
	z-index: 1000000;
	overflow: hidden;
	padding: var(--gap-sm);
	min-width: 12rem;
	max-width: calc(100vw - 1.25rem);

	&.submenu {
		z-index: 1000001;
	}

	.item {
		align-items: center;
		background: transparent;
		border: 0;
		box-shadow: none;
		color: var(--color-text-primary);
		cursor: pointer;
		display: flex;
		font-family: inherit;
		font-size: inherit;
		font-weight: 500;
		gap: var(--gap-sm);
		justify-content: space-between;
		padding: var(--gap-sm);
		border-radius: var(--radius-sm);
		text-align: left;
		width: 100%;

		.item-content {
			align-items: center;
			display: flex;
			gap: var(--gap-sm);
			min-width: 0;
		}

		.submenu-chevron {
			color: var(--color-text-secondary);
			flex-shrink: 0;
			height: 1.25rem;
			width: 1.25rem;
			margin-right: -0.25rem;
		}

		:deep(svg) {
			color: var(--color-base);
		}

		&:hover:not(.safe-triangle-hover),
		&:active,
		&:focus-visible,
		&.active {
			:deep(svg) {
				color: inherit;
			}

			&.base {
				background-color: var(--color-button-bg);
				color: var(--color-contrast);
			}

			&.primary {
				background-color: var(--color-brand);
				color: var(--color-accent-contrast);
				font-weight: 500;
			}

			&.danger {
				background-color: var(--color-red);
				color: var(--color-accent-contrast);
				font-weight: 500;
			}

			&.contrast {
				background-color: var(--color-orange);
				color: var(--color-accent-contrast);
				font-weight: 500;
			}
		}
	}

	.mobile-back {
		border-bottom: 1px solid var(--color-divider);
		border-radius: 0;
		margin: calc(var(--gap-sm) * -1) calc(var(--gap-sm) * -1) var(--gap-sm);
		width: calc(100% + var(--gap-sm) * 2);
	}

	.divider {
		border: 1px solid var(--color-divider);
		margin: var(--gap-sm);
		pointer-events: none;
	}
}

.fade-enter-active,
.fade-leave-active {
	transition: opacity 0.2s ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
