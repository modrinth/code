import {Checkbox, Combobox, MarkdownEditor, StyledInput, Toggle} from "@modrinth/ui"
import {markRaw} from "vue"

import {
  withExtraProps,
  withEditable,
  withNoneLabel,
  withEnabled,
  withFix,
  withValue,
  withIcon,
  withId,
  withLayout,
  withMessaging,
  withComponent,
  withOnClick,
  withPriority,
  withRequired,
  withSelectable,
  withStageMeta,
  withTitle,
  withTooltip,
  withShown,
} from "./capabilities"
import type {ComponentNodePropsContext} from "./capabilities"
import {withChildren, withAutoProps} from "./builder"
import ActionButton from "./components/ActionButton.vue"
import {pipe} from "./pipe"
import type {NodeState, NodeStateWithChildren} from "./state"

function getBooleanValue(raw: NodeState): boolean {
  if (typeof raw === "boolean") return raw
  if (raw && typeof raw === "object" && !(raw instanceof Set)) {
    const v = (raw as NodeStateWithChildren).value
    if (typeof v === "boolean") return v
  }
  return false
}

function setBooleanValue(raw: NodeState, next: boolean): NodeState {
  if (raw && typeof raw === "object" && !(raw instanceof Set)) {
    const {value: _v, ...children} = raw as NodeStateWithChildren & Record<string, NodeState>
    if (Object.keys(children).length > 0) return {...children, value: next}
  }
  return next
}

export const booleanValue = {
  _getValue: getBooleanValue,
  _setValue: setBooleanValue,
  _isActive: (v: boolean) => v === true,
}

export function toggle(id: string, label: string) {
  return pipe(
    {label} as { label: string },
    (n) => withId(n, id),
    withChildren,
    withShown,
    withIcon,
    withTooltip,
    withTitle,
    withMessaging,
    withRequired,
    withFix,
    withEnabled,
    withPriority,
    (n) => withValue(n, booleanValue),
    (n) =>
      withComponent(n, {
        component: markRaw(ActionButton),
        componentProps: (ctx) => ({
          label: n.label,
          icon: n._icon,
          needsAttention: ctx.nodeFacts?.needsAttention ?? false,
          fixActionable: ctx.nodeFacts?.fixActionable ?? false,
          tooltip: ctx.tooltip,
        }),
      }),
  )
}

export function button(label?: string) {
  return pipe(
    {label} as {label?: string},
    withShown,
    withIcon,
    withTooltip,
    withEnabled,
    withOnClick,
  )
}

export function check(id: string, label: string) {
  return withAutoProps(
    pipe(
      {label} as { label: string },
      (n) => withId(n, id),
      withChildren,
      withShown,
      withTooltip,
      withTitle,
      withMessaging,
      withRequired,
      withFix,
      withEnabled,
      (n) => withValue(n, booleanValue),
      (n) =>
        withComponent(n, {
          component: markRaw(Checkbox),
          componentProps: () => ({label}),
        }),
      withExtraProps,
    ),
  )
}

export function toggleSwitch(id: string, label: string) {
  return withAutoProps(
    pipe(
      {label} as { label: string },
      (n) => withId(n, id),
      withChildren,
      withShown,
      withTooltip,
      withTitle,
      withMessaging,
      withRequired,
      withFix,
      withEnabled,
      (n) => withValue(n, booleanValue),
      (n) => withComponent(n, {component: markRaw(Toggle)}),
      withExtraProps,
    ),
  )
}

export function group(id?: string) {
  const base = pipe(
    {} as {},
    withChildren,
    withShown,
    withTitle,
    withRequired,
    withSelectable,
    withLayout,
  )
  return id === undefined ? base : withId(base, id)
}

export type GroupNode = ReturnType<typeof group>

export function option(value: string, label: string) {
  return pipe(
    {value, label} as { value: string; label: string },
    withChildren,
    withShown,
    withMessaging,
  )
}

type OptionNode = ReturnType<typeof option>

interface HasOptions {
  _options: OptionNode[]
  options(this: this, ...opts: OptionNode[]): this
}

function withOptions<T extends object>(node: T): T & HasOptions {
  return Object.assign(node, {
    _options: [] as OptionNode[],
    options(this: any, ...opts: OptionNode[]) {
      this._options = opts
      return this
    },
  })
}

