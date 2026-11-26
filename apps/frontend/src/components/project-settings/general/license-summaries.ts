/**
 * No i18n pass here.
 *
 * License summaries and rule descriptions adapted from github/choosealicense.com.
 * Source: https://github.com/github/choosealicense.com/tree/7ae29dd78100be78e3f334ab08a9ecb85f48d4e1
 * Imported from _licenses/*.txt and _data/rules.yml; full license texts are fetched separately.
 *
 * Copyright (c) GitHub, Inc. and contributors
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */
export interface LicenseSummary {
	description: string
	sourceUrl: string
	permissions: string[]
	limitations: string[]
	conditions: string[]
}

export type LicenseSummaryCategory = 'permissions' | 'limitations' | 'conditions'

export const licenseSummaryRules: Record<
	LicenseSummaryCategory,
	Record<string, { label: string; description: string }>
> = {
	permissions: {
		'commercial-use': {
			label: 'Commercial use',
			description: 'The licensed material and derivatives may be used for commercial purposes.',
		},
		modifications: {
			label: 'Modification',
			description: 'The licensed material may be modified.',
		},
		distribution: {
			label: 'Distribution',
			description: 'The licensed material may be distributed.',
		},
		'private-use': {
			label: 'Private use',
			description: 'The licensed material may be used and modified in private.',
		},
		'patent-use': {
			label: 'Patent use',
			description: 'This license provides an express grant of patent rights from contributors.',
		},
	},
	limitations: {
		copying: {
			label: 'Copying',
			description: 'No permission to copy the project is granted by this selection.',
		},
		modifications: {
			label: 'Modification',
			description:
				'No permission to modify the project or create derivative works is granted by this selection.',
		},
		distribution: {
			label: 'Distribution',
			description: 'No permission to redistribute the project is granted by this selection.',
		},
		'private-use': {
			label: 'Private use',
			description: 'No permission for private use or play is granted by this selection.',
		},
		'commercial-use': {
			label: 'Commercial use',
			description: 'No permission for commercial use is granted by this selection.',
		},
		'trademark-use': {
			label: 'Trademark use',
			description:
				'This license explicitly states that it does NOT grant trademark rights, even though licenses without such a statement probably do not grant any implicit trademark rights.',
		},
		liability: {
			label: 'Liability',
			description: 'This license includes a limitation of liability.',
		},
		'patent-use': {
			label: 'Patent use',
			description:
				'This license explicitly states that it does NOT grant any rights in the patents of contributors.',
		},
		warranty: {
			label: 'Warranty',
			description: 'This license explicitly states that it does NOT provide any warranty.',
		},
	},
	conditions: {
		'include-copyright': {
			label: 'License and copyright notice',
			description:
				'A copy of the license and copyright notice must be included with the licensed material.',
		},
		'include-copyright--source': {
			label: 'License and copyright notice for source',
			description:
				'A copy of the license and copyright notice must be included with the licensed material in source form, but is not required for binaries.',
		},
		'document-changes': {
			label: 'State changes',
			description: 'Changes made to the licensed material must be documented.',
		},
		'disclose-source': {
			label: 'Disclose source',
			description: 'Source code must be made available when the licensed material is distributed.',
		},
		'network-use-disclose': {
			label: 'Network use is distribution',
			description:
				'Users who interact with the licensed material via network are given the right to receive a copy of the source code.',
		},
		'same-license': {
			label: 'Same license',
			description:
				'Modifications must be released under the same license when distributing the licensed material. In some cases a similar or related license may be used.',
		},
		'same-license--file': {
			label: 'Same license (file)',
			description:
				'Modifications of existing files must be released under the same license when distributing the licensed material. In some cases a similar or related license may be used.',
		},
		'same-license--library': {
			label: 'Same license (library)',
			description:
				'Modifications must be released under the same license when distributing the licensed material. In some cases a similar or related license may be used, or this condition may not apply to works that use the licensed material as a library.',
		},
	},
}

