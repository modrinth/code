<template>
  <div v-if="version">
    <Card>
      <div class="version-title">
        <h2>{{ version.name }}</h2>
        <span v-if="version.featured">Auto-Featured</span>
      </div>
      <div class="button-group">
        <Button color="primary">
          <DownloadIcon/>
          Download
        </Button>
        <Button :link="`/project/${$route.params.id}/versions`">
          <LeftArrowIcon/>
          Back to list
        </Button>
        <Button>
          <ReportIcon/>
          Report
        </Button>
        <a :href="`https://modrinth.com/mod/${$route.params.id}/version/${$route.params.version}`" rel="external" target="_blank" class="btn">
          <ExternalIcon/>
          Modrinth Website
        </a>
      </div>
    </Card>
  <div class="version-container">
    <div class="description-cards">
      <Card >
        <h3 class="card-title">Changelog</h3>
        <div class="markdown-body" v-html="renderString(version.changelog)"/>
      </Card>
      <Card >
        <h3 class="card-title">Files</h3>
        <Card v-for="file in version.files"  :key="file.id" :class="{'primary': file.primary}" class="file">
          <span class="label">
            <FileIcon/>
            <span>
              <span class="title">
                {{ file.filename }}
              </span>
              ({{ formatBytes(file.size) }})
              <span v-if="file.primary" class="primary-label">
                Primary
              </span>
            </span>
          </span>
          <Button class="download">
            <DownloadIcon/>
            Download
          </Button>
        </Card>
      </Card>
      <Card v-if="dependencies[0]">
        <h2>Dependencies</h2>
        <div v-for="dependency in dependencies" :key="dependency.id" class="btn dependency">
          <Avatar size="sm" :src="dependency.icon_url"/>
          <span> {{ dependency.title }} </span>
        </div>
      </Card>
    </div>
    <Card class="metadata-card">
      <h3 class="card-title">Metadata</h3>
      <div class="metadata">
        <div class="metadata-item">
          <span class="metadata-label">Release Channel</span>
          <span class="metadata-value"><Badge :color="releaseColor(version.version_type)" :type="version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)"/></span>
        </div>
        <div class="metadata-item">
          <span class="metadata-label">Version Number</span>
          <span class="metadata-value">{{ version.version_number }}</span>
        </div>
        <div class="metadata-item">
          <span class="metadata-label">Loaders</span>
          <span class="metadata-value">{{ version.loaders.map(loader => loader.charAt(0).toUpperCase() + loader.slice(1)).join(', ') }}</span>
        </div>
        <div class="metadata-item">
          <span class="metadata-label">Game Versions</span>
          <span class="metadata-value"> {{version.game_versions.join(', ')}} </span>
        </div>
        <div class="metadata-item">
          <span class="metadata-label">Downloads</span>
          <span class="metadata-value">{{ version.downloads }}</span>
        </div>
        <div class="metadata-item">
          <span class="metadata-label">Publication Date</span>
          <span class="metadata-value">
            {{ new Date(version.date_published).toLocaleString('en-US', {month: 'long', day: 'numeric', year: 'numeric',})}}
            at
            {{ new Date(version.date_published).toLocaleString('en-US', {hour: 'numeric', minute: 'numeric', second: 'numeric', hour12: true})}}
          </span>
        </div>
        <div class="metadata-item" v-if="author">
          <span class="metadata-label">Author</span>
          <a :href="`https://modrinth.com/user/${author.user.username}`" rel="external" target="_blank" class="metadata-value btn author">
            <Avatar size="sm" :src="author.user.avatar_url" circle/>
            <span>
              <strong>
                {{ author.user.username }}
              </strong>
              <br/>
              {{ author.role }}
            </span>
          </a>
        </div>
        <div class="metadata-item">
          <span class="metadata-label">Version ID</span>
          <span class="metadata-value">{{ version.downloads }}</span>
        </div>
      </div>
    </Card>
  </div>
  </div>
</template>

<script setup>
import {Card, Button, DownloadIcon, FileIcon, Avatar, LeftArrowIcon, ReportIcon, Badge, ExternalIcon, formatBytes, renderString} from 'omorphia'
import { releaseColor } from '@/helpers/utils'
</script>

<script>
const url = (id) => `https://api.modrinth.com/v2/version/${id}`;
const projects = (id) => `https://api.modrinth.com/v2/projects?ids=[${id}]`;
const project = (id) => `https://api.modrinth.com/v2/project/${id}`;
const user = (id) => `https://api.modrinth.com/v2/project/${id}/members`;

export default {
  data() {
    return {
      version: null,
      author: null,
      dependencies: []
    }
  },
  async mounted() {
    try {
      const response = await fetch(url(this.$route.params.version));
      const data = await response.json();

      this.version = data;

      const userResponse = await fetch(user(this.$route.params.id));
      this.author = (await userResponse.json()).find(obj => obj.user.id === data.author_id);

      if (data.dependencies) {
        if (data.dependencies.length > 1) {
          const depIds = data.dependencies.map(dep => `"${dep.project_id}"`).toString();
          const projectsResponse = await fetch(projects(depIds));
          this.dependencies = await projectsResponse.json();
        } else {
          const projectId = data.dependencies[0].project_id;
          const projectResponse = await fetch(project(projectId));
          this.dependencies = [(await projectResponse.json())];
        }
      }
    } catch (error) {
      console.log(error);
    }
  }
}
</script>

<style scoped lang="scss">
.version-container {
  display: flex;
  flex-direction: row;
  gap: 1rem;
}

.version-title {
  margin-bottom: 1rem;
  h2 {
    font-size: var(--font-size-2xl);
    font-weight: 700;
    color: var(--color-contrast);
    margin: 0;
  }
}

.dependency {
  display: flex;
  padding: 0.5rem 1rem 0.5rem 0.5rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
}

.file {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  background: var(--color-button-bg);
  color: var(--color-base);
  padding: 0.5rem 1rem;

  .download {
    margin-left: auto;
    background-color: var(--color-raised-bg);
  }

  .label {
    display: flex;
    margin: auto 0 auto;
    gap: 0.5rem;

    .title {
      font-weight: bolder;
      word-break: break-all;
    }

    svg {
      min-width: 1.1rem;
      min-height: 1.1rem;
      width: 1.1rem;
      height: 1.1rem;
      margin: auto 0;
    }

    .primary-label {
      font-style: italic;
    }
  }
}

.primary {
  background: var(--color-brand-highlight);
  color: var(--color-contrast);
}


.button-group {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 0.5rem;
}

.card-title {
  font-size: var(--font-size-lg);
  color: var(--color-contrast);
  margin: 0 0 0.5rem;
}

.description-cards {
  width: 100%;
}

.metadata-card {
  width: 20rem;
  height: min-content;
}

.metadata {
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: 1rem;

  .metadata-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;

    .metadata-label {
      font-weight: bold;
    }
  }
}

.author {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  align-items: center;
  text-decoration: none;
  color: var(--color-base);
  background: var(--color-raised-bg);
  padding: 0.5rem;
  width: 100%;
  box-shadow: none;
}
</style>
