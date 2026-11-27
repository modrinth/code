import type { Ref } from 'vue'

import type { BulkOperationState } from '../../composables/bulk-operations'
import type { ContentItem } from '../../types'

export interface ContentFlowState {
	selectedItems: Readonly<Ref<ContentItem[]>>
	clearSelection: () => void
	removeFromSelection: (id: string) => void
	bulk: BulkOperationState
	markChanging: (id: string) => void
	unmarkChanging: (id: string) => void
	retain: (id: string) => void
	itemType: Readonly<Ref<string>>
}
