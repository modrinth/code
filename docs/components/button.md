# Buttons

## Standard

<DemoContainer>
<Button><BookmarkIcon /> Save</Button>
<Button color="primary"><UploadIcon /> Upload</Button>
<Button color="secondary"><PlusIcon /> Create new instance</Button>
<Button color="highlight"><ScaleIcon /> Submit for review</Button>
<Button color="danger"><TrashIcon /> Delete</Button>
</DemoContainer>

```vue
<Button><BookmarkIcon /> Save</Button>
<Button color="primary"><UploadIcon /> Upload</Button>
<Button color="secondary"><PlusIcon /> Create new instance</Button>
<Button color="highlight"><ScaleIcon /> Submit for review</Button>
<Button color="danger"><TrashIcon /> Delete</Button>
```

## Large

<DemoContainer>
<Button color="primary" large><DownloadIcon /> Download</Button>
<Button color="blue" large><ServerIcon /> Host a Server</Button>
<Button color="purple" large><HeartIcon /> Donate</Button>
</DemoContainer>

```vue
<Button color="primary" large><DownloadIcon /> Download</Button>
<Button color="blue" large><ServerIcon /> Host a Server</Button>
<Button color="purple" large><HeartIcon /> Donate</Button>
```

## Outline

<DemoContainer>
<Button color="primary" outline><DownloadIcon/> Get Modrinth App</Button>
<Button color="red" outline><ReportIcon /> Report project</Button>
</DemoContainer>

```vue
<Button color="primary" outline><DownloadIcon/> Get Modrinth App</Button>
<Button color="red" outline><ReportIcon /> Report project</Button>
```

## Transparent

<DemoContainer>
<Button transparent><IssuesIcon /> Report issues</Button>
<Button transparent><CodeIcon /> View sources</Button>
<Button color="blue" transparent><GlobeIcon/> Visit website</Button>
</DemoContainer>

```vue
<Button transparent><IssuesIcon /> Report issues</Button>
<Button transparent><CodeIcon /> View sources</Button>
<Button color="blue" transparent><GlobeIcon/> Visit website</Button>
```

## Hover-filled

<DemoContainer>
<Button color="green" transparent hoverFilled><PlayIcon /> Play</Button>
<Button color="red" transparent hoverFilled><TrashIcon /> Delete</Button>
<Button color="green" outline hoverFilled><PlayIcon /> Play</Button>
<Button color="red" outline hoverFilled><TrashIcon /> Delete</Button>
</DemoContainer>

```vue
<Button color="green" transparent hoverFilled><PlayIcon /> Play</Button>
<Button color="red" transparent hoverFilled><TrashIcon /> Delete</Button>
<Button color="green" outline hoverFilled><PlayIcon /> Play</Button>
<Button color="red" outline hoverFilled><TrashIcon /> Delete</Button>
```

## Hover-filled-only

<DemoContainer>
<Button color="green" transparent hoverFilledOnly><PlayIcon /> Play</Button>
<Button color="red" transparent hoverFilledOnly><TrashIcon /> Delete</Button>
<Button color="green" outline hoverFilledOnly><PlayIcon /> Play</Button>
<Button color="red" outline hoverFilledOnly><TrashIcon /> Delete</Button>
</DemoContainer>

```vue
<Button color="green" transparent hoverFilledOnly><PlayIcon /> Play</Button>
<Button color="red" transparent hoverFilledOnly><TrashIcon /> Delete</Button>
<Button color="green" outline hoverFilledOnly><PlayIcon /> Play</Button>
<Button color="red" outline hoverFilledOnly><TrashIcon /> Delete</Button>
```

## Icon-only

<DemoContainer>
<Button icon-only><HeartIcon /></Button>
<Button icon-only><XIcon /></Button>
<Button icon-only><MoreHorizontalIcon /></Button>
<Button icon-only transparent><SunIcon /></Button>
<Button icon-only transparent><DropdownIcon /></Button>
</DemoContainer>

```vue
<Button icon-only><HeartIcon /></Button>
<Button icon-only><XIcon /></Button>
<Button icon-only><MoreHorizontalIcon /></Button>
<Button icon-only transparent><SunIcon /></Button>
<Button icon-only transparent><DropdownIcon /></Button>
```

## Joined buttons

<DemoContainer>
<div class="joined-buttons">
  <Button color="primary"><UploadIcon /> Upload</Button>
  <OverflowMenu :options="[
  {
    'id': 'import',
    'action': () => {},
  },
  {
    'id': 'edit',
    'action': () => {}
  }
]" class="btn btn-primary btn-dropdown-animation icon-only">
    <DropdownIcon />
    <template #import>
      <ImportIcon /> Import
    </template>
    <template #edit>
      <EditIcon /> Edit
    </template>
  </OverflowMenu>
</div>
</DemoContainer>

```vue
<div class="joined-buttons">
  <Button color="primary"><UploadIcon /> Upload</Button>
  <OverflowMenu :options="[
  {
    'id': 'import',
    'action': () => {},
  },
  {
    'id': 'edit',
    'action': () => {}
  }
]" class="btn btn-primary btn-dropdown-animation icon-only">
    <DropdownIcon />
    <template #import>
      <ImportIcon /> Import
    </template>
    <template #edit>
      <EditIcon /> Edit
    </template>
  </OverflowMenu>
</div>
```