function getStringValue(raw: NodeState): string {
  return typeof raw === "string" ? raw : ""
}

function setStringValue(_raw: NodeState, next: string): NodeState {
  return next || undefined
}

const dropdownValue = {
  _getValue: getStringValue,
  _setValue: setStringValue,
  _isActive: (v: string) => v !== "",
}

export function dropdown(id: string) {
  return pipe(
    {} as {},
    (n) => withId(n, id),
    withShown,
    withTitle,
    withRequired,
    (n) => withValue(n, dropdownValue),
    withNoneLabel,
    withOptions,
    (n) =>
      withComponent(n, {
        component: markRaw(Combobox),
        componentProps: () => ({
          options: [
            ...(n._none !== undefined ? [{value: "", label: n._none}] : []),
            ...n._options.map((o) => ({value: o.value, label: o.label})),
          ],
          triggerClass:
            "!bg-[var(--color-button-bg)] !rounded-[var(--radius-md)] !shadow-[var(--shadow-inset-sm),0_0_0_0_transparent]",
          dropdownClass: "!rounded-[var(--radius-md)] !bg-[var(--color-button-bg)] !border-0",
        }),
      }),
  )
}

export function stage(id: string, title: string) {
  return pipe(
    {label: title} as { label: string },
    (n) => withId(n, id),
    withChildren,
    withShown,
    withIcon,
    withStageMeta,
    withMessaging,
  )
}

export type StageNode = ReturnType<typeof stage>

const stringValue = dropdownValue

export function text(id: string) {
  return withAutoProps(
    pipe(
      {} as {},
      (n) => withId(n, id),
      withChildren,
      withShown,
      withTooltip,
      withTitle,
      withMessaging,
      withRequired,
      withFix,
      withEnabled,
      withEditable,
      (n) => Object.assign(n, {_showTooltip: true, _imperativeSync: true}),
      (n) => withValue(n, stringValue),
      (n) =>
        withComponent(n, {
          component: markRaw(StyledInput),
          componentProps: () => ({class: "min-w-40 flex-1", autocomplete: "off"}),
        }),
      withExtraProps,
    ),
  )
}

export function markdown(id: string) {
  return withAutoProps(
    pipe(
      {} as {},
      (n) => withId(n, id),
      withChildren,
      withShown,
      withTooltip,
      withTitle,
      withMessaging,
      withRequired,
      withFix,
      withEnabled,
      withEditable,
      (n) => withValue(n, stringValue),
      (n) =>
        withComponent(n, {
          component: markRaw(MarkdownEditor),
          componentProps: (ctx) => ({
            maxHeight: 300,
            disabled: false,
            headingButtons: false,
            onImageUpload: ctx.onImageUpload,
          }),
        }),
      withExtraProps,
    ),
  )
}

function getSetValue(raw: NodeState): string[] {
  return raw instanceof Set ? Array.from(raw) : []
}

function setSetValue(_raw: NodeState, next: string[]): NodeState {
  return next.length > 0 ? new Set(next) : undefined
}

const setValue = {
  _getValue: getSetValue,
  _setValue: setSetValue,
  _isActive: (v: string[]) => v.length > 0,
}

const stringValueBehavior = {_getValue: getStringValue, _setValue: setStringValue, _isActive: (v: string) => v !== ""}

export function appComponent(id: string, rendererKey: string) {
  const node = pipe(
    {} as {},
    (n) => withId(n, id),
    withChildren,
    withShown,
    withTooltip,
    withTitle,
    withMessaging,
    withRequired,
    withFix,
    withEnabled,
    (n) => withValue(n, stringValueBehavior),
    (n) => withComponent(n, {rendererKey}),
    withExtraProps,
  )
  return Object.assign(node, {
    valueKind(this: any, kind: "boolean" | "string" | "set") {
      Object.assign(this, kind === "boolean" ? booleanValue : kind === "set" ? setValue : stringValueBehavior)
      return this
    },
    props(this: any, fn: (ctx: ComponentNodePropsContext) => Record<string, unknown>) {
      this._extraProps = fn
      return this
    },
  })
}
