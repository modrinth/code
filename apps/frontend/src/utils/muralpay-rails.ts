export type FieldType = 'text' | 'select' | 'email' | 'tel' | 'date'

export interface FieldConfig {
	name: string
	type: FieldType
	label: string
	required: boolean
	placeholder?: string
	helpText?: string
	options?: Array<{ value: string; label: string }>
	pattern?: string
	validate?: (value: string) => string | null // Returns error message or null if valid
}

export interface RailConfig {
	id: string
	name: string
	currency: string
	type: 'fiat' | 'crypto'
	railCode?: string // For fiat: 'usd', 'eur', 'cop', etc.
	blockchain?: string // For crypto: 'POLYGON', 'BASE', 'ETHEREUM', 'CELO'
	fields: FieldConfig[]
	warningMessage?: string
	requiresBankName?: boolean // Whether to show bank name field (most fiat rails need this)
}

const DOCUMENT_TYPE_OPTIONS = [
	{ value: 'NATIONAL_ID', label: 'National ID' },
	{ value: 'PASSPORT', label: 'Passport' },
	{ value: 'RESIDENT_ID', label: 'Resident ID' },
	{ value: 'RUC', label: 'RUC' },
	{ value: 'TAX_ID', label: 'Tax ID' },
]

const ACCOUNT_TYPE_OPTIONS = [
	{ value: 'CHECKING', label: 'Checking' },
	{ value: 'SAVINGS', label: 'Savings' },
]

