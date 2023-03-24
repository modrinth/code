# Modal
:::raw

<DemoContainer>
  <Button :action="() => this.$refs.reportModal.modal.show()">Show Modal</Button>
  <ModalReport
  ref="reportModal"
  itemType="project"
  :reportTypes="['cringitude', 'rudeness', 'notgamer', 'windowsuser']"
  >
  </ModalReport>
</DemoContainer>
:::

```vue
  <Button :action="() => this.$refs.reportModal.modal.show()">Show Modal</Button>
  <ModalReport
  ref="reportModal"
  itemType="project"
  :reportTypes="['cringitude', 'rudeness', 'notgamer', 'windowsuser']"
  />
```
