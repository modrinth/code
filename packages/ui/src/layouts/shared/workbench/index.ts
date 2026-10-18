export { SlotOutlet } from './components/slot-outlet'
export { createWorkbench, type WorkbenchOptions } from './composables/create-workbench'
export { default as WorkbenchLayout } from './layout.vue'
export {
	injectWorkbench,
	injectWorkbenchPanel,
	type WorkbenchContext,
	type WorkbenchPanelContext,
} from './providers'
export * from './types'
