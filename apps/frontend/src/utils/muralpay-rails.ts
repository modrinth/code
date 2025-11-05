import { defineMessage, type MessageDescriptor } from '@vintl/vintl'

export type FieldType = 'text' | 'select' | 'email' | 'tel' | 'date'

export interface FieldConfig {
	name: string
	type: FieldType
	label: MessageDescriptor
	required: boolean
	placeholder?: MessageDescriptor
	helpText?: MessageDescriptor
	options?: Array<{ value: string; label: MessageDescriptor }>
	pattern?: string
	validate?: (value: string) => string | null
	autocomplete?: string
}

export interface RailConfig {
	id: string
	name: MessageDescriptor
	currency: string
	fee: string
	type: 'fiat' | 'crypto'
	railCode?: string
	blockchain?: string
	fields: FieldConfig[]
	warningMessage?: MessageDescriptor
	requiresBankName?: boolean
}

const DOCUMENT_TYPE_OPTIONS = [
	{
		value: 'NATIONAL_ID',
		label: defineMessage({
			id: 'muralpay.document-type.national-id',
			defaultMessage: 'National ID',
		}),
	},
	{
		value: 'PASSPORT',
		label: defineMessage({ id: 'muralpay.document-type.passport', defaultMessage: 'Passport' }),
	},
	{
		value: 'RESIDENT_ID',
		label: defineMessage({
			id: 'muralpay.document-type.resident-id',
			defaultMessage: 'Resident ID',
		}),
	},
	{
		value: 'RUC',
		label: defineMessage({ id: 'muralpay.document-type.ruc', defaultMessage: 'RUC' }),
	},
	{
		value: 'TAX_ID',
		label: defineMessage({ id: 'muralpay.document-type.tax-id', defaultMessage: 'Tax ID' }),
	},
]

const ACCOUNT_TYPE_OPTIONS = [
	{
		value: 'CHECKING',
		label: defineMessage({ id: 'muralpay.account-type.checking', defaultMessage: 'Checking' }),
	},
	{
		value: 'SAVINGS',
		label: defineMessage({ id: 'muralpay.account-type.savings', defaultMessage: 'Savings' }),
	},
]

