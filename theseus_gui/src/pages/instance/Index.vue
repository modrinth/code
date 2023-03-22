<template>
  <div class="instance-container">
    <div class="side-cards">
      <Card class="project-card">
        <Avatar size="md" :src="getInstance(instances).icon"/>
        <div class="instance-info">
          <h2 class="name">{{ getInstance(instances).name }}</h2>
          <span class="badge">
          <DownloadIcon/>
          Downloaded
        </span>
          <Categories
              :categories="[
          {
            icon: '<svg xmlns=\'http://www.w3.org/2000/svg\' xml:space=\'preserve\' fill-rule=\'evenodd\' stroke-linecap=\'round\' stroke-linejoin=\'round\' clip-rule=\'evenodd\' viewBox=\'0 0 24 24\'>\n  <path fill=\'none\' d=\'M0 0h24v24H0z\'/>\n  <path fill=\'none\' stroke=\'currentColor\' stroke-width=\'23\' d=\'m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8\' transform=\'matrix(.08671 0 0 .0867 -49.8 -56)\'/>\n</svg>',
            name: 'fabric',
          },
          {
            icon: '<svg xmlns=\'http://www.w3.org/2000/svg\' width=\'24\' height=\'24\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><line x1=\'6\' y1=\'11\' x2=\'10\' y2=\'11\'></line><line x1=\'8\' y1=\'9\' x2=\'8\' y2=\'13\'></line><line x1=\'15\' y1=\'12\' x2=\'15.01\' y2=\'12\'></line><line x1=\'18\' y1=\'10\' x2=\'18.01\' y2=\'10\'></line><path d=\'M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.545-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z\'></path></svg>',
            name: getInstance(instances).version,
          }]"
              type="mod"
              class="tags"
          />
          <hr class="card-divider">
          <div class="stats">
            <div class="stat">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
                   stroke="currentColor"
                   stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="6" y1="11" x2="10" y2="11"></line>
                <line x1="8" y1="9" x2="8" y2="13"></line>
                <line x1="15" y1="12" x2="15.01" y2="12"></line>
                <line x1="18" y1="10" x2="18.01" y2="10"></line>
                <path
                    d="M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.545-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z"></path>
              </svg>
              <strong>120</strong
              ><span class="stat-label"> plays</span>
            </div>
          </div>
          <hr class="card-divider">
          <div class="actions">
            <Button color="primary">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none"
                   stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="5 3 19 12 5 21 5 3"></polygon>
              </svg>
              Play
            </Button>
            <Button color="danger">
              <TrashIcon/>
              Delete
            </Button>
          </div>
        </div>
      </Card>
      <Card>
        Advertisement
      </Card>
      <Card class="quick-actions">
        <Button>
          <TrashIcon />
          Delete
        </Button>
        <Button>
          <TrashIcon />
          Delete
        </Button>
        <Button>
          <TrashIcon />
          Delete
        </Button>
        <Button>
          <TrashIcon />
          Delete
        </Button>
        <Button>
          <TrashIcon />
          Delete
        </Button>
      </Card>
    </div>
    <div class="content">
      <Card class="instance-nav pages-list">
        <RouterLink :to="`/instance/${this.$route.params.id}/`" class="btn">
          <BoxIcon/>
          Mods
        </RouterLink>
        <RouterLink :to="`/instance/${this.$route.params.id}/settings`" class="btn">
          <SettingsIcon/>
          Options
        </RouterLink>
        <RouterLink :to="`/instance/${this.$route.params.id}/logs`" class="btn">
          <FileIcon/>
          Logs
        </RouterLink>
      </Card>
      <router-view/>
    </div>
  </div>
</template>
<script setup>
import {BoxIcon, SettingsIcon, FileIcon, Button, Avatar, Card, Categories, DownloadIcon, TrashIcon} from 'omorphia'
import {useInstances} from '@/store/state'

const instances = useInstances();
instances.fetchInstances();
</script>
<script>
export default {
  methods: {
    getInstance(instances) {
      return instances.instances.find((i) => i.id === parseInt(this.$route.params.id));
    }
  },
}
</script>

<style scoped lang="scss">
.instance-gallery-item {
  width: 9rem;
  height: 9rem;
  background: url('https://avatars1.githubusercontent.com/u/6166773?v=4');
  background-position: center;
  background-size: cover;
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  padding: 0.3rem;
  cursor: pointer;
  transition: all ease-in-out 0.2s;
  border-radius: var(--radius-md);

  &:before {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    background-image: linear-gradient(to bottom right, rgba(50, 50, 50, 0.9), rgba(0, 0, 0, 0.9));
    opacity: 0.5;
    z-index: 10;
    border-radius: inherit;
  }

  &:hover {
    box-shadow: 0 0 4px 4px rgba(0, 0, 0, 0.5);
  }

  p {
    font-weight: bold;
    font-size: 0.75rem;
    color: #fff;
    z-index: 11;
    margin-bottom: 0.2rem;

    &:nth-child(1) {
      font-weight: normal;
      color: #ddd;
      font-size: 0.6rem;
    }
  }
}

Button {
  width: 100%;
}

.side-cards {
  display: flex;
  flex-direction: column;
  height: min-content;
  width: 20rem;
}

.quick-actions {
  display: inline-flex;
  gap: 0.5rem;
  flex-wrap: wrap;

  button {
    width: min-content;
  }
}

.instance-nav {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 1rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
  height: 100%;
}

.name {
  font-size: 1.25rem;
  color: var(--color-contrast);
}

.instance-container {
  display: flex;
  flex-direction: row;
  overflow: hidden;
  margin: 1rem;
  gap: 1rem;
}

.instance-info {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  width: 100%;
}

.badge {
  display: flex;
  align-items: center;
  font-weight: bold;
  width: fit-content;
  color: var(--color-orange);

  svg {
    margin-right: 0.25rem;
  }
}

.pages-list {
  a {
    font-size: 0.9rem;
    font-weight: 400;
    word-spacing: 3px;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-primary);

    &.router-link-active {
      background: var(--color-button-bg);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }

    svg {
      margin-right: 1rem;
      width: 1.2rem;
      height: 1.2rem;
    }
  }
}

.header-nav {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 0.5rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
}

.project-card {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 1rem;
  background: var(--color-raised-bg);
  width: 20rem;
}

.instance-nav {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  justify-content: left;
  padding: 1rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
  height: min-content;
  width: 100%;
}

.actions {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: 0.5rem;

  button {
    width: min-content;
  }
}

.content {
  width: 100%;
  display: flex;
  flex-direction: column;
}

.stats {
  grid-area: stats;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: var(--gap-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--gap-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      height: var(--stat-strong-size);
      width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }

  @media screen and (max-width: 750px) {
    flex-direction: row;
    column-gap: var(--gap-md);
    margin-top: var(--gap-xs);
  }

  @media screen and (max-width: 600px) {
    margin-top: 0;

    .stat-label {
      display: none;
    }
  }
}

.card-divider {
  background-color: var(--color-button-bg);
  border: none;
  color: var(--color-button-bg);
  height: 1px;
  margin: var(--gap-sm) 0;
}
</style>