export const MURALPAY_RAILS: Record<string, RailConfig> = {
	fiat_usd: {
		id: 'muralpay_usd',
		name: 'Bank Transfer (USD)',
		currency: 'USD',
		type: 'fiat',
		railCode: 'usd',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
			{
				name: 'bankRoutingNumber',
				type: 'text',
				label: 'Routing Number',
				required: true,
				placeholder: 'Enter 9-digit routing number',
				pattern: '^[0-9]{9}$',
				validate: (val) => (/^\d{9}$/.test(val) ? null : 'Must be exactly 9 digits'),
			},
		],
	},

	fiat_eur: {
		id: 'muralpay_eur',
		name: 'Bank Transfer (EUR)',
		currency: 'EUR',
		type: 'fiat',
		railCode: 'eur',
		requiresBankName: true,
		fields: [
			{
				name: 'iban',
				type: 'text',
				label: 'IBAN',
				required: true,
				placeholder: 'Enter IBAN',
				helpText: 'International Bank Account Number',
			},
			{
				name: 'swiftBic',
				type: 'text',
				label: 'SWIFT/BIC',
				required: true,
				placeholder: 'Enter SWIFT/BIC code',
				helpText: 'Bank Identifier Code',
			},
			{
				name: 'country',
				type: 'select',
				label: 'Country',
				required: true,
				options: [
					{ value: 'AT', label: 'Austria' },
					{ value: 'BE', label: 'Belgium' },
					{ value: 'CY', label: 'Cyprus' },
					{ value: 'EE', label: 'Estonia' },
					{ value: 'FI', label: 'Finland' },
					{ value: 'FR', label: 'France' },
					{ value: 'DE', label: 'Germany' },
					{ value: 'GR', label: 'Greece' },
					{ value: 'IE', label: 'Ireland' },
					{ value: 'IT', label: 'Italy' },
					{ value: 'LV', label: 'Latvia' },
					{ value: 'LT', label: 'Lithuania' },
					{ value: 'LU', label: 'Luxembourg' },
					{ value: 'MT', label: 'Malta' },
					{ value: 'NL', label: 'Netherlands' },
					{ value: 'PT', label: 'Portugal' },
					{ value: 'SK', label: 'Slovakia' },
					{ value: 'ES', label: 'Spain' },
				],
			},
		],
	},

	fiat_mxn: {
		id: 'muralpay_mxn',
		name: 'Bank Transfer (MXN)',
		currency: 'MXN',
		type: 'fiat',
		railCode: 'mxn',
		requiresBankName: true,
		fields: [
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'CLABE',
				required: true,
				placeholder: 'Enter 18-digit CLABE',
				helpText: 'Clave Bancaria Estandarizada (Mexican bank account number)',
				pattern: '^[0-9]{18}$',
				validate: (val) => (/^\d{18}$/.test(val) ? null : 'CLABE must be exactly 18 digits'),
			},
		],
	},

	fiat_brl: {
		id: 'muralpay_brl',
		name: 'PIX Transfer (BRL)',
		currency: 'BRL',
		type: 'fiat',
		railCode: 'brl',
		requiresBankName: true,
		fields: [
			{
				name: 'pixAccountType',
				type: 'select',
				label: 'PIX Key Type',
				required: true,
				options: [
					{ value: 'PHONE', label: 'Phone Number' },
					{ value: 'EMAIL', label: 'Email' },
					{ value: 'DOCUMENT', label: 'CPF/CNPJ' },
					{ value: 'BANK_ACCOUNT', label: 'Bank Account' },
				],
			},
			{
				name: 'pixEmail',
				type: 'email',
				label: 'PIX Email',
				required: false,
				placeholder: 'Enter PIX email',
			},
			{
				name: 'pixPhone',
				type: 'tel',
				label: 'PIX Phone',
				required: false,
				placeholder: '+55...',
			},
			{
				name: 'branchCode',
				type: 'text',
				label: 'Branch Code',
				required: false,
				placeholder: 'Enter branch code',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'CPF/CNPJ',
				required: true,
				placeholder: 'Enter CPF or CNPJ',
				helpText: 'Brazilian tax identification number',
			},
		],
	},

	fiat_cop: {
		id: 'muralpay_cop',
		name: 'Bank Transfer (COP)',
		currency: 'COP',
		type: 'fiat',
		railCode: 'cop',
		requiresBankName: true,
		fields: [
			{
				name: 'phoneNumber',
				type: 'tel',
				label: 'Phone Number',
				required: true,
				placeholder: '+57...',
			},
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter document number',
			},
		],
	},

	fiat_ars: {
		id: 'muralpay_ars',
		name: 'Bank Transfer (ARS)',
		currency: 'ARS',
		type: 'fiat',
		railCode: 'ars',
		requiresBankName: true,
		fields: [
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number (CBU/CVU)',
				required: true,
				placeholder: 'Enter CBU or CVU',
				helpText: 'Clave Bancaria Uniforme or Clave Virtual Uniforme',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'CUIT/CUIL',
				required: true,
				placeholder: 'Enter CUIT or CUIL',
				helpText: 'Argentine tax ID',
			},
			{
				name: 'bankAccountNumberType',
				type: 'text',
				label: 'Account Number Type',
				required: true,
				placeholder: 'CBU or CVU',
			},
		],
	},

	fiat_clp: {
		id: 'muralpay_clp',
		name: 'Bank Transfer (CLP)',
		currency: 'CLP',
		type: 'fiat',
		railCode: 'clp',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter RUT or other ID',
			},
		],
	},

	fiat_crc: {
		id: 'muralpay_crc',
		name: 'Bank Transfer (CRC)',
		currency: 'CRC',
		type: 'fiat',
		railCode: 'crc',
		requiresBankName: true,
		fields: [
			{
				name: 'iban',
				type: 'text',
				label: 'IBAN',
				required: true,
				placeholder: 'Enter Costa Rican IBAN',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter cédula or ID number',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
		],
	},

	fiat_pen: {
		id: 'muralpay_pen',
		name: 'Bank Transfer (PEN)',
		currency: 'PEN',
		type: 'fiat',
		railCode: 'pen',
		requiresBankName: true,
		fields: [
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter DNI or other ID',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number (CCI)',
				required: true,
				placeholder: 'Enter 20-digit CCI',
				helpText: 'Código de Cuenta Interbancaria',
			},
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
		],
	},

	fiat_bob: {
		id: 'muralpay_bob',
		name: 'Bank Transfer (BOB)',
		currency: 'BOB',
		type: 'fiat',
		railCode: 'bob',
		requiresBankName: true,
		fields: [
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter CI or other ID',
				helpText: 'Bolivian identity document',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
		],
	},

	fiat_zar: {
		id: 'muralpay_zar',
		name: 'Bank Transfer (ZAR)',
		currency: 'ZAR',
		type: 'fiat',
		railCode: 'zar',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
		],
	},

	'fiat_usd-peru': {
		id: 'muralpay_usd_peru',
		name: 'Bank Transfer (USD - Peru)',
		currency: 'USD',
		type: 'fiat',
		railCode: 'usd-peru',
		requiresBankName: true,
		fields: [
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter DNI or other ID',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
		],
	},

	'fiat_usd-china': {
		id: 'muralpay_usd_china',
		name: 'Bank Transfer (USD - China)',
		currency: 'USD',
		type: 'fiat',
		railCode: 'usd-china',
		requiresBankName: false,
		fields: [
			{
				name: 'bankName',
				type: 'text',
				label: 'Bank Name',
				required: true,
				placeholder: 'Enter bank name',
			},
			{
				name: 'accountType',
				type: 'select',
				label: 'Account Type',
				required: true,
				options: ACCOUNT_TYPE_OPTIONS,
			},
			{
				name: 'bankAccountNumber',
				type: 'text',
				label: 'Account Number',
				required: true,
				placeholder: 'Enter account number',
			},
			{
				name: 'documentNumber',
				type: 'text',
				label: 'Document Number',
				required: true,
				placeholder: 'Enter ID number',
			},
			{
				name: 'documentType',
				type: 'select',
				label: 'Document Type',
				required: true,
				options: DOCUMENT_TYPE_OPTIONS,
			},
			{
				name: 'phoneNumber',
				type: 'tel',
				label: 'Phone Number',
				required: true,
				placeholder: '+86...',
			},
			{
				name: 'address',
				type: 'text',
				label: 'Address',
				required: true,
				placeholder: 'Enter address',
			},
			{
				name: 'swiftBic',
				type: 'text',
				label: 'SWIFT/BIC',
				required: true,
				placeholder: 'Enter SWIFT/BIC code',
			},
		],
	},

	'blockchain_usdc_polygon': {
		id: 'muralpay_polygon_usdc',
		name: 'USDC (Polygon)',
		currency: 'USDC',
		type: 'crypto',
		blockchain: 'POLYGON',
		warningMessage:
			'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: 'Wallet Address',
				required: true,
				placeholder: '0x...',
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
			},
		],
	},

	'blockchain_usdc_base': {
		id: 'muralpay_base_usdc',
		name: 'USDC (Base)',
		currency: 'USDC',
		type: 'crypto',
		blockchain: 'BASE',
		warningMessage:
			'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: 'Wallet Address',
				required: true,
				placeholder: '0x...',
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
			},
		],
	},

	'blockchain_usdc_ethereum': {
		id: 'muralpay_ethereum_usdc',
		name: 'USDC (Ethereum)',
		currency: 'USDC',
		type: 'crypto',
		blockchain: 'ETHEREUM',
		warningMessage:
			'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: 'Wallet Address',
				required: true,
				placeholder: '0x...',
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
			},
		],
	},

	'blockchain_usdc_cello': {
		id: 'muralpay_celo_usdc',
		name: 'USDC (Celo)',
		currency: 'USDC',
		type: 'crypto',
		blockchain: 'CELO',
		warningMessage:
			'Double-check your wallet address. Funds sent to an incorrect address cannot be recovered.',
		fields: [
			{
				name: 'walletAddress',
				type: 'text',
				label: 'Wallet Address',
				required: true,
				placeholder: '0x...',
				pattern: '^0x[a-fA-F0-9]{40}$',
				validate: (val) =>
					/^0x[a-fA-F0-9]{40}$/.test(val) ? null : 'Must be a valid Ethereum address (0x...)',
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
