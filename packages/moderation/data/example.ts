import type { Stage } from '../types/stage'

export default [
  {
    title: 'An example',
    guidance_url: 'https://notion.so/modrinth/moderation-guidelines#content-moderation',
    actions: [
      {
        id: 'content-violation',
        type: 'toggle',
        weight: 100,
        label: 'Content Violation Detected',
        description: 'Check this if the content violates community guidelines',
        defaultChecked: false,
        suggestedStatus: 'flagged',
        suggestedSeverity: 'medium',
        message: async () => await import('../messages/example.md?raw'),
        enablesActions: [
          {
            id: 'severity-level',
            type: 'dropdown',
            label: 'Violation Severity',
            weight: 200,
            options: [
              {
                label: 'Minor (Warning)',
                weight: 201,
                message: async () => await import('../messages/example.md?raw'),
              },
              {
                label: 'Severe (Immediate Action)',
                weight: 202,
                message: async () => await import('../messages/example.md?raw'),
              },
            ],
            defaultOption: 0,
            suggestedStatus: 'flagged',
            suggestedSeverity: 'medium',
          },
          {
            id: 'repeat-offender',
            type: 'toggle',
            weight: 300,
            label: 'Repeat Offender',
            description: 'This user has violated guidelines before',
            defaultChecked: false,
            suggestedSeverity: 'high',
            message: async () => 'Previous violations on record.',
          },
        ],
      },
      {
        id: 'conditional-message-button',
        type: 'conditional-button',
        label: 'Apply Moderation Action',
        weight: 400,
        suggestedStatus: 'rejected',
        messageVariants: [
          {
            weight: 401,
            conditions: {
              requiredActions: ['content-violation', 'repeat-offender'],
            },
            message: async () => await import('../messages/example.md?raw'),
          },
          {
            weight: 402,
            conditions: {
              requiredActions: ['content-violation'],
              excludedActions: ['repeat-offender'],
            },
            message: async () => await import('../messages/example.md?raw'),
          },
        ],
        relevantExtraInput: [
          {
            label: 'Additional Notes',
            placeholder: 'Provide specific details about the violation...',
            required: false,
            showWhen: {
              requiredActions: ['content-violation'],
            },
          },
          {
            label: 'Escalation Reason',
            placeholder: 'Explain why this requires escalation...',
            required: true,
            showWhen: {
              requiredActions: ['repeat-offender'],
            },
          },
        ],
      },
      {
        id: 'conditional-enhanced-button',
        type: 'button',
        weight: 500,
        label: 'Enhanced Moderation',
        suggestedStatus: 'rejected',
        suggestedSeverity: 'high',
        message: async () => 'Standard moderation message.',
        conditionalMessages: [
          {
            weight: 501,
            conditions: {
              requiredActions: ['content-violation', 'repeat-offender'],
            },
            message: async () => await import('../messages/example.md?raw'),
            fallbackMessage: async () => await import('../messages/example.md?raw'),
          },
          {
            weight: 502,
            conditions: {
              requiredActions: ['content-violation'],
            },
            message: async () => await import('../messages/example.md?raw'),
          },
        ],
      },
    ],
  },
  {
    title: 'Mandatory Cat Picture',
    guidance_url: 'https://notion.so/modrinth/moderation-guidelines#mandatory-cat-picture',
    actions: [
      {
        id: 'no-cat-picture',
        type: 'button',
        weight: 999,
        label: 'No Cat Picture',
        suggestedStatus: 'rejected',
        suggestedSeverity: 'critical',
        message: async () => await import('../messages/example.md?raw'),
      },
    ],
  },
] as ReadonlyArray<Stage>
