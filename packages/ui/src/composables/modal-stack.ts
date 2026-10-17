import { useScrollLock } from '@vueuse/core'
import { computed, type Ref, ref } from 'vue'

import { injectAppRoot } from '#ui/providers/app-root.ts'

import { dismissFloatingMenus } from '../providers/floating-menu'
import { dismissTooltip } from '../providers/tooltip'

const isClient = typeof window !== 'undefined'

type ModalStackState = {
	stack: symbol[]
	stackSizeRef: Ref<number>
}

const MODAL_STACK_STATE_KEY = '__modrinth_ui_document_modal_stacks__' as const
const globalScope = globalThis as typeof globalThis & {
	[MODAL_STACK_STATE_KEY]?: WeakMap<Document, ModalStackState>
}
const documentStacks = (globalScope[MODAL_STACK_STATE_KEY] ??= new WeakMap<
	Document,
	ModalStackState
>())

export const MODAL_STACK_BASE_Z = 100
export const MODAL_STACK_STEP_Z = 10
export const MODAL_OVERLAY_Z_OFFSET = 19
export const MODAL_TAURI_Z_OFFSET = 20
export const MODAL_CONTAINER_Z_OFFSET = 21

export function getModalStackZBase(stackDepth: number) {
	return MODAL_STACK_BASE_Z + stackDepth * MODAL_STACK_STEP_Z
}

export function useModalStack(getDocument: () => Document = () => document) {
	const id = Symbol()
	let activeState: ModalStackState | undefined

	function getState() {
		const ownerDocument = getDocument()
		let state = documentStacks.get(ownerDocument)
		if (!state) {
			state = { stack: [], stackSizeRef: ref(0) }
			documentStacks.set(ownerDocument, state)
		}
		return state
	}

	let isLocked: ReturnType<typeof useScrollLock> | undefined = undefined

	/** The app root lives in the main document, so modals in other documents (pop-outs) leave it scrollable. */
	function locksAppRoot(state: ModalStackState) {
		return state === documentStacks.get(document)
	}

	if (isClient) {
		const ctx = injectAppRoot(null)!
		if (ctx) isLocked = useScrollLock(ctx.root)
	}

	function push() {
		if (!isClient || activeState) return
		const state = getState()
		dismissTooltip()
		dismissFloatingMenus()
		state.stack.push(id)
		state.stackSizeRef.value = state.stack.length
		activeState = state
		if (isLocked && locksAppRoot(state)) isLocked.value = true
	}

	function pop() {
		if (!activeState) return
		const { stack, stackSizeRef } = activeState
		const idx = stack.indexOf(id)
		if (idx !== -1) {
			stack.splice(idx, 1)
			stackSizeRef.value = stack.length
		}
		if (stackSizeRef.value <= 0 && isLocked && locksAppRoot(activeState)) isLocked.value = false
		activeState = undefined
	}

	function isTopmost() {
		if (!isClient) return true
		const { stack } = getState()
		return stack.length === 0 || stack[stack.length - 1] === id
	}

	function stackSize() {
		return isClient ? getState().stack.length : 0
	}

	const stackCount: Readonly<Ref<number>> = computed(() =>
		isClient ? getState().stackSizeRef.value : 0,
	)
	const hasModal = computed(() => stackCount.value > 0)

	return { push, pop, isTopmost, stackSize, hasModal, stackCount }
}
