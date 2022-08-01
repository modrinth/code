<script lang="ts">
    import { Field, Slider, TextInput, Button } from "omorphia"
    import TitledSection from "$components/TitledSection.svelte"
    import WindowSettings from "$components/WindowSettings.svelte"

    export let maxConcurrentDownloads: number = 20;
</script>

<div class="section">
    <TitledSection title="Downloads">
        <Field label="Max concurrent downloads">
            <Slider min=1 max=64 bind:value={maxConcurrentDownloads} />
        </Field>
    </TitledSection>
    <TitledSection title="Override game resolution" toggleable=true>
        <WindowSettings />
    </TitledSection>
    <TitledSection title="Profile hooks">
        <Field label="Pre-launch">
            <TextInput />
        </Field>
        <Field label="Wrapper">
            <TextInput />
        </Field>
        <Field label="Post-exit">
            <TextInput />
        </Field>
    </TitledSection>
    <TitledSection title="Java">
        <Field label="Java 8 installation">
            <TextInput placeholder="/Library/Java/JavaVirtualMachines/zulu-8.jdk/Contents/Home" />
            <div class="button-group">
                <Button>Auto-detect</Button>
                <Button>Browse installations</Button>
                <Button>Test</Button>
            </div>
        </Field>
        <Field label="Java 17 installation">
            <TextInput placeholder="/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home" />
            <div class="button-group">
                <Button>Auto-detect</Button>
                <Button>Browse installations</Button>
                <Button>Test</Button>
            </div>
        </Field>
        <Field label="Minimum memory allocatted (in MB)">
            <TextInput />
        </Field>
        <Field label="Maximum memory allocatted (in MB)">
            <TextInput value="2048" />
        </Field>
        <Field label="Arguments">
            <TextInput/>
        </Field>
    </TitledSection>
</div>

<style lang="postcss">
</style>
