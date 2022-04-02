import gameVersions from '$generated/gameVersions.json';

export function formatVersions(versionArray: string[]): string {
	const allVersions = gameVersions.slice().reverse();
	const allReleases = allVersions.filter((x) => x.version_type === 'release');

	const intervals = [];
	let currentInterval = 0;

	for (let i = 0; i < versionArray.length; i++) {
		const index = allVersions.findIndex((x) => x.version === versionArray[i]);
		const releaseIndex = allReleases.findIndex((x) => x.version === versionArray[i]);

		if (i === 0) {
			intervals.push([[versionArray[i], index, releaseIndex]]);
		} else {
			const intervalBase = intervals[currentInterval];

			if (
				(index - intervalBase[intervalBase.length - 1][1] === 1 ||
					releaseIndex - intervalBase[intervalBase.length - 1][2] === 1) &&
				(allVersions[intervalBase[0][1]].version_type === 'release' ||
					allVersions[index].version_type !== 'release')
			) {
				intervalBase[1] = [versionArray[i], index, releaseIndex];
			} else {
				currentInterval += 1;
				intervals[currentInterval] = [[versionArray[i], index, releaseIndex]];
			}
		}
	}

	const newIntervals = [];
	for (let i = 0; i < intervals.length; i++) {
		const interval = intervals[i];

		if (interval.length === 2 && interval[0][2] !== -1 && interval[1][2] === -1) {
			let lastSnapshot = null;
			for (let j = interval[1][1]; j > interval[0][1]; j--) {
				if (allVersions[j].version_type === 'release') {
					newIntervals.push([
						interval[0],
						[
							allVersions[j].version,
							j,
							allReleases.findIndex((x) => x.version === allVersions[j].version),
						],
					]);

					if (lastSnapshot !== null && lastSnapshot !== j + 1) {
						newIntervals.push([[allVersions[lastSnapshot].version, lastSnapshot, -1], interval[1]]);
					} else {
						newIntervals.push([interval[1]]);
					}

					break;
				} else {
					lastSnapshot = j;
				}
			}
		} else {
			newIntervals.push(interval);
		}
	}

	const output = [];

	for (const interval of newIntervals) {
		if (interval.length === 2) {
			output.push(`${interval[0][0]}—${interval[1][0]}`);
		} else {
			output.push(interval[0][0]);
		}
	}

	return output.join(', ');
}

export const getPrimary = (files: Version['files']) =>
	files.find((file) => file.primary) || files[0];

export function downloadUrl(file): string {
	return import.meta.env.VITE_API_URL + `version_file/${file?.hashes.sha1}/download`;
}
