import type { Stage } from '../types/stage'
import type {
  Action,
  ButtonAction,
  ToggleAction,
  DropdownAction,
  MultiSelectChipsAction,
  ConditionalButtonAction,
  ConditionalMessage,
  AdditionalTextInput,
} from '../types/actions'

export const testStage: Stage = {
  title: 'Comprehensive Test Stage',
  guidance_url: 'https://docs.modrinth.com/moderation/test-stage',
  navigate: '/settings#general',
  shouldShow: () => true,
  actions: [
    // Basic button action
    {
      type: 'button',
      id: 'basic-button',
      label: 'Flag for Review',
      weight: 100,
      message: async () => 'This project has been flagged for additional review.',
      suggestedStatus: 'flagged',
      severity: 'medium',
      enablesActions: [
        {
          type: 'button',
          id: 'escalate-button',
          label: 'Escalate to Senior Moderator',
          weight: 110,
          message: async () => 'Escalated to senior moderation team for review.',
          severity: 'high',
        } as ButtonAction,
      ],
    } as ButtonAction,

    // Toggle action with description
    {
      type: 'toggle',
      id: 'contains-ads',
      label: 'Contains Advertisements',
      description: 'Check if the project contains any form of advertising or promotional content',
      defaultChecked: false,
      weight: 200,
      message: async () => 'Project contains advertising content that needs review.',
      conditionalMessages: [
        {
          weight: 210,
          message: async () => 'Project contains excessive advertising that violates guidelines.',
          conditions: {
            requiredActions: ['monetization-excessive'],
          },
        } as ConditionalMessage,
      ],
      enablesActions: [
        {
          type: 'multi-select-chips',
          id: 'ad-types',
          label: 'Types of Advertisements Found',
          options: [
            {
              label: 'Banner Ads',
              weight: 221,
              message: async () => '- Contains banner advertisements',
            },
            {
              label: 'Video Ads',
              weight: 222,
              message: async () => '- Contains video advertisements',
            },
            {
              label: 'Affiliate Links',
              weight: 223,
              message: async () => '- Contains affiliate links',
            },
            {
              label: 'Sponsored Content',
              weight: 224,
              message: async () => '- Contains sponsored content',
            },
          ],
        } as MultiSelectChipsAction,
      ],
    } as ToggleAction,

    // Dropdown action with multiple options
    {
      type: 'dropdown',
      id: 'content-quality',
      label: 'Overall Content Quality',
      defaultOption: 2,
      options: [
        {
          label: 'Excellent',
          weight: 301,
          message: async () => 'Content quality is excellent and well-documented.',
        },
        {
          label: 'Good',
          weight: 302,
          message: async () => 'Content quality is good with minor improvements needed.',
        },
        {
          label: 'Acceptable',
          weight: 303,
          message: async () => 'Content quality is acceptable but could be improved.',
        },
        {
          label: 'Poor',
          weight: 304,
          message: async () => 'Content quality is poor and needs significant improvement.',
        },
        {
          label: 'Unacceptable',
          weight: 305,
          message: async () => 'Content quality is unacceptable and violates standards.',
        },
      ],
      relevantExtraInput: [
        {
          label: 'Specific Quality Issues',
          placeholder: 'Describe the quality issues found...',
          required: true,
          variable: 'QUALITY_ISSUES',
          showWhen: {
            // Only show when Poor or Unacceptable is selected (indices 3 or 4)
            // This would need custom logic in the component to check dropdown values
          },
        } as AdditionalTextInput,
      ],
    } as DropdownAction,

    // Conditional button that changes based on other selections
    {
      type: 'conditional-button',
      id: 'monetization-check',
      label: 'Check Monetization Compliance',
      messageVariants: [
        {
          weight: 400,
          message: async () => 'Monetization appears compliant with guidelines.',
          conditions: {
            excludedActions: ['contains-ads'],
          },
        } as ConditionalMessage,
        {
          weight: 401,
          message: async () => 'Monetization needs review due to advertising content.',
          conditions: {
            requiredActions: ['contains-ads'],
            excludedActions: ['monetization-excessive'],
          },
        } as ConditionalMessage,
        {
          weight: 402,
          message: async () => 'CRITICAL: Excessive monetization violates platform guidelines!',
          conditions: {
            requiredActions: ['contains-ads', 'monetization-excessive'],
          },
          fallbackMessage: async () => 'Monetization status needs review.',
        } as ConditionalMessage,
      ],
      enablesActions: [
        {
          type: 'toggle',
          id: 'monetization-excessive',
          label: 'Excessive Monetization',
          description: 'Monetization significantly impacts user experience',
          weight: 410,
          message: async () => 'Project has excessive monetization.',
          suggestedStatus: 'rejected',
          severity: 'high',
        } as ToggleAction,
      ],
    } as ConditionalButtonAction,

    // Multi-select chips with complex interactions
    {
      type: 'multi-select-chips',
      id: 'compliance-issues',
      label: 'Compliance Issues Found',
      options: [
        {
          label: 'Missing License',
          weight: 501,
          message: async () => '- Missing required license information',
        },
        {
          label: 'Copyright Violation',
          weight: 502,
          message: async () => '- Potential copyright violation detected',
        },
        {
          label: 'Inappropriate Content',
          weight: 503,
          message: async () => '- Contains inappropriate content',
        },
        {
          label: 'Security Risk',
          weight: 504,
          message: async () => '- Potential security risk identified',
        },
      ],
      relevantExtraInput: [
        {
          label: 'Copyright Details',
          placeholder: 'Describe the copyright violation...',
          required: true,
          variable: 'COPYRIGHT_DETAILS',
          showWhen: {
            requiredActions: ['copyright-selected'], // Would need custom logic for chip selection
          },
        } as AdditionalTextInput,
        {
          label: 'Security Risk Description',
          placeholder: 'Describe the security risk...',
          required: true,
          variable: 'SECURITY_DETAILS',
          showWhen: {
            requiredActions: ['security-selected'], // Would need custom logic for chip selection
          },
        } as AdditionalTextInput,
      ],
    } as MultiSelectChipsAction,

    // Button that disables other actions
    {
      type: 'button',
      id: 'auto-approve',
      label: 'Auto-Approve (No Issues Found)',
      weight: 600,
      message: async () => 'Project meets all guidelines and is approved.',
      suggestedStatus: 'approved',
      severity: 'low',
      disablesActions: [
        'basic-button',
        'escalate-button',
        'compliance-issues',
        'monetization-check',
        'monetization-excessive',
      ],
      relevantExtraInput: [
        {
          label: 'Approval Notes',
          placeholder: 'Add any additional notes for approval...',
          required: false,
          variable: 'APPROVAL_NOTES',
        } as AdditionalTextInput,
      ],
    } as ButtonAction,

    // Complex button with multiple conditional messages and inputs
    {
      type: 'button',
      id: 'request-changes',
      label: 'Request Changes from Author',
      weight: 700,
      message: async () => {
        // Load from markdown file
        return (await import('./messages/rule-breaking.md?raw')).default
      },
      conditionalMessages: [
        {
          weight: 710,
          message: async () => (await import('./messages/rule-breaking.md?raw')).default.default,
          conditions: {
            requiredActions: ['monetization-excessive'],
          },
        } as ConditionalMessage,
        {
          weight: 720,
          message: async () => 'Minor changes requested:\n%CHANGE_DETAILS%',
          conditions: {
            excludedActions: ['monetization-excessive', 'compliance-issues'],
          },
        } as ConditionalMessage,
      ],
      relevantExtraInput: [
        {
          label: 'Changes Required',
          placeholder: 'List the specific changes needed...',
          required: true,
          variable: 'CHANGE_DETAILS',
        } as AdditionalTextInput,
        {
          label: 'Deadline for Changes',
          placeholder: 'e.g., 7 days',
          required: false,
          variable: 'DEADLINE',
          showWhen: {
            requiredActions: ['monetization-excessive'],
          },
        } as AdditionalTextInput,
        {
          label: 'Contact Method',
          placeholder: 'How should we contact the author?',
          required: false,
          variable: 'CONTACT_METHOD',
          showWhen: {
            excludedActions: ['auto-approve'],
          },
        } as AdditionalTextInput,
      ],
    } as ButtonAction,
  ] as Action[],
}
