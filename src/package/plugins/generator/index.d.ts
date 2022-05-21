export default function Generator(options: PluginOptions): {
    name: string;
    buildStart(): Promise<void>;
};
export interface PluginOptions {
    projectColors: boolean;
    landingPage: boolean;
    gameVersions: boolean;
    tags: boolean;
}
