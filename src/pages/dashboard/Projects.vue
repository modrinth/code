<template>
  <layout>
    <h2>My projects</h2>

    <h3>Mods</h3>
    <table>
      <colgroup>
        <col span="2">
      </colgroup>
      <thead>
        <tr>
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
            <g-image src="~/assets/images/aof-mini.png" />
            All of Fabric Mini
          </td>
          <td>Contributor</td>
          <td><span class="badge red">Abandoned</span></td>
          <td>357</td>
          <td>Apr 1, 2020</td>
        </tr>
        <tr>
          <td>Finite Water</td>
          <td>Owner</td>
          <td><span class="badge green">Active</span></td>
          <td>274</td>
          <td>Nov 4, 2019</td>
        </tr>
        <tr>
          <td>Vivatech</td>
          <td>Contributor</td>
          <td><span class="badge green">Active</span></td>
          <td>2,381</td>
          <td>Jul 31, 2019</td>
        </tr>
      </tbody>
    </table>
  </layout>
</template>

<style lang="scss">
h2 {
  margin-top: 0;
  margin-bottom: 1.5rem;
}

h3 {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.badge {
  border-radius: 1rem;
  font-size: 0.8rem;
  font-weight: bold;
  letter-spacing: 0.02rem;
  padding: 0.25rem 0.5rem;

  &.red {
    background-color: #FED7D7;
    color: #9B2C2C;
  }

  &.green {
    background-color: #C6F6D5;
    color: #276749;
  }
}

table {
  background: var(--color-bg);
  border-collapse: collapse;
  border-radius: 0.5rem;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  width: 100%;

  * {
    text-align: start;
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
    padding: 1rem 1.25rem;
  }

  th {
    color: #718096;
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    text-transform: uppercase;
  }

  td {
    &:nth-child(5n - 4) {
      max-width: 50%;
    }

    &:not(:nth-child(5n - 4)) {
      min-width: 15%;
    }
  }
}
</style>
