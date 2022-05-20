import { promises as fs } from 'fs';
import { landingPage } from './outputs/landingPage.js';
import { projectColors } from './outputs/projectColors.js';
import { gameVersions } from './outputs/gameVersions.js';
import { tags } from './outputs/tags.js';

const API_URL = 'https://api.modrinth.com/v2/'; //TODO Remove
process.env.VITE_API_URL || process.env?.NODE_ENV === 'development'
    ? 'https://staging-api.modrinth.com/v2/'
    : 'https://api.modrinth.com/v2/';

// Time to live: 7 days
const TTL = 7 * 24 * 60 * 60 * 1000;

export default function Generator(options: PluginOptions) {
    return {
        name: 'rollup-plugin-omorphia-generator',
        async buildStart() {
            let state: State = {};

            try {
                state = JSON.parse(await fs.readFile('./generated/state.json', 'utf8'));
            } catch {
                // File doesn't exist, create folder
                await fs.mkdir('./generated', { recursive: true });

                await fs.writeFile(
                    './generated/state.json',
                    JSON.stringify(
                        {
                            options,
                        },
                        null,
                        2
                    )
                );
            }

            // Don't generate if the last generation was less than TTL and the options are the same
            if (
                state?.lastGenerated &&
                new Date(state.lastGenerated).getTime() + TTL > new Date().getTime() &&
                JSON.stringify(state.options) === JSON.stringify(options)
            ) {
                return;
            }

            if (options.landingPage) await landingPage(API_URL);
            if (options.projectColors) await projectColors(API_URL);
            if (options.gameVersions) await gameVersions(API_URL);
            if (options.tags) await tags(API_URL);

            // Write new state
            state.lastGenerated = new Date().toISOString();
            state.options = options;
            await fs.writeFile('./generated/state.json', JSON.stringify(state, null, 2));
        },
    };
}

export interface PluginOptions {
    projectColors: boolean;
    landingPage: boolean;
    gameVersions: boolean;
    tags: boolean;
}

interface State {
    lastGenerated?: string;
    options?: PluginOptions;
}
