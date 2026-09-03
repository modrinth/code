import { computed, type Ref, ref } from 'vue'

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

export function useModalStack() {
	const id = Symbol()

	function push() {
		if (isClient && !stack.includes(id)) {
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
