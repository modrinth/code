import { fetch } from 'undici';
import { promises as fs } from 'fs';
import cliProgress from 'cli-progress';

export async function tags(API_URL: string) {
    const progressBar = new cliProgress.SingleBar({
        format: 'Generating tags           | {bar} | {percentage}%',
        barCompleteChar: '\u2588',
        barIncompleteChar: '\u2591',
        hideCursor: true,
    });

    progressBar.start(7, 0);

    // eslint-disable-next-line prefer-const
    let [categories, loaders, licenses, donationPlatforms]: any = await Promise.all([
        await (await fetch(API_URL + 'tag/category')).json(),
        await (await fetch(API_URL + 'tag/loader')).json(),
        await (await fetch(API_URL + 'tag/license')).json(),
        await (await fetch(API_URL + 'tag/donation_platform')).json(),
    ]);

    progressBar.update(4);

    // Delete icons from original arrays
    categories = categories.map(({ icon, ...rest }) => rest);
    loaders = loaders.map(({ icon, ...rest }) => rest);

    progressBar.increment();

    // Create single object with icons
    const tagIcons = {
        ...categories.reduce((a, v) => ({ ...a, [v.name]: v.icon }), {}),
        ...loaders.reduce((a, v) => ({ ...a, [v.name]: v.icon }), {}),
    };

    progressBar.increment();

    // Set project types
    const projectTypes = ['mod', 'modpack'];

    // Write JSON file
    await fs.writeFile(
        './generated/tags.json',
        JSON.stringify({ categories, loaders, projectTypes, licenses, donationPlatforms, tagIcons })
    );
    progressBar.increment();

    progressBar.stop();
}
