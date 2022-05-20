import { fetch } from 'undici';
import { promises as fs } from 'fs';
import cliProgress from 'cli-progress';

export async function landingPage(API_URL: string) {
    const progressBar = new cliProgress.SingleBar({
        format: 'Generating landing page   | {bar} | {percentage}% || {value}/{total} mods',
        barCompleteChar: '\u2588',
        barIncompleteChar: '\u2591',
        hideCursor: true,
    });

    progressBar.start(100, 0);

    // Fetch top 100 mods
    const response = (await (
        await fetch(API_URL + 'search?limit=100&facets=[["project_type:mod"]]')
    ).json()) as Record<string, any>;

    // Simplified array with the format: ['id', 'slug', 'icon_extension']
    const compressed = response.hits
        .filter((project) => project.icon_url)
        .map((project) => {
            progressBar.increment();
            return [
                project.project_id,
                project.slug || '',
                project.icon_url.match(/\.[0-9a-z]+$/i)[0].substring(1),
            ];
        });

    // Write JSON file
    await fs.writeFile(
        './generated/landingPage.json',
        JSON.stringify({
            mods: compressed,
            random: Math.random(),
        })
    );

    progressBar.stop();
}
