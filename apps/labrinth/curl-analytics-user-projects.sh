#!/usr/bin/env bash
set -euo pipefail

: "${MODRINTH_USERNAME:?set MODRINTH_USERNAME to the user whose projects should be queried}"
: "${MODRINTH_TOKEN:?set MODRINTH_TOKEN to a Modrinth token with analytics scope}"

API_BASE="${API_BASE:-https://api.modrinth.com}"

project_ids="$(
	curl --fail-with-body --silent --show-error \
		--header "Accept: application/json" \
		"${API_BASE}/v2/user/${MODRINTH_USERNAME}/projects" \
		| jq --compact-output '[.[].id]'
)"

i=0
while true; do
	curl --fail-with-body --silent --show-error \
		--request POST \
		--header "Authorization: Bearer ${MODRINTH_TOKEN}" \
		--header "Content-Type: application/json" \
		--header "Accept: application/json" \
		--data "$(
			jq --null-input --compact-output \
				--argjson project_ids "${project_ids}" \
				'{
					time_range: {
						start: "2020-01-01T00:00:00Z",
						end: "2027-01-01T00:00:00Z",
						resolution: { slices: 1024 }
					},
					project_ids: $project_ids
				}'
		)" \
		"${API_BASE}/v3/analytics/facets"

	echo -e "\ni = $i"
	i=$((i+1))
	sleep 0.25
done
