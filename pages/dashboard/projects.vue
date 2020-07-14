<template>
  <div class="content">
    <h2>My projects</h2>

    <div class="section-header">
      <h3>Mods</h3>
      <button>Create a new mod</button>
    </div>
    <table>
      <thead>
        <tr>
          <th></th>
          <th>Name</th>
          <th>Role</th>
          <th>Status</th>
          <th>Downloads</th>
          <th>Last updated</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>
            <img class="rounded-md" src="~/assets/images/aof-mini.png" />
          </td>
          <td>Finite Water</td>
          <td>Owner</td>
          <td><span class="badge green">Active</span></td>
          <td>274</td>
          <td>Nov 4, 2019</td>
        </tr>
        <tr>
          <td>
            <img class="rounded-md" src="~/assets/images/aof-mini.png" />
          </td>
          <td>Vivatech</td>
          <td>Contributor</td>
          <td><span class="badge green">Active</span></td>
          <td>2,381</td>
          <td>Jul 31, 2019</td>
        </tr>
      </tbody>
    </table>

    <div class="section-header">
      <h3>Modpacks</h3>
      <button>Create a new modpack</button>
    </div>
    <table>
      <thead>
        <tr>
          <th></th>
          <th>Name</th>
          <th>Role</th>
          <th>Status</th>
          <th>Downloads</th>
          <th>Last updated</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>
            <img class="rounded-md" src="~/assets/images/aof-mini.png" />
          </td>
          <td>All of Fabric Mini</td>
          <td>Contributor</td>
          <td><span class="badge red">Abandoned</span></td>
          <td>357</td>
          <td>Apr 1, 2020</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style lang="scss">
.section-header {
  display: flex;
  margin-bottom: 1rem;

  & > * {
    margin-right: 1rem;
  }
}

table {
  background: var(--color-bg);
  border-collapse: collapse;
  border-radius: 0.5rem;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  table-layout: fixed;
  width: 100%;

  * {
    text-align: left;
  }

  tr:not(:last-child),
  tr:first-child {
    th,
    td {
      border-bottom: 1px solid var(--color-grey-2);
    }
  }

  th,
  td {
    &:first-child {
      text-align: center;
      width: 5%;
    }

    &:nth-child(2) {
      padding-left: 0;
      width: 30%;
    }
  }

  th {
    color: #718096;
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    padding: 1rem 1rem;
    text-transform: uppercase;
  }

  td {
    padding: 0.25rem 1rem;

    img {
      height: 3rem;
      width: 3rem;
    }
  }
}
</style>
