export { createWorkbench, type WorkbenchOptions } from './composables/workbench-layout'
export { default as WorkbenchLayout } from './layout.vue'
export {
	injectWorkbench,
	injectWorkbenchPanel,
	type WorkbenchContext,
	type WorkbenchPanelContext,
} from './providers/workbench'
export * from './types'