const licenseSummaries: Record<string, LicenseSummary> = {
	'licenseref-all-rights-reserved': {
		description:
			'Your project will be shown as All rights reserved. This selection does not grant permission to use, copy, modify, or redistribute your project, including for private or commercial use. Permissions may be granted separately or by applicable platform terms, and exceptions under copyright law still apply.',
		sourceUrl: 'https://choosealicense.com/no-license/',
		permissions: [],
		limitations: ['copying', 'modifications', 'distribution', 'private-use', 'commercial-use'],
		conditions: [],
	},
	'0bsd': {
		description:
			'The BSD Zero Clause license goes further than the BSD 2-Clause license to allow you unlimited freedom with the software without requirements to include the copyright notice, license text, or disclaimer in either source or binary forms.',
		sourceUrl: 'https://choosealicense.com/licenses/0bsd/',
		permissions: ['commercial-use', 'distribution', 'modifications', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: [],
	},
	'afl-3.0': {
		description:
			'The Academic Free License is a variant of the Open Software License that does not require that the source code of derivative works be disclosed. It contains explicit copyright and patent grants and reserves trademark rights in the author.',
		sourceUrl: 'https://choosealicense.com/licenses/afl-3.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use', 'patent-use'],
		limitations: ['trademark-use', 'liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes'],
	},
	'agpl-3.0': {
		description:
			'Permissions of this strongest copyleft license are conditioned on making available complete source code of licensed works and modifications, which include larger works using a licensed work, under the same license. Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. When a modified version is used to provide a service over a network, the complete source code of the modified version must be made available.',
		sourceUrl: 'https://choosealicense.com/licenses/agpl-3.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: [
			'include-copyright',
			'document-changes',
			'disclose-source',
			'network-use-disclose',
			'same-license',
		],
	},
	'apache-2.0': {
		description:
			'A permissive license whose main conditions require preservation of copyright and license notices. Contributors provide an express grant of patent rights. Licensed works, modifications, and larger works may be distributed under different terms and without source code.',
		sourceUrl: 'https://choosealicense.com/licenses/apache-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['trademark-use', 'liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes'],
	},
	'artistic-2.0': {
		description:
			'Heavily favored by the Perl community, the Artistic license requires that modified versions of the software do not prevent users from running the standard version.',
		sourceUrl: 'https://choosealicense.com/licenses/artistic-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'trademark-use', 'warranty'],
		conditions: ['include-copyright', 'document-changes'],
	},
	'blueoak-1.0.0': {
		description:
			'A permissive license whose main conditions require providing notice of the license. Contributors provide an express grant of patent rights. Licensed works, modifications, and larger works may be distributed under different terms and without source code.',
		sourceUrl: 'https://choosealicense.com/licenses/blueoak-1.0.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'bsd-2-clause-patent': {
		description:
			'This license is designed to provide: a) a simple permissive license; b) that is compatible with the GNU GPL-2.0; and c) which also has an express patent grant included.',
		sourceUrl: 'https://choosealicense.com/licenses/bsd-2-clause-patent/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'bsd-2-clause': {
		description:
			'A permissive license that comes in two variants, the <a href="/licenses/bsd-2-clause/">BSD 2-Clause</a> and <a href="/licenses/bsd-3-clause/">BSD 3-Clause</a>. Both have very minute differences to the MIT license.',
		sourceUrl: 'https://choosealicense.com/licenses/bsd-2-clause/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'bsd-3-clause-clear': {
		description:
			'A variant of the <a href="/licenses/bsd-3-clause/">BSD 3-Clause License</a> that explicitly does not grant any patent rights.',
		sourceUrl: 'https://choosealicense.com/licenses/bsd-3-clause-clear/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'patent-use', 'warranty'],
		conditions: ['include-copyright'],
	},
	'bsd-3-clause': {
		description:
			'A permissive license similar to the <a href="/licenses/bsd-2-clause/">BSD 2-Clause License</a>, but with a 3rd clause that prohibits others from using the name of the copyright holder or its contributors to promote derived products without written consent.',
		sourceUrl: 'https://choosealicense.com/licenses/bsd-3-clause/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'bsd-4-clause': {
		description:
			'A permissive license similar to the <a href="/licenses/bsd-3-clause/">BSD 3-Clause License</a>, but with an "advertising clause" that requires an acknowledgment of the original source in all advertising material.',
		sourceUrl: 'https://choosealicense.com/licenses/bsd-4-clause/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'bsl-1.0': {
		description:
			'A simple permissive license only requiring preservation of copyright and license notices for source (and not binary) distribution. Licensed works, modifications, and larger works may be distributed under different terms and without source code.',
		sourceUrl: 'https://choosealicense.com/licenses/bsl-1.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright--source'],
	},
	'cc-by-4.0': {
		description:
			'Permits almost any use subject to providing credit and license notice. Frequently used for media assets and educational materials. The most common license for Open Access scientific publications. Not recommended for software.',
		sourceUrl: 'https://choosealicense.com/licenses/cc-by-4.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'trademark-use', 'patent-use', 'warranty'],
		conditions: ['include-copyright', 'document-changes'],
	},
	'cc-by-sa-4.0': {
		description:
			'Similar to <a href=\'/licenses/cc-by-4.0/\'>CC-BY-4.0</a> but requires derivatives be distributed under the same or a similar, <a href="https://creativecommons.org/compatiblelicenses">compatible</a> license. Frequently used for media assets and educational materials. A previous version is the default license for Wikipedia and other Wikimedia projects. Not recommended for software.',
		sourceUrl: 'https://choosealicense.com/licenses/cc-by-sa-4.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'trademark-use', 'patent-use', 'warranty'],
		conditions: ['include-copyright', 'document-changes', 'same-license'],
	},
	'cc0-1.0': {
		description:
			'The <a href="https://creativecommons.org/publicdomain/zero/1.0/">Creative Commons CC0 Public Domain Dedication</a> waives copyright interest in a work you\'ve created and dedicates it to the world-wide public domain. Use CC0 to opt out of copyright entirely and ensure your work has the widest reach. As with the Unlicense and typical software licenses, CC0 disclaims warranties. CC0 is very similar to the Unlicense.',
		sourceUrl: 'https://choosealicense.com/licenses/cc0-1.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'trademark-use', 'patent-use', 'warranty'],
		conditions: [],
	},
	'cecill-2.1': {
		description:
			'Strong copyleft license made by three French public research organisations, CEA, CNRS, and Inria, compatible with AGPL-3.0, EUPL-1.1, and GPL-2.0, or later versions of those licenses.',
		sourceUrl: 'https://choosealicense.com/licenses/cecill-2.1/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use', 'patent-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'network-use-disclose', 'disclose-source', 'same-license'],
	},
	'cern-ohl-p-2.0': {
		description:
			'A permissive license for hardware designs, with conditions only requiring preservation of notices. Contributors provide an express grant of patent rights. Licensed works, modifications, and larger works may be distributed under different terms and without sources.',
		sourceUrl: 'https://choosealicense.com/licenses/cern-ohl-p-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use', 'patent-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes'],
	},
	'cern-ohl-s-2.0': {
		description:
			'Permissions of this strongly reciprocal license for hardware designs are conditioned on making available complete sources of licensed works and modifications, which include larger works using a licensed work, under the same license. Notices must be preserved. Contributors provide an express grant of patent rights.',
		sourceUrl: 'https://choosealicense.com/licenses/cern-ohl-s-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use', 'patent-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes', 'disclose-source', 'same-license'],
	},
	'cern-ohl-w-2.0': {
		description:
			'Permissions of this weakly-reciprocal license are conditioned on making available complete sources of licensed works and modifications under the same license. Notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work through interfaces provided by the licensed work may be distributed under different terms and without sources for the larger work.',
		sourceUrl: 'https://choosealicense.com/licenses/cern-ohl-w-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use', 'patent-use'],
		limitations: ['liability', 'warranty'],
		conditions: [
			'include-copyright',
			'document-changes',
			'disclose-source',
			'same-license--library',
		],
	},
	'ecl-2.0': {
		description:
			'The Educational Community License version 2.0 ("ECL") consists of the Apache 2.0 license, modified to change the scope of the patent grant in section 3 to be specific to the needs of the education communities using this license.',
		sourceUrl: 'https://choosealicense.com/licenses/ecl-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['trademark-use', 'liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes'],
	},
	'epl-1.0': {
		description:
			'This commercially-friendly copyleft license provides the ability to commercially license binaries; a modern royalty-free patent license grant; and the ability for linked works to use other licenses, including commercial ones.',
		sourceUrl: 'https://choosealicense.com/licenses/epl-1.0/',
		permissions: ['commercial-use', 'distribution', 'modifications', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['disclose-source', 'include-copyright', 'same-license'],
	},
	'epl-2.0': {
		description:
			'This commercially-friendly copyleft license provides the ability to commercially license binaries; a modern royalty-free patent license grant; and the ability for linked works to use other licenses, including commercial ones.',
		sourceUrl: 'https://choosealicense.com/licenses/epl-2.0/',
		permissions: ['commercial-use', 'distribution', 'modifications', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['disclose-source', 'include-copyright', 'same-license'],
	},
	'eupl-1.1': {
		description:
			'The “European Union Public Licence” (EUPL) is a copyleft free/open source software license created on the initiative of and approved by the European Commission in 22 official languages of the European Union.',
		sourceUrl: 'https://choosealicense.com/licenses/eupl-1.1/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'trademark-use', 'warranty'],
		conditions: [
			'include-copyright',
			'disclose-source',
			'document-changes',
			'network-use-disclose',
			'same-license',
		],
	},
	'eupl-1.2': {
		description:
			'The European Union Public Licence (EUPL) is a copyleft free/open source software license created on the initiative of and approved by the European Commission in <a href="https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12">23 official languages</a> of the European Union.',
		sourceUrl: 'https://choosealicense.com/licenses/eupl-1.2/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'trademark-use', 'warranty'],
		conditions: [
			'include-copyright',
			'disclose-source',
			'document-changes',
			'network-use-disclose',
			'same-license',
		],
	},
	'gfdl-1.3': {
		description: 'A copyleft license for software documentation.',
		sourceUrl: 'https://choosealicense.com/licenses/gfdl-1.3/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'disclose-source', 'same-license', 'document-changes'],
	},
	'gpl-2.0': {
		description:
			'The GNU GPL is the most widely used free software license and has a strong copyleft requirement. When distributing derived works, the source code of the work must be made available under the same license. There are multiple variants of the GNU GPL, each with different requirements.',
		sourceUrl: 'https://choosealicense.com/licenses/gpl-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes', 'disclose-source', 'same-license'],
	},
	'gpl-3.0': {
		description:
			'Permissions of this strong copyleft license are conditioned on making available complete source code of licensed works and modifications, which include larger works using a licensed work, under the same license. Copyright and license notices must be preserved. Contributors provide an express grant of patent rights.',
		sourceUrl: 'https://choosealicense.com/licenses/gpl-3.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes', 'disclose-source', 'same-license'],
	},
	isc: {
		description:
			'A permissive license lets people do anything with your code with proper attribution and without warranty. The ISC license is functionally equivalent to the <a href="/licenses/bsd-2-clause/">BSD 2-Clause</a> and <a href="/licenses/mit/">MIT</a> licenses, removing some language that is no longer necessary.',
		sourceUrl: 'https://choosealicense.com/licenses/isc/',
		permissions: ['commercial-use', 'distribution', 'modifications', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'lgpl-2.1': {
		description:
			'Primarily used for software libraries, the GNU LGPL requires that derived works be licensed under the same license, but works that only link to it do not fall under this restriction. There are two commonly used versions of the GNU LGPL.',
		sourceUrl: 'https://choosealicense.com/licenses/lgpl-2.1/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: [
			'include-copyright',
			'disclose-source',
			'document-changes',
			'same-license--library',
		],
	},
	'lgpl-3.0': {
		description:
			'Permissions of this copyleft license are conditioned on making available complete source code of licensed works and modifications under the same license or the GNU GPLv3. Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work through interfaces provided by the licensed work may be distributed under different terms and without source code for the larger work.',
		sourceUrl: 'https://choosealicense.com/licenses/lgpl-3.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: [
			'include-copyright',
			'disclose-source',
			'document-changes',
			'same-license--library',
		],
	},
	'lppl-1.3c': {
		description:
			'The LaTeX Project Public License (LPPL) is the primary license under which the LaTeX kernel and the base LaTeX packages are distributed.',
		sourceUrl: 'https://choosealicense.com/licenses/lppl-1.3c/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'document-changes', 'disclose-source'],
	},
	'mit-0': {
		description:
			'A short and simple permissive license with no conditions, not even requiring preservation of copyright and license notices. Licensed works, modifications, and larger works may be distributed under different terms and without source code.',
		sourceUrl: 'https://choosealicense.com/licenses/mit-0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: [],
	},
	mit: {
		description:
			'A short and simple permissive license with conditions only requiring preservation of copyright and license notices. Licensed works, modifications, and larger works may be distributed under different terms and without source code.',
		sourceUrl: 'https://choosealicense.com/licenses/mit/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'mpl-2.0': {
		description:
			'Permissions of this weak copyleft license are conditioned on making available source code of licensed files and modifications of those files under the same license (or in certain cases, one of the GNU licenses). Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work may be distributed under different terms and without source code for files added in the larger work.',
		sourceUrl: 'https://choosealicense.com/licenses/mpl-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'trademark-use', 'warranty'],
		conditions: ['disclose-source', 'include-copyright', 'same-license--file'],
	},
	'ms-pl': {
		description: 'An open source license with a patent grant.',
		sourceUrl: 'https://choosealicense.com/licenses/ms-pl/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['warranty', 'trademark-use'],
		conditions: ['include-copyright'],
	},
	'ms-rl': {
		description:
			'An open source license with a patent grant similar to the <a href="/licenses/ms-pl/">Microsoft Public License</a>, with the additional condition that any source code for any derived file be provided under this license.',
		sourceUrl: 'https://choosealicense.com/licenses/ms-rl/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['warranty', 'trademark-use'],
		conditions: ['disclose-source', 'include-copyright', 'same-license--file'],
	},
	'mulanpsl-2.0': {
		description:
			'A permissive license similar to the <a href="/licenses/apache-2.0/">Apache License</a>, but explicitly states that it does not grant trademark rights. Mulan Permissive Software License is the first open source license in both Chinese and English approved by OSI.',
		sourceUrl: 'https://choosealicense.com/licenses/mulanpsl-2.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'trademark-use', 'warranty'],
		conditions: ['include-copyright'],
	},
	ncsa: {
		description:
			'The University of Illinois/NCSA Open Source License, or UIUC license, is a permissive free software license, based on the <a href="/licenses/mit/">MIT/X11 license</a>  and the <a href="/licenses/bsd-3-clause/">BSD 3-clause License</a>. Its conditions include requiring the preservation of copyright and license notices both in source and in binary distributions and the prohibition of using the names of the authors or the project organization to promote or endorse derived products.',
		sourceUrl: 'https://choosealicense.com/licenses/ncsa/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	'odbl-1.0': {
		description:
			'The Open Database License (ODbL) is a license agreement intended to allow users to freely share, modify, and use a database while maintaining this same freedom for others.',
		sourceUrl: 'https://choosealicense.com/licenses/odbl-1.0/',
		permissions: ['commercial-use', 'distribution', 'modifications', 'private-use'],
		limitations: ['liability', 'patent-use', 'trademark-use', 'warranty'],
		conditions: ['disclose-source', 'include-copyright', 'same-license'],
	},
	'ofl-1.1': {
		description:
			'The Open Font License (OFL) is maintained by SIL International. It attempts to be a compromise between the values of the free software and typeface design communities. It is used for almost all open source font projects, including those by Adobe, Google and Mozilla.',
		sourceUrl: 'https://choosealicense.com/licenses/ofl-1.1/',
		permissions: ['private-use', 'commercial-use', 'modifications', 'distribution'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright', 'same-license'],
	},
	'osl-3.0': {
		description:
			"Permissions of this copyleft license are conditioned on distributing source code of licensed works and modifications under the same license. Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. Using the work or modifications to provide services to external users is treated as distribution and also requires making source code available. Works that merely link to a licensed work are considered collective works and are not subject to the license's reciprocity requirements.",
		sourceUrl: 'https://choosealicense.com/licenses/osl-3.0/',
		permissions: ['commercial-use', 'distribution', 'modifications', 'patent-use', 'private-use'],
		limitations: ['trademark-use', 'liability', 'warranty'],
		conditions: [
			'include-copyright',
			'disclose-source',
			'document-changes',
			'network-use-disclose',
			'same-license',
		],
	},
	postgresql: {
		description: 'A very short, BSD-style license, used specifically for PostgreSQL.',
		sourceUrl: 'https://choosealicense.com/licenses/postgresql/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	unlicense: {
		description:
			'A license with no conditions whatsoever which dedicates works to the public domain. Unlicensed works, modifications, and larger works may be distributed under different terms and without source code.',
		sourceUrl: 'https://choosealicense.com/licenses/unlicense/',
		permissions: ['private-use', 'commercial-use', 'modifications', 'distribution'],
		limitations: ['liability', 'warranty'],
		conditions: [],
	},
	'upl-1.0': {
		description:
			'A permissive, OSI- and FSF-approved, GPL-compatible license, expressly allowing attribution with just a copyright notice and a short form link rather than the full text of the license.  Includes an express grant of patent rights.  Licensed works and modifications may be distributed under different terms and without source code, and the patent grant may also optionally be expanded to larger works to permit use as a contributor license agreement.',
		sourceUrl: 'https://choosealicense.com/licenses/upl-1.0/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'patent-use', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright'],
	},
	vim: {
		description:
			'There are no restrictions on using or distributing an unmodified copy of the software. Parts of the software may also be distributed, but the license text must always be included. For modified versions a few restrictions apply. The license is GPL compatible, you may compile the software with GPL libraries and distribute it.',
		sourceUrl: 'https://choosealicense.com/licenses/vim/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: [],
		conditions: ['include-copyright', 'document-changes', 'disclose-source', 'same-license'],
	},
	wtfpl: {
		description:
			'The easiest license out there. It gives the user permissions to do whatever they want with your code.',
		sourceUrl: 'https://choosealicense.com/licenses/wtfpl/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: [],
		conditions: [],
	},
	zlib: {
		description:
			'A short permissive license, compatible with GPL. Requires altered source versions to be documented as such.',
		sourceUrl: 'https://choosealicense.com/licenses/zlib/',
		permissions: ['commercial-use', 'modifications', 'distribution', 'private-use'],
		limitations: ['liability', 'warranty'],
		conditions: ['include-copyright--source', 'document-changes'],
	},
}

/** Match known license identifiers and their only/or-later forms; do not infer custom or combined licenses. */
export function getLicenseSummary(licenseId: string): LicenseSummary | undefined {
	const id = licenseId.toLowerCase()
	const baseId = id.replace(/-(only|or-later)$/, '')
	const key = Object.hasOwn(licenseSummaries, id) ? id : baseId
	return Object.hasOwn(licenseSummaries, key) ? licenseSummaries[key] : undefined
}
