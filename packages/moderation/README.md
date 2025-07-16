# @modrinth/moderation

This package contains the moderation checklist system used for reviewing projects on Modrinth. It provides a structured and transparent way to define moderation stages, actions, and messages that are displayed to moderators during the review process.

## Structure

The package is organized as follows:

```
/packages/moderation/
├── data/
│   ├── checklist.ts        # Main checklist definition - imports and exports all stages
│   ├── messages/           # Markdown files containing message templates
│   │   ├── title/          # Messages for the title stage
│   │   ├── description/    # Messages for the description stage
│   │   └── ...             # One directory per stage
│   └── stages/             # Stage definition files
│       ├── title.ts        # Title stage definition
│       ├── description.ts  # Description stage definition
│       └── ...             # One file per stage
└── types/                  # Type definitions
    ├── actions.ts          # Action-related types
    ├── messages.ts         # Message-related types
    └── stage.ts            # Stage-related types
```

## Stages

A stage represents a discrete step in the moderation process, like checking a project's title, description, or links. Each stage has:

- A title displayed to moderators
- A link to guidance documentation
- An optional navigation path to direct moderators to the relevant part of the project page
- A list of actions that moderators can take

Stages are defined in individual files in the `data/stages` directory and are assembled into the complete checklist in `data/checklist.ts`.

## Actions

Actions represent decisions moderators can make for each stage. They can be buttons, dropdowns, toggles, etc. Actions can have:

- Labels displayed to the moderator
- Messages that are included in the final moderation decision
- Suggested moderation status and severity
- Optional text inputs for additional information
- Conditional behavior based on other selected actions

Each action requires a unique `id` field that is used for conditional logic and action relationships. The `suggestedStatus` and `severity` fields help determine the overall moderation outcome.

## Messages

Messages are the actual text that will be included in communications to project authors. To promote maintainability and reuse, messages are stored as Markdown files in the `data/messages` directory, organized by stage.

### Variable replacement

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

## Conditional logic

The moderation system supports conditional behavior that changes based on the selection of other actions.

### Conditional messages

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

### Enabling and disabling actions

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

### Conditional text inputs

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
