import { computed, type Ref, ref } from 'vue'

import { dismissFloatingMenus } from '../providers/floating-menu'
import { dismissTooltip } from '../providers/tooltip'

const isClient = typeof window !== 'undefined'

type ModalStackState = {
	stack: symbol[]
	stackSizeRef: Ref<number>
}

const MODAL_STACK_STATE_KEY = '__modrinth_ui_modal_stack_state__' as const
const globalScope = globalThis as typeof globalThis & {
	[MODAL_STACK_STATE_KEY]?: ModalStackState
}
const modalStackState: ModalStackState = globalScope[MODAL_STACK_STATE_KEY] ?? {
	stack: [],
	stackSizeRef: ref(0),
}
globalScope[MODAL_STACK_STATE_KEY] = modalStackState

const { stack, stackSizeRef } = modalStackState

export const MODAL_STACK_BASE_Z = 100
export const MODAL_STACK_STEP_Z = 10
export const MODAL_OVERLAY_Z_OFFSET = 19
export const MODAL_TAURI_Z_OFFSET = 20
export const MODAL_CONTAINER_Z_OFFSET = 21

export function getModalStackZBase(stackDepth: number) {
	return MODAL_STACK_BASE_Z + stackDepth * MODAL_STACK_STEP_Z
}

export function useModalStack() {
	const id = Symbol()

	function push() {
		if (isClient && !stack.includes(id)) {
			dismissTooltip()
			dismissFloatingMenus()
			stack.push(id)
			stackSizeRef.value = stack.length
		}
	}

	function pop() {
		if (!isClient) return
		const idx = stack.indexOf(id)
		if (idx !== -1) {
			stack.splice(idx, 1)
			stackSizeRef.value = stack.length
		}
	}

	function isTopmost() {
		if (!isClient) return true
		return stack.length === 0 || stack[stack.length - 1] === id
	}

	function stackSize() {
		return isClient ? stack.length : 0
	}

	const hasModal = computed(() => stackSizeRef.value > 0)
	const stackCount: Readonly<Ref<number>> = stackSizeRef

	return { push, pop, isTopmost, stackSize, hasModal, stackCount }
}
