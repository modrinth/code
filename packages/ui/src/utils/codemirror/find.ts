import {
	closeSearchPanel,
	findNext,
	findPrevious,
	getSearchQuery,
	openSearchPanel,
	replaceAll,
	replaceNext,
	search,
	SearchQuery,
	searchPanelOpen,
	setSearchQuery,
} from '@codemirror/search'
import { EditorSelection } from '@codemirror/state'
import type { EditorView } from '@codemirror/view'

/** How many matches a search has, and which of them (from 1) is selected, or `0` for none. */
export interface FindMatches {
	total: number
	current: number
}

const NO_MATCHES: FindMatches = { total: 0, current: 0 }

/**
 * Search state for an editor whose find bar is drawn by the page rather than by CodeMirror: the
 * panel it opens is empty, but keeps every match highlighted while searching.
 */
export const findWithoutPanel = search({
	createPanel: (view) => ({ dom: view.dom.ownerDocument.createElement('div') }),
})

export function countMatches(view: EditorView): FindMatches {
	const query = getSearchQuery(view.state)
	if (!query.valid) return NO_MATCHES
	const selection = view.state.selection.main
	const cursor = query.getCursor(view.state)
	let total = 0
	let current = 0
	for (let match = cursor.next(); !match.done; match = cursor.next()) {
		total++
		if (match.value.from === selection.from && match.value.to === selection.to) current = total
	}
	return { total, current }
}

function setQuery(view: EditorView, text: string, replacement = '') {
	view.dispatch({
		effects: setSearchQuery.of(new SearchQuery({ search: text, replace: replacement, literal: true })),
	})
}

/**
 * Searches for the text (ignoring case) and selects its first match from where the selection
 * starts, so a search that's typed out keeps its place.
 */
export function findText(view: EditorView, text: string) {
	if (!text) {
		clearFind(view)
		return NO_MATCHES
	}
	if (!searchPanelOpen(view.state)) openSearchPanel(view)
	setQuery(view, text)
	view.dispatch({ selection: EditorSelection.cursor(view.state.selection.main.from) })
	findNext(view)
	return countMatches(view)
}

export function findNextMatch(view: EditorView) {
	findNext(view)
	return countMatches(view)
}

export function findPreviousMatch(view: EditorView) {
	findPrevious(view)
	return countMatches(view)
}

/** Replaces the selected match (or else the next one) and selects the one after it. */
export function replaceMatch(view: EditorView, replacement: string) {
	setQuery(view, getSearchQuery(view.state).search, replacement)
	replaceNext(view)
	return countMatches(view)
}

export function replaceAllMatches(view: EditorView, replacement: string) {
	setQuery(view, getSearchQuery(view.state).search, replacement)
	replaceAll(view)
	findNext(view)
	return countMatches(view)
}

export function clearFind(view: EditorView) {
	setQuery(view, '')
	closeSearchPanel(view)
}
