export {
	type BreadcrumbDefinition,
	type BreadcrumbHandle,
	type BreadcrumbManager,
	type BreadcrumbVisual,
	injectBreadcrumbManager,
	provideBreadcrumbManager,
	provideBreadcrumbParent,
	type ResolvedBreadcrumb,
} from './context'
export { createBreadcrumbManager } from './create-breadcrumb-manager'
export { useBreadcrumb, useRootBreadcrumb } from './use-breadcrumb'
