<script lang="ts">
    import { FormField, Slider, TextInput, Button } from "omorphia"
    import TitledSection from "$components/TitledSection.svelte"
    import WindowSettings from "$components/WindowSettings.svelte"

    export let maxConcurrentDownloads: number = 20;
</script>

<div class="section">
    <TitledSection title="Downloads">
        <FormField label="Max concurrent downloads">
            <Slider min=1 max=64 bind:value={maxConcurrentDownloads} />
        </FormField>
    </TitledSection>
    <TitledSection title="Override game resolution" toggleable=true>
        <WindowSettings />
    </TitledSection>
    <TitledSection title="Profile hooks">
        <FormField label="Pre-launch">
            <TextInput />
        </FormField>
        <FormField label="Wrapper">
            <TextInput />
        </FormField>
        <FormField label="Post-exit">
            <TextInput />
        </FormField>
    </TitledSection>
    <TitledSection title="Java">
        <FormField label="Java 8 installation">
            <TextInput placeholder="/Library/Java/JavaVirtualMachines/zulu-8.jdk/Contents/Home" />
            <div class="button-group">
                <Button>Auto-detect</Button>
                <Button>Browse installations</Button>
                <Button>Test</Button>
            </div>
        </FormField>
        <FormField label="Java 17 installation">
            <TextInput placeholder="/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home" />
            <div class="button-group">
                <Button>Auto-detect</Button>
                <Button>Browse installations</Button>
                <Button>Test</Button>
            </div>
        </FormField>
        <FormField label="Minimum memory allocatted (in MB)">
            <TextInput />
        </FormField>
        <FormField label="Maximum memory allocatted (in MB)">
            <TextInput value="2048" />
        </FormField>
        <FormField label="Arguments">
            <TextInput/>
        </FormField>
    </TitledSection>
</div>

<style lang="postcss">
</style>
