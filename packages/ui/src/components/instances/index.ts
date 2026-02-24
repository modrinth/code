export { default as ContentCardItem } from './ContentCardItem.vue'
export { default as ContentCardTable } from './ContentCardTable.vue'
/**
 * @deprecated Use `ContentCardTable` with `ContentCardItem` instead.
 * This alias is kept for backwards compatibility and will be removed in a future version.
 */
export { default as ContentCard } from './ContentCardItem.vue'
export { default as ContentModpackCard } from './ContentModpackCard.vue'
// export { default as ContentUpdaterModal } from './modals/ContentUpdaterModal.vue'
export { default as ModpackContentModal } from './modals/ModpackContentModal.vue'

export type {
	ContentCardProject,
	ContentCardTableItem,
	ContentCardVersion,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from './types'
