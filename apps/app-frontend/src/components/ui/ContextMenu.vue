<template>
	<transition name="fade">
		<div
			v-show="shown"
			ref="contextMenu"
			class="context-menu"
			:style="{
				left: left,
				top: top,
			}"
		>
			<div v-for="(option, index) in options" :key="index" @click.stop="optionClicked(option.name)">
				<hr v-if="option.type === 'divider'" class="divider" />
				<div
					v-else-if="!(isLinkedData(item) && option.name === `add_content`)"
					class="item clickable"
					:class="[option.color ?? 'base']"
				>
					<slot :name="option.name" />
				</div>
			</div>
		</div>
	</transition>
</template>

<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

const emit = defineEmits(['menu-closed', 'option-clicked'])

const item = ref(null)
const contextMenu = ref(null)
const options = ref([])
const left = ref('0px')
const top = ref('0px')
const shown = ref(false)

defineExpose({
	showMenu: (event, passedItem, passedOptions) => {
		item.value = passedItem
		options.value = passedOptions

		// show to get dimensions
		shown.value = true

		// then, adjust position if overflowing
		nextTick(() => {
			const menuWidth = contextMenu.value?.clientWidth || 200
			const menuHeight = contextMenu.value?.clientHeight || 100
			const minFromEdge = 10

			if (event.pageX + menuWidth + minFromEdge >= window.innerWidth) {
				left.value = Math.max(minFromEdge, event.pageX - menuWidth - minFromEdge) + 'px'
			} else {
				left.value = event.pageX + minFromEdge + 'px'
			}

			if (event.pageY + menuHeight + minFromEdge >= window.innerHeight) {
				top.value = Math.max(minFromEdge, event.pageY - menuHeight - minFromEdge) + 'px'
			} else {
				top.value = event.pageY + minFromEdge + 'px'
			}
		})
	},
})

const isLinkedData = (item) => {
	if (item.instance != undefined && item.instance.linked_data) {
		return true
	} else if (item != undefined && item.linked_data) {
		return true
	}
	return false
}

const hideContextMenu = () => {
	shown.value = false
	emit('menu-closed')
}

const optionClicked = (option) => {
	emit('option-clicked', {
		item: item.value,
		option: option,
	})
	hideContextMenu()
}

const onEscKeyRelease = (event) => {
	if (event.keyCode === 27) {
		hideContextMenu()
	}
}

const handleClickOutside = (event) => {
	const elements = document.elementsFromPoint(event.clientX, event.clientY)
	if (
		contextMenu.value &&
		contextMenu.value.$el !== event.target &&
		!elements.includes(contextMenu.value.$el)
	) {
		hideContextMenu()
	}
}

onMounted(() => {
	window.addEventListener('click', handleClickOutside)
	document.body.addEventListener('keyup', onEscKeyRelease)
})

onBeforeUnmount(() => {
	window.removeEventListener('click', handleClickOutside)
	document.removeEventListener('keyup', onEscKeyRelease)
})
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

	.item {
		align-items: center;
		color: var(--color-base);
		cursor: pointer;
		display: flex;
		gap: var(--gap-sm);
		padding: var(--gap-sm);
		border-radius: var(--radius-sm);

		&:hover,
		&:active {
			&.base {
				background-color: var(--color-button-bg);
				color: var(--color-contrast);
			}

			&.primary {
				background-color: var(--color-brand);
				color: var(--color-accent-contrast);
				font-weight: bold;
			}

			&.danger {
				background-color: var(--color-red);
				color: var(--color-accent-contrast);
				font-weight: bold;
			}

			&.contrast {
				background-color: var(--color-orange);
				color: var(--color-accent-contrast);
				font-weight: bold;
			}
		}
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
