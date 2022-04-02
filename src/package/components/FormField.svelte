<script lang="ts">
    import uniqueId from 'lodash.uniqueid'

    export let required = false;
    export let label: string;

    const id = `form-field-${uniqueId()}`
</script>

<div class="form-field">
    <label for={id} class="text-input__label" class:required>
        {label}
    </label>
    <slot {id}  />
</div>


<style lang="postcss">
    .form-field {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
</style>
