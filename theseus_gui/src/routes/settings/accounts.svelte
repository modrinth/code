<script>
  import { Button } from 'omorphia';
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from 'svelte';

  let user = { uninitialized: true };

  function fetchUser() {
    invoke('auth_users')
      .then((res) => {
        user = res[0];
      })
      .catch((error) => console.error(error));
  }

  function userAdd() {
    invoke('auth_user_add')
      .then(() => {
        fetchUser();
      })
      .catch((error) => console.error(error));
  }

  function userRemove() {
    invoke('auth_remove_user', { user: user.id })
      .then((user = undefined))
      .catch((error) => console.error(error));
  }

  onMount(() => {
    fetchUser();
  });
</script>

<div class="settings-accounts">
  {#if user !== undefined}
    {#if user.uninitialized}
      <div>
        <p>Loading accounts...</p>
      </div>
    {:else}
      <div>
        Username: {user.username}
        <br />
        UUID: {user.id}
      </div>
    {/if}
  {:else}
    <div>
      <p>No accounts found.</p>
    </div>
  {/if}

  <div class="button-row">
    <Button color="primary" class="button-row__items" on:click={userAdd}>Add an account</Button>
    <Button
      color="primary"
      class="button-row__items"
      disabled={user === undefined || user.uninitialized}
      on:click={userRemove}>Log out</Button
    >
  </div>
</div>

<style lang="postcss">
  .settings-accounts {
    margin: 1rem;
  }

  .button-row {
    display: flex;
    flex-direction: row;
    padding: 1rem 0;
    grid-gap: 1rem;

    &__items {
      display: flex;
      grid-gap: 1rem;
      align-items: flex-start;
      overflow-x: auto;
      padding: 0 1rem;

      /* Hide scrollbar */
      -ms-overflow-style: none;
      scrollbar-width: none;
    }
  }

  .button-acc {
    margin-top: 0.5rem;
  }
</style>
