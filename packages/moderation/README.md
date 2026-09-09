<!-- TODO: After checklist v1.5, move everything into src directory. -->

# @modrinth/moderation

This package contains both the moderation checklist system used by moderators for reviewing projects on Modrinth, and the publishing checklist (nag system) that provides automated feedback to project authors during the submission process.

## Structure

The package is organized as follows:

```
/packages/moderation/
├── data/
│   ├── checklist.ts        # Main moderation checklist definition - imports and exports all stages
│   ├── messages/           # Markdown files containing message templates for moderation
│   │   ├── title/          # Messages for the title stage
│   │   ├── description/    # Messages for the description stage
│   │   └── ...             # One directory per stage
│   ├── stages/             # Moderation stage definition files
│   │   ├── title.ts        # Title stage definition
│   │   ├── description.ts  # Description stage definition
│   │   └── ...             # One file per stage
│   └── nags/               # Publishing checklist (nag system) files
│       ├── index.ts        # Backend-kind registry and presentation adapters
│       ├── name.ts         # Name nag copy
│       └── ...             # One copy file per settings area
└── types/                  # Type definitions
    ├── actions.ts          # Action-related types (moderation)
    ├── messages.ts         # Message-related types (moderation)
    ├── stage.ts            # Stage-related types (moderation)
    └── nags.ts             # Nag-related types (publishing checklist)
```

## Moderation Checklist System

The moderation checklist provides a structured and transparent way to define moderation stages, actions, and messages that are displayed to moderators during the review process.

### Stages

A stage represents a discrete step in the moderation process, like checking a project's title, description, or links. Each stage has:

- A title displayed to moderators
- A link to guidance documentation
- An optional navigation path to direct moderators to the relevant part of the project page
- A list of actions that moderators can take

Stages are defined in individual files in the `data/stages` directory and are assembled into the complete checklist in `data/checklist.ts`.

### Actions

Actions represent decisions moderators can make for each stage. They can be buttons, dropdowns, toggles, etc. Actions can have:

- Labels displayed to the moderator
- Messages that are included in the final moderation decision
- Suggested moderation status and severity
- Optional text inputs for additional information
- Conditional behavior based on other selected actions

Each action requires a unique `id` field that is used for conditional logic and action relationships. The `suggestedStatus` and `severity` fields help determine the overall moderation outcome.

### Messages

Messages are the actual text that will be included in communications to project authors. To promote maintainability and reuse, messages are stored as Markdown files in the `data/messages` directory, organized by stage.

#### Variable replacement

You can use variables in your messages that will be replaced with user input:

1. Define a variable in the `relevantExtraInput` array of an action:

```typescript
relevantExtraInput: [
  {
    label: 'Explanation for the user',
    variable: 'MESSAGE',
    required: true,
  },
],
```

2. Use the variable in your message with `%VARIABLE%` syntax:

```markdown
# Your Message Title

Here is some explanation about the issue.

%MESSAGE%

More text after the variable.
```

The `%MESSAGE%` placeholder will be replaced with the text entered by the moderator.

### Conditional logic

The moderation system supports conditional behavior that changes based on the selection of other actions.

#### Conditional messages

You can define different messages for an action based on other selected actions:

```typescript
{
  id: 'my_action',
  type: 'button',
  label: 'My Action',
  weight: 100,
  message: async () => (await import('../messages/default-message.md?raw')).default,
  conditionalMessages: [
    {
      conditions: {
        requiredActions: ['other_action_id'],
        excludedActions: ['another_action_id']
      },
      message: async () => (await import('../messages/conditional-message.md?raw')).default,
    }
  ]
}
```

#### Enabling and disabling actions

Actions can enable or disable other actions when selected:

```typescript
{
  id: 'parent_action',
  type: 'button',
  label: 'Parent Action',
  // This will show these actions when parent_action is selected
  enablesActions: [
    {
      id: 'child_action',
      type: 'button',
      label: 'Child Action',
      // ...other properties
    }
  ],
  // This will hide actions with these IDs when parent_action is selected
  disablesActions: ['incompatible_action_id']
}
```

#### Conditional text inputs

Text inputs can be conditionally shown based on selected actions:

```typescript
relevantExtraInput: [
	{
		label: 'Additional Information',
		variable: 'INFO',
		showWhen: {
			requiredActions: ['specific_action_id'],
			excludedActions: ['incompatible_action_id'],
		},
	},
]
```

## Publishing Checklist (Nag System)

The nag system presents automated feedback returned by Labrinth's project validation endpoint. Labrinth decides which nags apply and their severity; this package owns localized copy and navigation metadata.

### Nags

A nag represents a specific issue or suggestion for improvement. Each nag has:

- A unique `id` for identification
- A `title` and `description` displayed to the user
- A `status` indicating severity: `'required'`, `'warning'`, or `'suggestion'`
- An optional `link` to help users address the issue

### Internationalization

Use vintl's `defineMessage` syntax.

### Adding New Nags

To add a new nag:

1. Add the nag copy to the appropriate area file in `data/nags/` and register its backend kind in `data/nags/index.ts`.
2. Assign the settings destination for the affected field.
3. Run `pnpm run fix` to fix lint issues and generate the root locale index.json file.

Example:

```typescript
// In description.ts
{
  id: 'new-nag',
  title: messages.newNagTitle,
  description: messages.newNagDescription,
	destination: 'description',
}
```

```typescript
// In description.i18n.ts
newNagTitle: {
  id: 'nags.new-nag.title',
  defaultMessage: 'New Nag Title',
},
newNagDescription: {
  id: 'nags.new-nag.description',
  defaultMessage: 'Description of the new nag issue.',
```