export const MURALPAY_RAILS: Record<string, RailConfig> = {
	fiat_usd: {
		id: 'fiat_usd',
		name: defineMessage({
			id: 'muralpay.rail.fiat-usd.name',
			defaultMessage: 'Bank Transfer (USD)',
		}),
		currency: 'USD',
		type: 'fiat',
		fee: '≈ 1.50% + $0.50',
		railCode: 'usd',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.bank-account-number',
					defaultMessage: 'Account number',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-account-number',
					defaultMessage: 'Enter account number',
				}),
				autocomplete: 'off',
			},
			{
				name: 'bankRoutingNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.routing-number',
					defaultMessage: 'Routing number',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-routing-number',
					defaultMessage: 'Enter 9-digit routing number',
				}),
				pattern: '^[0-9]{9}$',
				validate: (val) => (/^\d{9}$/.test(val) ? null : 'Must be exactly 9 digits'),
				autocomplete: 'off',
			},
		],
	},

	fiat_eur: {
		id: 'fiat_eur',
		name: defineMessage({
			id: 'muralpay.rail.fiat-eur.name',
			defaultMessage: 'Bank Transfer (EUR)',
		}),
		currency: 'EUR',
		type: 'fiat',
		fee: '≈ 1.60% + €1.00',
		railCode: 'eur',
		requiresBankName: true,
		fields: [
			{
				name: 'iban',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.iban', defaultMessage: 'IBAN' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-iban',
					defaultMessage: 'Enter IBAN',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.iban',
					defaultMessage: 'International Bank Account Number',
				}),
				autocomplete: 'iban',
			},
			{
				name: 'swiftBic',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.swift-bic', defaultMessage: 'SWIFT/BIC' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-swift-bic',
					defaultMessage: 'Enter SWIFT/BIC code',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.swift-bic',
					defaultMessage: 'Bank Identifier Code',
				}),
				autocomplete: 'swift',
			},
			{
				name: 'country',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.country', defaultMessage: 'Country' }),
				required: true,
				autocomplete: 'off',
				options: [
					{
						value: 'AT',
						label: defineMessage({ id: 'muralpay.country.at', defaultMessage: 'Austria' }),
					},
					{
						value: 'BE',
						label: defineMessage({ id: 'muralpay.country.be', defaultMessage: 'Belgium' }),
					},
					{
						value: 'CY',
						label: defineMessage({ id: 'muralpay.country.cy', defaultMessage: 'Cyprus' }),
					},
					{
						value: 'EE',
						label: defineMessage({ id: 'muralpay.country.ee', defaultMessage: 'Estonia' }),
					},
					{
						value: 'FI',
						label: defineMessage({ id: 'muralpay.country.fi', defaultMessage: 'Finland' }),
					},
					{
						value: 'FR',
						label: defineMessage({ id: 'muralpay.country.fr', defaultMessage: 'France' }),
					},
					{
						value: 'DE',
						label: defineMessage({ id: 'muralpay.country.de', defaultMessage: 'Germany' }),
					},
					{
						value: 'GR',
						label: defineMessage({ id: 'muralpay.country.gr', defaultMessage: 'Greece' }),
					},
					{
						value: 'IE',
						label: defineMessage({ id: 'muralpay.country.ie', defaultMessage: 'Ireland' }),
					},
					{
						value: 'IT',
						label: defineMessage({ id: 'muralpay.country.it', defaultMessage: 'Italy' }),
					},
					{
						value: 'LV',
						label: defineMessage({ id: 'muralpay.country.lv', defaultMessage: 'Latvia' }),
					},
					{
						value: 'LT',
						label: defineMessage({ id: 'muralpay.country.lt', defaultMessage: 'Lithuania' }),
					},
					{
						value: 'LU',
						label: defineMessage({ id: 'muralpay.country.lu', defaultMessage: 'Luxembourg' }),
					},
					{
						value: 'MT',
						label: defineMessage({ id: 'muralpay.country.mt', defaultMessage: 'Malta' }),
					},
					{
						value: 'NL',
						label: defineMessage({ id: 'muralpay.country.nl', defaultMessage: 'Netherlands' }),
					},
					{
						value: 'PT',
						label: defineMessage({ id: 'muralpay.country.pt', defaultMessage: 'Portugal' }),
					},
					{
						value: 'SK',
						label: defineMessage({ id: 'muralpay.country.sk', defaultMessage: 'Slovakia' }),
					},
					{
						value: 'ES',
						label: defineMessage({ id: 'muralpay.country.es', defaultMessage: 'Spain' }),
					},
				],
			},
		],
	},

	fiat_mxn: {
		id: 'fiat_mxn',
		name: defineMessage({
			id: 'muralpay.rail.fiat-mxn.name',
			defaultMessage: 'Bank Transfer (MXN)',
		}),
		currency: 'MXN',
		type: 'fiat',
		fee: '≈ 1.90% + $0.50',
		railCode: 'mxn',
		requiresBankName: true,
		fields: [
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.clabe', defaultMessage: 'CLABE' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-clabe',
					defaultMessage: 'Enter 18-digit CLABE',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.clabe',
					defaultMessage: 'Clave Bancaria Estandarizada (Mexican bank account number)',
				}),
				pattern: '^[0-9]{18}$',
				validate: (val) => (/^\d{18}$/.test(val) ? null : 'CLABE must be exactly 18 digits'),
				autocomplete: 'off',
			},
		],
	},

	fiat_brl: {
		id: 'fiat_brl',
		name: defineMessage({
			id: 'muralpay.rail.fiat-brl.name',
			defaultMessage: 'PIX Transfer (BRL)',
		}),
		currency: 'BRL',
		type: 'fiat',
		fee: '≈ 2.30% + $0.25',
		railCode: 'brl',
		requiresBankName: true,
		fields: [
			{
				name: 'pixAccountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.pix-key-type', defaultMessage: 'PIX key type' }),
				required: true,
				autocomplete: 'off',
				options: [
					{
						value: 'PHONE',
						label: defineMessage({ id: 'muralpay.pix-type.phone', defaultMessage: 'Phone number' }),
					},
					{
						value: 'EMAIL',
						label: defineMessage({ id: 'muralpay.pix-type.email', defaultMessage: 'Email' }),
					},
					{
						value: 'DOCUMENT',
						label: defineMessage({ id: 'muralpay.pix-type.document', defaultMessage: 'CPF/CNPJ' }),
					},
					{
						value: 'BANK_ACCOUNT',
						label: defineMessage({
							id: 'muralpay.pix-type.bank-account',
							defaultMessage: 'Bank account',
						}),
					},
				],
			},
			{
				name: 'pixEmail',
				type: 'email',
				label: defineMessage({ id: 'muralpay.field.pix-email', defaultMessage: 'PIX email' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-pix-email',
					defaultMessage: 'Enter PIX email',
				}),
				autocomplete: 'email',
			},
			{
				name: 'pixPhone',
				type: 'tel',
				label: defineMessage({ id: 'muralpay.field.pix-phone', defaultMessage: 'PIX phone' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.pix-phone',
					defaultMessage: '+55...',
				}),
				autocomplete: 'tel',
			},
			{
				name: 'branchCode',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.branch-code', defaultMessage: 'Branch code' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-branch-code',
					defaultMessage: 'Enter branch code',
				}),
				autocomplete: 'off',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.cpf-cnpj', defaultMessage: 'CPF/CNPJ' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-cpf-cnpj',
					defaultMessage: 'Enter CPF or CNPJ',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.cpf-cnpj',
					defaultMessage: 'Brazilian tax identification number',
				}),
				autocomplete: 'off',
			},
		],
	},

	fiat_cop: {
		id: 'fiat_cop',
		name: defineMessage({
			id: 'muralpay.rail.fiat-cop.name',
			defaultMessage: 'Bank Transfer (COP)',
		}),
		currency: 'COP',
		type: 'fiat',
		fee: '≈ 1.95% + $0.35',
		railCode: 'cop',
		requiresBankName: true,
		fields: [
			{
				name: 'phoneNumber',
				type: 'tel',
				label: defineMessage({ id: 'muralpay.field.phone-number', defaultMessage: 'Phone number' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.phone-cop',
					defaultMessage: '+57...',
				}),
				autocomplete: 'tel',
			},
			{
				name: 'accountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.bank-account-number',
					defaultMessage: 'Account number',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.enter-account-number',
					defaultMessage: 'Enter account number',
				}),
				autocomplete: 'off',
			},
			{
				name: 'documentType',
				type: 'select',
				label: defineMessage({
					id: 'muralpay.field.document-type',
					defaultMessage: 'Document type',
				}),
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
		],
	},

	fiat_ars: {
		id: 'fiat_ars',
		name: defineMessage({
			id: 'muralpay.rail.fiat-ars.name',
			defaultMessage: 'Bank Transfer (ARS)',
		}),
		currency: 'ARS',
		type: 'fiat',
		fee: '≈ 1.50% + $0.00',
		railCode: 'ars',
		requiresBankName: true,
		fields: [
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.account-number-cbu-cvu',
					defaultMessage: 'Account number (CBU/CVU)',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.cbu-cvu',
					defaultMessage: 'Enter CBU or CVU',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.cbu-cvu',
					defaultMessage: 'Clave Bancaria Uniforme or Clave Virtual Uniforme',
				}),
				autocomplete: 'off',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.cuit-cuil', defaultMessage: 'CUIT/CUIL' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.cuit-cuil',
					defaultMessage: 'Enter CUIT or CUIL',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.cuit-cuil',
					defaultMessage: 'Argentine tax ID',
				}),
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumberType',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.account-number-type',
					defaultMessage: 'Account number type',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.cbu-cvu-type',
					defaultMessage: 'CBU or CVU',
				}),
				autocomplete: 'off',
			},
		],
	},

	fiat_clp: {
		id: 'fiat_clp',
		name: defineMessage({
			id: 'muralpay.rail.fiat-clp.name',
			defaultMessage: 'Bank Transfer (CLP)',
		}),
		currency: 'CLP',
		type: 'fiat',
		fee: '≈ 1.95% + $1.20',
		railCode: 'clp',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.account-number',
					defaultMessage: 'Account number',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.account-number',
					defaultMessage: 'Enter account number',
				}),
				autocomplete: 'off',
			},
			{
				name: 'documentType',
				type: 'select',
				label: defineMessage({
					id: 'muralpay.field.document-type',
					defaultMessage: 'Document type',
				}),
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
		],
	},

	fiat_crc: {
		id: 'fiat_crc',
		name: defineMessage({
			id: 'muralpay.rail.fiat-crc.name',
			defaultMessage: 'Bank Transfer (CRC)',
		}),
		currency: 'CRC',
		type: 'fiat',
		fee: '≈ 2.05% + $0.80',
		railCode: 'crc',
		requiresBankName: true,
		fields: [
			{
				name: 'iban',
				type: 'text',
				label: defineMessage({ id: 'muralpay.field.iban', defaultMessage: 'IBAN' }),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.iban-crc',
					defaultMessage: 'Enter Costa Rican IBAN',
				}),
				autocomplete: 'iban',
			},
			{
				name: 'documentType',
				type: 'select',
				label: defineMessage({
					id: 'muralpay.field.document-type',
					defaultMessage: 'Document type',
				}),
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
		],
	},

	fiat_pen: {
		id: 'fiat_pen',
		name: defineMessage({
			id: 'muralpay.rail.fiat-pen.name',
			defaultMessage: 'Bank Transfer (PEN)',
		}),
		currency: 'PEN',
		type: 'fiat',
		fee: '≈ 2.15% + $1.00',
		railCode: 'pen',
		requiresBankName: true,
		fields: [
			{
				name: 'documentType',
				type: 'select',
				label: defineMessage({
					id: 'muralpay.field.document-type',
					defaultMessage: 'Document type',
				}),
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.account-number-cci',
					defaultMessage: 'Account number (CCI)',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.cci',
					defaultMessage: 'Enter 20-digit CCI',
				}),
				helpText: defineMessage({
					id: 'muralpay.help.cci',
					defaultMessage: 'Código de Cuenta Interbancaria',
				}),
				autocomplete: 'off',
			},
			{
				name: 'accountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
		],
	},

	// fiat_bob: {
	// 	id: 'fiat_bob',
	// 	name: defineMessage({
	// 		id: 'muralpay.rail.fiat-bob.name',
	// 		defaultMessage: 'Bank Transfer (BOB)',
	// 	}),
	// 	currency: 'BOB',
	// 	type: 'fiat',
	// 	fee: 'TBD',
	// 	railCode: 'bob',
	// 	requiresBankName: true,
	// 	fields: [
	// 		{
	// 			name: 'bankAccountNumber',
	// 			type: 'text',
	// 			label: defineMessage({
	// 				id: 'muralpay.field.account-number',
	// 				defaultMessage: 'Account number',
	// 			}),
	// 			required: true,
	// 			placeholder: defineMessage({
	// 				id: 'muralpay.placeholder.account-number',
	// 				defaultMessage: 'Enter account number',
	// 			}),
	// 			autocomplete: 'off',
	// 		},
	// 		{
	// 			name: 'documentType',
	// 			type: 'select',
	// 			label: defineMessage({
	// 				id: 'muralpay.field.document-type',
	// 				defaultMessage: 'Document type',
	// 			}),
	// 			required: true,
	// 			options: DOCUMENT_TYPE_OPTIONS,
	// 			autocomplete: 'off',
	// 		},
	// 	],
	// },

	fiat_zar: {
		id: 'fiat_zar',
		name: defineMessage({
			id: 'muralpay.rail.fiat-zar.name',
			defaultMessage: 'Bank Transfer (ZAR)',
		}),
		currency: 'ZAR',
		type: 'fiat',
		fee: '≈ 2.40% + $1.50',
		railCode: 'zar',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.account-number',
					defaultMessage: 'Account number',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.account-number',
					defaultMessage: 'Enter account number',
				}),
				autocomplete: 'off',
			},
		],
	},

	'fiat_usd-peru': {
		id: 'fiat_usd-peru',
		name: defineMessage({
			id: 'muralpay.rail.fiat-usd-peru.name',
			defaultMessage: 'Bank Transfer (USD - Peru)',
		}),
		currency: 'USD',
		type: 'fiat',
		fee: '≈ 1.50% + $5.00',
		railCode: 'usd-peru',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.account-number',
					defaultMessage: 'Account number',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.account-number',
					defaultMessage: 'Enter account number',
				}),
				autocomplete: 'off',
			},
			{
				name: 'documentType',
				type: 'select',
				label: defineMessage({
					id: 'muralpay.field.document-type',
					defaultMessage: 'Document type',
				}),
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
				autocomplete: 'off',
			},
		],
	},

	// 'fiat_usd-china': {
	// 	id: 'fiat_usd-china',
	// 	name: defineMessage({
	// 		id: 'muralpay.rail.fiat-usd-china.name',
	// 		defaultMessage: 'Bank Transfer (USD - China)',
	// 	}),
	// 	currency: 'USD',
	// 	type: 'fiat',
	// 	fee: 'TBD',
	// 	railCode: 'usd-china',
	// 	requiresBankName: false,
	// 	fields: [
	// 		{
	// 			name: 'bankName',
	// 			type: 'text',
	// 			label: defineMessage({ id: 'muralpay.field.bank-name', defaultMessage: 'Bank Name' }),
	// 			required: true,
	// 			placeholder: defineMessage({
	// 				id: 'muralpay.placeholder.bank-name',
	// 				defaultMessage: 'Enter bank name',
	// 			}),
	// 			autocomplete: 'off',
	// 		},
	// 		{
	// 			name: 'accountType',
	// 			type: 'select',
	// 			label: defineMessage({ id: 'muralpay.field.account-type', defaultMessage: 'Account type' }),
	// 			required: true,
	// 			options: ACCOUNT_TYPE_OPTIONS,
	// 			autocomplete: 'off',
	// 		},
	// 		{
	// 			name: 'bankAccountNumber',
	// 			type: 'text',
	// 			label: defineMessage({
	// 				id: 'muralpay.field.account-number',
	// 				defaultMessage: 'Account number',
	// 			}),
	// 			required: true,
	// 			placeholder: defineMessage({
	// 				id: 'muralpay.placeholder.account-number',
	// 				defaultMessage: 'Enter account number',
	// 			}),
	// 			autocomplete: 'off',
	// 		},
	// 		{
	// 			name: 'documentType',
	// 			type: 'select',
	// 			label: defineMessage({
	// 				id: 'muralpay.field.document-type',
	// 				defaultMessage: 'Document type',
	// 			}),
	// 			required: true,
	// 			options: DOCUMENT_TYPE_OPTIONS,
	// 			autocomplete: 'off',
	// 		},
	// 		{
	// 			name: 'phoneNumber',
	// 			type: 'tel',
	// 			label: defineMessage({ id: 'muralpay.field.phone-number', defaultMessage: 'Phone number' }),
	// 			required: true,
	// 			placeholder: defineMessage({
	// 				id: 'muralpay.placeholder.phone-china',
	// 				defaultMessage: '+86...',
	// 			}),
	// 			autocomplete: 'tel',
	// 		},
	// 		{
	// 			name: 'address',
	// 			type: 'text',
	// 			label: defineMessage({ id: 'muralpay.field.address', defaultMessage: 'Address' }),
	// 			required: true,
	// 			placeholder: defineMessage({
	// 				id: 'muralpay.placeholder.address',
	// 				defaultMessage: 'Enter address',
	// 			}),
	// 			autocomplete: 'street-address',
	// 		},
	// 		{
	// 			name: 'swiftBic',
	// 			type: 'text',
	// 			label: defineMessage({ id: 'muralpay.field.swift-bic', defaultMessage: 'SWIFT/BIC' }),
	// 			required: true,
	// 			placeholder: defineMessage({
	// 				id: 'muralpay.placeholder.swift-bic',
	// 				defaultMessage: 'Enter SWIFT/BIC code',
	// 			}),
	// 			autocomplete: 'swift',
	// 		},
	// 	],
	// },

	blockchain_usdc_polygon: {
		id: 'blockchain_usdc_polygon',
		name: defineMessage({
			id: 'muralpay.rail.usdc-polygon.name',
			defaultMessage: 'Crypto (USDC)',
		}),
		currency: 'USDC',
		type: 'crypto',
		fee: '≈ 1%',
		blockchain: 'POLYGON',
		warningMessage: defineMessage({
			id: 'muralpay.warning.wallet-address',
			defaultMessage:
				'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		}),
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.wallet-address',
					defaultMessage: 'Wallet address',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.wallet-address-eth',
					defaultMessage: '0x...',
				}),
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
				autocomplete: 'off',
			},
		],
	},

	blockchain_usdc_base: {
		id: 'blockchain_usdc_base',
		name: defineMessage({ id: 'muralpay.rail.usdc-base.name', defaultMessage: 'USDC (Base)' }),
		currency: 'USDC',
		type: 'crypto',
		fee: '≈ 1%',
		blockchain: 'BASE',
		warningMessage: defineMessage({
			id: 'muralpay.warning.wallet-address',
			defaultMessage:
				'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		}),
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.wallet-address',
					defaultMessage: 'Wallet address',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.wallet-address-eth',
					defaultMessage: '0x...',
				}),
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
				autocomplete: 'off',
			},
		],
	},

	blockchain_usdc_ethereum: {
		id: 'blockchain_usdc_ethereum',
		name: defineMessage({
			id: 'muralpay.rail.usdc-ethereum.name',
			defaultMessage: 'USDC (Ethereum)',
		}),
		currency: 'USDC',
		type: 'crypto',
		fee: '≈ 1%',
		blockchain: 'ETHEREUM',
		warningMessage: defineMessage({
			id: 'muralpay.warning.wallet-address',
			defaultMessage:
				'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		}),
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.wallet-address',
					defaultMessage: 'Wallet address',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.wallet-address-eth',
					defaultMessage: '0x...',
				}),
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
				autocomplete: 'off',
			},
		],
	},

	blockchain_usdc_celo: {
		id: 'blockchain_usdc_celo',
		name: defineMessage({ id: 'muralpay.rail.usdc-celo.name', defaultMessage: 'USDC (Celo)' }),
		currency: 'USDC',
		type: 'crypto',
		fee: '≈ 1%',
		blockchain: 'CELO',
		warningMessage: defineMessage({
			id: 'muralpay.warning.wallet-address',
			defaultMessage:
				'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		}),
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: defineMessage({
					id: 'muralpay.field.wallet-address',
					defaultMessage: 'Wallet address',
				}),
				required: true,
				placeholder: defineMessage({
					id: 'muralpay.placeholder.wallet-address-eth',
					defaultMessage: '0x...',
				}),
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
				autocomplete: 'off',
			},
		],
	},
}

export function getAvailableRails(): string[] {
	return Object.keys(MURALPAY_RAILS)
}

export function getRailsByType(type: 'fiat' | 'crypto'): RailConfig[] {
	return Object.values(MURALPAY_RAILS).filter((rail) => rail.type === type)
}

export function getRailConfig(railId: string): RailConfig | undefined {
	return MURALPAY_RAILS[railId]
}
