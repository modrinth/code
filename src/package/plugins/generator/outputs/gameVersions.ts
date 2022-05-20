import { fetch } from 'undici';
import { promises as fs } from 'fs';
import cliProgress from 'cli-progress';

export async function gameVersions(API_URL: string) {
    const progressBar = new cliProgress.SingleBar({
        format: 'Generating game versions  | {bar} | {percentage}%',
        barCompleteChar: '\u2588',
        barIncompleteChar: '\u2591',
        hideCursor: true,
    });

    progressBar.start(2, 0);

    const gameVersions = await (await fetch(API_URL + 'tag/game_version')).json();

    progressBar.increment();

    // Write JSON file
    await fs.writeFile('./generated/gameVersions.json', JSON.stringify(gameVersions));
    progressBar.increment();

    progressBar.stop();
}
