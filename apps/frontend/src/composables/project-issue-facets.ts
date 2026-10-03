import type { Labrinth } from '@modrinth/api-client'

type Facet = Labrinth.Threads.v3.ThreadIssue['facets'][number]

/** Temporary shared facet addressed state until the backend supports facet updates. */
export function useMockProjectIssueFacets() {
	const addressed = useState<Record<string, boolean>>('mock-project-issue-facets', () => ({}))

	function isFacetAddressed(facet: Facet): boolean {
		return facet.verdict !== 'open' || addressed.value[facet.id] === true
	}

	function addressFacets(facets: Facet[]) {
		for (const facet of facets) addressed.value[facet.id] = true
	}

	return { isFacetAddressed, addressFacets }
}
