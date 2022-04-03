<script context="module" lang="ts">

</script>

<script lang="ts">
    import { FormField, TextInput, Button } from "omorphia"
</script>

<div class="section">
    <div class="section__title">Java</div>
    <div class="section__items">
        <FormField label="Installation">
            <TextInput placeholder="/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home" />
        </FormField>
        <div class="button-group">
            <Button>Auto-detect</Button>
            <Button>Browse installations</Button>
            <Button>Test</Button>
        </div>

        <FormField label="Arguments">
            <TextInput/>
        </FormField>
    </div>
</div>

<style lang="postcss">
    .section {
        display: flex;
        flex-direction: column;
        padding: 1rem;
        grid-gap: 1rem;

        &__title {
            display: flex;
            grid-gap: 1rem;
            align-items: center;

            &::after {
                flex: 1 1;
                content: " ";
                background-color: hsla(0, 0%, 100%, 0.2);
                height: 0.2rem;
                border-radius: var(--rounded-max);
            }
        }

        &__items {
            display: flex;
            flex-direction: column;
            grid-gap: 1rem;
            padding: 0 8px;
        }
    }
</style>
