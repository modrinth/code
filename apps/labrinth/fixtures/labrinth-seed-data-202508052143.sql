BEGIN TRANSACTION;

COPY public.categories (id, category, project_type, icon, header, ordering) FROM stdin;
60	cursed	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="7.5" width="10" height="14" rx="5"/><polyline points="2 12.5 4 14.5 7 14.5"/><polyline points="22 12.5 20 14.5 17 14.5"/><polyline points="3 21.5 5 18.5 7 17.5"/><polyline points="21 21.5 19 18.5 17 17.5"/><polyline points="3 8.5 5 10.5 7 11.5"/><polyline points="21 8.5 19 10.5 17 11.5"/><line x1="12" y1="7.5" x2="12" y2="21.5"/><path d="M15.38,8.82A3,3,0,0,0,16,7h0a3,3,0,0,0-3-3H11A3,3,0,0,0,8,7H8a3,3,0,0,0,.61,1.82"/><line x1="9" y1="4.5" x2="8" y2="2.5"/><line x1="15" y1="4.5" x2="16" y2="2.5"/></svg>	categories	0
61	locale	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>	features	0
62	48x	3		resolutions	0
1	technology	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="12" x2="2" y2="12"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/><line x1="6" y1="16" x2="6.01" y2="16"/><line x1="10" y1="16" x2="10.01" y2="16"/></svg>	categories	0
16	challenging	2	<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" /></svg>	categories	0
5	decoration	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>	categories	0
6	library	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>	categories	0
7	cursed	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="7.5" width="10" height="14" rx="5"/><polyline points="2 12.5 4 14.5 7 14.5"/><polyline points="22 12.5 20 14.5 17 14.5"/><polyline points="3 21.5 5 18.5 7 17.5"/><polyline points="21 21.5 19 18.5 17 17.5"/><polyline points="3 8.5 5 10.5 7 11.5"/><polyline points="21 8.5 19 10.5 17 11.5"/><line x1="12" y1="7.5" x2="12" y2="21.5"/><path d="M15.38,8.82A3,3,0,0,0,16,7h0a3,3,0,0,0-3-3H11A3,3,0,0,0,8,7H8a3,3,0,0,0,.61,1.82"/><line x1="9" y1="4.5" x2="8" y2="2.5"/><line x1="15" y1="4.5" x2="16" y2="2.5"/></svg>	categories	0
2	adventure	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/></svg>	categories	0
56	64x	3		resolutions	0
55	32x	3		resolutions	0
58	256x	3		resolutions	0
59	512x+	3		resolutions	0
54	16x	3		resolutions	0
63	path-tracing	4	<svg viewBox="0 0 24 24" style="" fill="none" stroke="currentColor" stroke-width="2"><path d="M2.977 19.17h16.222" style="" transform="translate(-.189 -.328) scale(1.09932)"/><path d="M3.889 3.259 12 19.17l5.749-11.277" style="" transform="translate(-1.192 -.328) scale(1.09932)"/><path d="M9.865 6.192h4.623v4.623" style="" transform="scale(1.09931) rotate(-18 20.008 .02)"/></svg>	features	0
64	realistic	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/><circle cx="12" cy="13" r="3"/></svg>	categories	0
65	medium	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M2 20h.01"></path><path d="M7 20v-4"></path><path d="M12 20v-8"></path></svg>	performance impact	0
66	low	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M2 20h.01"></path><path d="M7 20v-4"></path></svg>	performance impact	0
67	high	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M2 20h.01"></path><path d="M7 20v-4"></path><path d="M12 20v-8"></path><path d="M17 20V8"></path></svg>	performance impact	0
68	atmosphere	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="M20 12h2"/><path d="m19.07 4.93-1.41 1.41"/><path d="M15.947 12.65a4 4 0 0 0-5.925-4.128"/><path d="M3 20a5 5 0 1 1 8.9-4H13a3 3 0 0 1 2 5.24"/><path d="M11 20v2"/><path d="M7 19v2"/></svg>	features	0
69	fantasy	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.2 1.2 0 0 0 1.72 0L21.64 5.36a1.2 1.2 0 0 0 0-1.72Z"/><path d="m14 7 3 3"/><path d="M5 6v4"/><path d="M19 14v4"/><path d="M10 2v2"/><path d="M7 8H3"/><path d="M21 16h-4"/><path d="M11 3H9"/></svg>	categories	0
70	foliage	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="" data-darkreader-inline-stroke=""><path d="M12 22v-7l-2-2"/><path d="M17 8v.8A6 6 0 0 1 13.8 20v0H10v0A6.5 6.5 0 0 1 7 8h0a5 5 0 0 1 10 0Z"/><path d="m14 14-2 2"/></svg>	features	0
71	bloom	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 2h8l4 10H4L8 2Z"/><path d="M12 12v6"/><path d="M8 22v-2c0-1.1.9-2 2-2h4a2 2 0 0 1 2 2v2H8Z"/></svg>	features	0
72	vanilla-like	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="" data-darkreader-inline-stroke=""><path d="m7 11 4.08 10.35a1 1 0 0 0 1.84 0L17 11"/><path d="M17 7A5 5 0 0 0 7 7"/><path d="M17 7a2 2 0 0 1 0 4H7a2 2 0 0 1 0-4"/></svg>	categories	0
73	cartoon	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="" data-darkreader-inline-stroke=""><path d="m9.06 11.9 8.07-8.06a2.85 2.85 0 1 1 4.03 4.03l-8.06 8.08"/><path d="M7.07 14.94c-1.66 0-3 1.35-3 3.02 0 1.33-2.5 1.52-2 2.02 1.08 1.1 2.49 2.02 4 2.02 2.2 0 4-1.8 4-4.04a3.01 3.01 0 0 0-3-3.02z"/></svg>	categories	0
74	potato	4	<svg viewBox="0 0 512 512" fill="currentColor" stroke="currentColor"><g><g><path d="M218.913,116.8c-6.4-6.4-16-6.4-22.4,0c-3.2,3.2-4.8,6.4-4.8,11.2s1.6,8,4.8,11.2c3.2,3.2,8,4.8,11.2,4.8    c4.8,0,8-1.6,11.2-4.8c3.2-3.2,4.8-6.4,4.8-11.2S222.113,120,218.913,116.8z"/></g></g><g><g><path d="M170.913,372.8c-6.4-6.4-16-6.4-22.4,0c-3.2,3.2-4.8,6.4-4.8,11.2s1.6,8,4.8,11.2c3.2,3.2,8,4.8,11.2,4.8    c4.8,0,8-1.6,11.2-4.8c3.2-3.2,4.8-8,4.8-11.2C175.713,379.2,174.113,376,170.913,372.8z"/></g></g><g><g><path d="M250.913,228.8c-4.8-6.4-16-6.4-22.4,0c-3.2,3.2-4.8,6.4-4.8,11.2s1.6,8,4.8,11.2c3.2,3.2,8,4.8,11.2,4.8    c4.8,0,8-1.6,11.2-4.8c3.2-3.2,4.8-8,4.8-11.2C255.713,235.2,254.113,232,250.913,228.8z"/></g></g><g><g><path d="M410.913,212.8c-4.8-6.4-16-6.4-22.4,0c-3.2,3.2-4.8,6.4-4.8,11.2s1.6,8,4.8,11.2c3.2,3.2,8,4.8,11.2,4.8    c4.8,0,8-1.6,11.2-4.8c3.2-3.2,4.8-8,4.8-11.2C415.713,219.2,414.113,216,410.913,212.8z"/></g></g><g><g><path d="M346.913,308.8c-4.8-6.4-16-6.4-22.4,0c-3.2,3.2-4.8,6.4-4.8,11.2s1.6,8,4.8,11.2c3.2,3.2,8,4.8,11.2,4.8    c4.8,0,8-1.6,11.2-4.8c3.2-3.2,4.8-8,4.8-11.2C351.713,315.2,350.113,312,346.913,308.8z"/></g></g><g><g><path d="M346.913,100.8c-6.4-6.4-16-6.4-22.4,0c-3.2,3.2-4.8,6.4-4.8,11.2s1.6,8,4.8,11.2c3.2,3.2,8,4.8,11.2,4.8    c4.8,0,8-1.6,11.2-4.8s4.8-6.4,4.8-11.2S350.113,104,346.913,100.8z"/></g></g><g><g><path d="M503.713,142.4c-28.8-136-179.2-142.4-208-142.4c-4.8,0-9.6,0-16,0c-67.2,1.6-132.8,36.8-187.2,97.6    c-60.8,67.2-96,155.2-91.2,227.2c8,126.4,70.4,187.2,192,187.2c115.2,0,201.6-33.6,256-100.8    C513.313,331.2,519.713,219.2,503.713,142.4z M423.713,392c-48,59.2-126.4,89.6-230.4,89.6s-152-48-160-158.4    c-4.8-64,28.8-144,83.2-203.2c48-54.4,107.2-84.8,164.8-88c4.8,0,9.6,0,14.4,0c140.8,0,171.2,89.6,176,116.8    C486.113,219.2,481.313,320,423.713,392z"/></g></g></svg>	performance impact	0
75	shadows	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m8 3 4 8 5-5 5 15H2L8 3z"/></svg>	features	0
76	pbr	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="9" y1="18" x2="15" y2="18"/><line x1="10" y1="22" x2="14" y2="22"/><path d="M15.09 14c.18-.98.65-1.74 1.41-2.5A4.65 4.65 0 0 0 18 8 6 6 0 0 0 6 8c0 1 .23 2.23 1.5 3.5A4.61 4.61 0 0 1 8.91 14"/></svg>	features	0
77	semi-realistic	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/><line x1="7" y1="2" x2="7" y2="22"/><line x1="17" y1="2" x2="17" y2="22"/><line x1="2" y1="12" x2="22" y2="12"/><line x1="2" y1="7" x2="7" y2="7"/><line x1="2" y1="17" x2="7" y2="17"/><line x1="17" y1="17" x2="22" y2="17"/><line x1="17" y1="7" x2="22" y2="7"/></svg>	categories	0
78	cursed	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="7.5" width="10" height="14" rx="5"/><polyline points="2 12.5 4 14.5 7 14.5"/><polyline points="22 12.5 20 14.5 17 14.5"/><polyline points="3 21.5 5 18.5 7 17.5"/><polyline points="21 21.5 19 18.5 17 17.5"/><polyline points="3 8.5 5 10.5 7 11.5"/><polyline points="21 8.5 19 10.5 17 11.5"/><line x1="12" y1="7.5" x2="12" y2="21.5"/><path d="M15.38,8.82A3,3,0,0,0,16,7h0a3,3,0,0,0-3-3H11A3,3,0,0,0,8,7H8a3,3,0,0,0,.61,1.82"/><line x1="9" y1="4.5" x2="8" y2="2.5"/><line x1="15" y1="4.5" x2="16" y2="2.5"/></svg>	categories	0
79	reflections	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style=""><path d="m3 7 5 5-5 5V7"/><path d="m21 7-5 5 5 5V7"/><path d="M12 20v2"/><path d="M12 14v2"/><path d="M12 8v2"/><path d="M12 2v2"/></svg>	features	0
80	screenshot	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="9" cy="9" r="2"></circle><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"></path></svg>	performance impact	0
81	colored-lighting	4	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="7.618" cy="6.578" r="5.422" style="" transform="translate(3.143 .726) scale(1.16268)"/><circle cx="7.618" cy="6.578" r="5.422" style="" transform="translate(-.862 7.796) scale(1.16268)"/><circle cx="7.618" cy="6.578" r="5.422" style="" transform="translate(7.148 7.796) scale(1.16268)"/></svg>	features	0
57	128x	3		resolutions	0
30	economy	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>	categories	0
31	management	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>	categories	0
26	optimization	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>	categories	0
27	mobs	1	<svg xml:space="preserve" fill-rule="evenodd" stroke-linejoin="round" stroke-miterlimit="1.5" clip-rule="evenodd" viewBox="0 0 24 24">\n  <path fill="none" d="M0 0h24v24H0z"/>\n  <path fill="none" stroke="currentColor" stroke-width="2" d="M3 3h18v18H3z"/>\n  <path stroke="currentColor" fill="currentColor" d="M6 6h4v4H6zm8 0h4v4h-4zm-4 4h4v2h2v6h-2v-2h-4v2H8v-6h2v-2Z"/>\n</svg>	categories	0
28	transportation	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="1" y="3" width="15" height="13"/><polygon points="16 8 20 8 23 11 23 16 16 16 16 8"/><circle cx="5.5" cy="18.5" r="2.5"/><circle cx="18.5" cy="18.5" r="2.5"/></svg>	categories	0
24	kitchen-sink	2	<svg viewBox="0 0 24 24" xml:space="preserve"><g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m19.9 14-1.4 4.9c-.3 1-1.1 1.7-2.1 1.7H7.6c-.9 0-1.8-.7-2.1-1.7L4.1 14h15.8zM12 10V4.5M12 4.5c0-1.2.9-2.1 2.1-2.1M14.1 2.4c1.2 0 2.1.9 2.1 2.1M22.2 12c0 .6-.2 1.1-.6 1.4-.4.4-.9.6-1.4.6H3.8c-1.1 0-2-.9-2-2 0-.6.2-1.1.6-1.4.4-.4.9-.6 1.4-.6h16.4c1.1 0 2 .9 2 2z"/></g><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M16.2 7.2h0"/></svg>	categories	0
44	blocks	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>	features	0
43	audio	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 18v-6a9 9 0 0 1 18 0v6"/><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/></svg>	features	0
21	combat	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.573 20.038L3.849 7.913 2.753 2.755 7.838 4.06 19.47 18.206l-1.898 1.832z"/><path d="M7.45 14.455l-3.043 3.661 1.887 1.843 3.717-3.25"/><path d="M16.75 10.82l3.333-2.913 1.123-5.152-5.091 1.28-2.483 2.985"/><path d="M21.131 16.602l-5.187 5.01 2.596-2.508 2.667 2.761"/><path d="M2.828 16.602l5.188 5.01-2.597-2.508-2.667 2.761"/></svg>	categories	0
22	adventure	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/></svg>	categories	0
23	technology	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="12" x2="2" y2="12"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/><line x1="6" y1="16" x2="6.01" y2="16"/><line x1="10" y1="16" x2="10.01" y2="16"/></svg>	categories	0
33	minigame	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="7"/><polyline points="8.21 13.89 7 23 12 20 17 23 15.79 13.88"/></svg>	categories	0
34	combat	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.573 20.038L3.849 7.913 2.753 2.755 7.838 4.06 19.47 18.206l-1.898 1.832z"/><path d="M7.45 14.455l-3.043 3.661 1.887 1.843 3.717-3.25"/><path d="M16.75 10.82l3.333-2.913 1.123-5.152-5.091 1.28-2.483 2.985"/><path d="M21.131 16.602l-5.187 5.01 2.596-2.508 2.667 2.761"/><path d="M2.828 16.602l5.188 5.01-2.597-2.508-2.667 2.761"/></svg>	categories	0
35	decoration	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>	categories	0
36	modded	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"\\><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>	categories	0
47	environment	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"]><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>	features	0
46	entities	3	<svg xml:space="preserve" fill-rule="evenodd" stroke-linejoin="round" stroke-miterlimit="1.5" clip-rule="evenodd" viewBox="0 0 24 24">\n  <path fill="none" d="M0 0h24v24H0z"/>\n  <path fill="none" stroke="currentColor" stroke-width="2" d="M3 3h18v18H3z"/>\n  <path stroke="currentColor" fill="currentColor" d="M6 6h4v4H6zm8 0h4v4h-4zm-4 4h4v2h2v6h-2v-2h-4v2H8v-6h2v-2Z"/>\n</svg>	features	0
32	game-mechanics	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="1" y1="14" x2="7" y2="14"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="17" y1="16" x2="23" y2="16"/></svg>	categories	0
41	utility	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>	categories	0
45	core-shaders	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2" ry="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>	features	0
40	tweaks	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>	categories	0
51	items	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>	features	0
52	models	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>	features	0
48	equipment	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>	features	0
49	fonts	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 7 4 4 20 4 20 7"/><line x1="9" y1="20" x2="15" y2="20"/><line x1="12" y1="4" x2="12" y2="20"/></svg>	features	0
37	simplistic	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>	categories	0
38	realistic	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>	categories	0
39	themed	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 19l7-7 3 3-7 7-3-3z"/><path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/><path d="M2 2l7.586 7.586"/><circle cx="11" cy="11" r="2"/></svg>	categories	0
3	magic	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 4V2"></path><path d="M15 16v-2"></path><path d="M8 9h2"></path><path d="M20 9h2"></path><path d="M17.8 11.8 19 13"></path><path d="M15 9h0"></path><path d="M17.8 6.2 19 5"></path><path d="m3 21 9-9"></path><path d="M12.2 6.2 11 5"></path></svg>	categories	0
4	utility	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>	categories	0
15	optimization	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>	categories	0
9	storage	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg>	categories	0
53	8x-	3		resolutions	0
42	vanilla-like	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="" data-darkreader-inline-stroke=""><path d="m7 11 4.08 10.35a1 1 0 0 0 1.84 0L17 11"/><path d="M17 7A5 5 0 0 0 7 7"/><path d="M17 7a2 2 0 0 1 0 4H7a2 2 0 0 1 0-4"/></svg>	categories	0
10	food	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2.27 21.7s9.87-3.5 12.73-6.36a4.5 4.5 0 0 0-6.36-6.37C5.77 11.84 2.27 21.7 2.27 21.7zM8.64 14l-2.05-2.04M15.34 15l-2.46-2.46"></path><path d="M22 9s-1.33-2-3.5-2C16.86 7 15 9 15 9s1.33 2 3.5 2S22 9 22 9z"></path><path d="M15 2s-2 1.33-2 3.5S15 9 15 9s2-1.84 2-3.5C17 3.33 15 2 15 2z"></path></svg>	categories	0
11	equipment	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.573 20.038L3.849 7.913 2.753 2.755 7.838 4.06 19.47 18.206l-1.898 1.832z"/><path d="M7.45 14.455l-3.043 3.661 1.887 1.843 3.717-3.25"/><path d="M16.75 10.82l3.333-2.913 1.123-5.152-5.091 1.28-2.483 2.985"/><path d="M21.131 16.602l-5.187 5.01 2.596-2.508 2.667 2.761"/><path d="M2.828 16.602l5.188 5.01-2.597-2.508-2.667 2.761"/></svg>	categories	0
50	gui	3	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>	features	0
8	worldgen	1	<svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>	categories	0
17	multiplayer	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>	categories	0
18	quests	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="2" width="6" height="6"></rect><rect x="16" y="16" width="6" height="6"></rect><rect x="2" y="16" width="6" height="6"></rect><path d="M12 8v4m0 0H5v4m7-4h7v4"></path></svg>	categories	0
19	magic	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 4V2"></path><path d="M15 16v-2"></path><path d="M8 9h2"></path><path d="M20 9h2"></path><path d="M17.8 11.8 19 13"></path><path d="M15 9h0"></path><path d="M17.8 6.2 19 5"></path><path d="m3 21 9-9"></path><path d="M12.2 6.2 11 5"></path></svg>	categories	0
20	lightweight	2	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"></path><line x1="16" y1="8" x2="2" y2="22"></line><line x1="17.5" y1="15" x2="9" y2="15"></line></svg>\n	categories	0
29	social	1	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>	categories	0
\.


TRUNCATE TABLE public.loaders RESTART IDENTITY CASCADE;
COPY public.loaders (id, loader, icon, hidable, metadata) FROM stdin;
1	forge	<svg xml:space="preserve" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="1.5" clip-rule="evenodd" viewBox="0 0 24 24">\n  <path fill="none" d="M0 0h24v24H0z"></path>\n  <path fill="none" stroke="currentColor" stroke-width="2" d="M2 7.5h8v-2h12v2s-7 3.4-7 6 3.1 3.1 3.1 3.1l.9 3.9H5l1-4.1s3.8.1 4-2.9c.2-2.7-6.5-.7-8-6Z"></path>\n</svg>	f	{}
3	quilt	<svg xmlns:xlink="http://www.w3.org/1999/xlink" xml:space="preserve" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="2" clip-rule="evenodd" viewBox="0 0 24 24">\n  <defs>\n    <path id="quilt" fill="none" stroke="currentColor" stroke-width="65.6" d="M442.5 233.9c0-6.4-5.2-11.6-11.6-11.6h-197c-6.4 0-11.6 5.2-11.6 11.6v197c0 6.4 5.2 11.6 11.6 11.6h197c6.4 0 11.6-5.2 11.6-11.7v-197Z"></path>\n  </defs>\n  <path fill="none" d="M0 0h24v24H0z"></path>\n  <use xlink:href="#quilt" stroke-width="65.6" transform="matrix(.03053 0 0 .03046 -3.2 -3.2)"></use>\n  <use xlink:href="#quilt" stroke-width="65.6" transform="matrix(.03053 0 0 .03046 -3.2 7)"></use>\n  <use xlink:href="#quilt" stroke-width="65.6" transform="matrix(.03053 0 0 .03046 6.9 -3.2)"></use>\n  <path fill="none" stroke="currentColor" stroke-width="70.4" d="M442.5 234.8c0-7-5.6-12.5-12.5-12.5H234.7c-6.8 0-12.4 5.6-12.4 12.5V430c0 6.9 5.6 12.5 12.4 12.5H430c6.9 0 12.5-5.6 12.5-12.5V234.8Z" transform="rotate(45 3.5 24) scale(.02843 .02835)"></path>\n</svg>	f	{}
2	fabric	<svg xmlns="http://www.w3.org/2000/svg" xml:space="preserve" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" clip-rule="evenodd" viewBox="0 0 24 24">\n  <path fill="none" d="M0 0h24v24H0z"/>\n  <path fill="none" stroke="currentColor" stroke-width="23" d="m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8" transform="matrix(.08671 0 0 .0867 -49.8 -56)"/>\n</svg>	f	{}
4	modloader	<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" xml:space="preserve"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M1.4 18V6h3.8v1.5h1.5V9h1.5V7.5h1.5V6h3.8v12H9.7v-5.3H9v1.5H6v-1.5h-.8V18H1.4zm12.1 0V6h3.8v9h5.3v3h-9.1z"/></svg>	f	{}
5	rift	<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" xml:space="preserve"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M2.7 6.6v10.8l9.3 5.3 9.3-5.3V6.6L12 1.3zm0 0L12 12m9.3-5.4L12 12m0 10.7V12"/></svg>	f	{}
6	liteloader	<svg clip-rule="evenodd" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="1.5" version="1.1" viewBox="0 0 24 24" xml:space="preserve" xmlns="http://www.w3.org/2000/svg"><rect width="24" height="24" fill="none"/><path d="m3.924 21.537s3.561-1.111 8.076-6.365c2.544-2.959 2.311-1.986 4-4.172" fill="none" stroke="currentColor" stroke-width="2px"/><path d="m7.778 19s1.208-0.48 4.222 0c2.283 0.364 6.037-4.602 6.825-6.702 1.939-5.165 0.894-10.431 0.894-10.431s-4.277 4.936-6.855 7.133c-5.105 4.352-6.509 11-6.509 11" fill="none" stroke="currentColor" stroke-width="2px"/></svg>	f	{}
7	minecraft	<svg viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9.504 1.132a1 1 0 01.992 0l1.75 1a1 1 0 11-.992 1.736L10 3.152l-1.254.716a1 1 0 11-.992-1.736l1.75-1zM5.618 4.504a1 1 0 01-.372 1.364L5.016 6l.23.132a1 1 0 11-.992 1.736L4 7.723V8a1 1 0 01-2 0V6a.996.996 0 01.52-.878l1.734-.99a1 1 0 011.364.372zm8.764 0a1 1 0 011.364-.372l1.733.99A1.002 1.002 0 0118 6v2a1 1 0 11-2 0v-.277l-.254.145a1 1 0 11-.992-1.736l.23-.132-.23-.132a1 1 0 01-.372-1.364zm-7 4a1 1 0 011.364-.372L10 8.848l1.254-.716a1 1 0 11.992 1.736L11 10.58V12a1 1 0 11-2 0v-1.42l-1.246-.712a1 1 0 01-.372-1.364zM3 11a1 1 0 011 1v1.42l1.246.712a1 1 0 11-.992 1.736l-1.75-1A1 1 0 012 14v-2a1 1 0 011-1zm14 0a1 1 0 011 1v2a1 1 0 01-.504.868l-1.75 1a1 1 0 11-.992-1.736L16 13.42V12a1 1 0 011-1zm-9.618 5.504a1 1 0 011.364-.372l.254.145V16a1 1 0 112 0v.277l.254-.145a1 1 0 11.992 1.736l-1.735.992a.995.995 0 01-1.022 0l-1.735-.992a1 1 0 01-.372-1.364z" clip-rule="evenodd" /></svg>	f	{}
9	spigot	<svg viewBox="0 0 332 284" style="fill-rule:evenodd;clip-rule:evenodd;stroke-linejoin:round;fill:none;fill-rule:nonzero;stroke-width:24px;" stroke="currentColor"><path d="M147.5,27l27,-15l27.5,15l66.5,0l0,33.5l-73,-0.912l0,45.5l26,-0.088l0,31.5l-12.5,0l0,15.5l16,21.5l35,0l0,-21.5l35.5,0l0,21.5l24.5,0l0,55.5l-24.5,0l0,17l-35.5,0l0,-27l-35,0l-55.5,14.5l-67.5,-14.5l-15,14.5l18,12.5l-3,24.5l-41.5,1.5l-48.5,-19.5l6,-19l24.5,-4.5l16,-41l79,-36l-7,-15.5l0,-31.5l23.5,0l0,-45.5l-73.5,0l0,-32.5l67,0Z"/></svg>	f	{"platform": false}
16	optifine	<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path d="M10.985 9.205c0-1.38-1.121-2.5-2.5-2.5H7.156a2.5 2.5 0 0 0-2.5 2.5v5.59a2.5 2.5 0 0 0 2.5 2.5h1.329c1.379 0 2.5-1.12 2.5-2.5v-5.59ZM14.793 17.295v-9.34a1.252 1.252 0 0 1 1.25-1.25h3.301M18.007 10.997h-3.214" /></svg>	f	{}
17	iris	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m22.59 12.013-3.01 3.126v4.405l.005.019-4.251-.005-2.994 3.115h-.003l-3.003-3.132H5.1l-.018.005.005-4.424-2.994-3.116-.003-.023L5.1 8.858V4.452l-.005-.019 4.252.005 2.993-3.115h.003l3.003 3.132h4.234l.018-.005-.005 4.425 2.994 3.115" style="" transform="translate(-.344)"/><path d="m17.229 12.005-1.436 1.491v2.101l.003.009-2.028-.002-1.428 1.486h-.001l-1.433-1.494H8.887l-.008.002.002-2.11-1.428-1.486-.001-.011L8.887 10.5V8.399l-.002-.009 2.027.002 1.428-1.485h.002l1.432 1.494h2.019l.009-.003-.003 2.11 1.428 1.486" style="" transform="translate(-.344)"/></svg>	f	{}
18	canvas	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 1.305 1.305 12 12 22.695 22.695 12 12 1.305Z" style=""/><path d="M12 5.547 5.547 12 12 18.453 18.453 12 12 5.547Z" style=""/><path d="M12 9.79 9.79 12 12 14.21 14.21 12 12 9.79Z" style=""/></svg>	f	{}
19	vanilla	<svg viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9.504 1.132a1 1 0 01.992 0l1.75 1a1 1 0 11-.992 1.736L10 3.152l-1.254.716a1 1 0 11-.992-1.736l1.75-1zM5.618 4.504a1 1 0 01-.372 1.364L5.016 6l.23.132a1 1 0 11-.992 1.736L4 7.723V8a1 1 0 01-2 0V6a.996.996 0 01.52-.878l1.734-.99a1 1 0 011.364.372zm8.764 0a1 1 0 011.364-.372l1.733.99A1.002 1.002 0 0118 6v2a1 1 0 11-2 0v-.277l-.254.145a1 1 0 11-.992-1.736l.23-.132-.23-.132a1 1 0 01-.372-1.364zm-7 4a1 1 0 011.364-.372L10 8.848l1.254-.716a1 1 0 11.992 1.736L11 10.58V12a1 1 0 11-2 0v-1.42l-1.246-.712a1 1 0 01-.372-1.364zM3 11a1 1 0 011 1v1.42l1.246.712a1 1 0 11-.992 1.736l-1.75-1A1 1 0 012 14v-2a1 1 0 011-1zm14 0a1 1 0 011 1v2a1 1 0 01-.504.868l-1.75 1a1 1 0 11-.992-1.736L16 13.42V12a1 1 0 011-1zm-9.618 5.504a1 1 0 011.364-.372l.254.145V16a1 1 0 112 0v.277l.254-.145a1 1 0 11.992 1.736l-1.735.992a.995.995 0 01-1.022 0l-1.735-.992a1 1 0 01-.372-1.364z" clip-rule="evenodd" /></svg>	f	{}
20	datapack	<svg viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9.504 1.132a1 1 0 01.992 0l1.75 1a1 1 0 11-.992 1.736L10 3.152l-1.254.716a1 1 0 11-.992-1.736l1.75-1zM5.618 4.504a1 1 0 01-.372 1.364L5.016 6l.23.132a1 1 0 11-.992 1.736L4 7.723V8a1 1 0 01-2 0V6a.996.996 0 01.52-.878l1.734-.99a1 1 0 011.364.372zm8.764 0a1 1 0 011.364-.372l1.733.99A1.002 1.002 0 0118 6v2a1 1 0 11-2 0v-.277l-.254.145a1 1 0 11-.992-1.736l.23-.132-.23-.132a1 1 0 01-.372-1.364zm-7 4a1 1 0 011.364-.372L10 8.848l1.254-.716a1 1 0 11.992 1.736L11 10.58V12a1 1 0 11-2 0v-1.42l-1.246-.712a1 1 0 01-.372-1.364zM3 11a1 1 0 011 1v1.42l1.246.712a1 1 0 11-.992 1.736l-1.75-1A1 1 0 012 14v-2a1 1 0 011-1zm14 0a1 1 0 011 1v2a1 1 0 01-.504.868l-1.75 1a1 1 0 11-.992-1.736L16 13.42V12a1 1 0 011-1zm-9.618 5.504a1 1 0 011.364-.372l.254.145V16a1 1 0 112 0v.277l.254-.145a1 1 0 11.992 1.736l-1.735.992a.995.995 0 01-1.022 0l-1.735-.992a1 1 0 01-.372-1.364z" clip-rule="evenodd" /></svg>	f	{}
15	sponge	<svg viewBox="0 0 268 313" style="fill-rule:evenodd;clip-rule:evenodd;stroke-linecap:round;stroke-linejoin:round;fill:none;fill-rule:nonzero;stroke-width:24px;" stroke="currentColor"><path d="M84.299,35.5c-5.547,-13.776 -19.037,-23.5 -34.799,-23.5c-20.711,0 -37.5,16.789 -37.5,37.5c-0,20.711 16.789,37.5 37.5,37.5c20.711,0 37.5,-16.789 37.5,-37.5c0,-4.949 -0.959,-9.674 -2.701,-14Zm0,0l44.701,-8.5l28,65m0,0l-99,20l-18,47.5l15.5,37l-25,32.5l0,72l222.5,0l2.5,-72l-33.5,-117l-65,-20Zm-60,65l0,15m94,-13.5l0,13.5m-67.5,45l46,0l-12.5,50.5l-14.5,0l-19,-50.5Z"/></svg>	f	{"platform": false}
21	folia	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"></path><path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"></path></svg>	f	{"platform": false}
13	waterfall	<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"/></svg>	f	{"platform": true}
22	neoforge	<svg enable-background="new 0 0 24 24" version="1.1" viewBox="0 0 24 24" xml:space="preserve" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="m12 19.2v2m0-2v2"/><path d="m8.4 1.3c0.5 1.5 0.7 3 0.1 4.6-0.2 0.5-0.9 1.5-1.6 1.5m8.7-6.1c-0.5 1.5-0.7 3-0.1 4.6 0.2 0.6 0.9 1.5 1.6 1.5"/><path d="m3.6 15.8h-1.7m18.5 0h1.7"/><path d="m3.2 12.1h-1.7m19.3 0h1.8"/><path d="m8.1 12.7v1.6m7.8-1.6v1.6"/><path d="m10.8 18h1.2m0 1.2-1.2-1.2m2.4 0h-1.2m0 1.2 1.2-1.2"/><path d="m4 9.7c-0.5 1.2-0.8 2.4-0.8 3.7 0 3.1 2.9 6.3 5.3 8.2 0.9 0.7 2.2 1.1 3.4 1.1m0.1-17.8c-1.1 0-2.1 0.2-3.2 0.7m11.2 4.1c0.5 1.2 0.8 2.4 0.8 3.7 0 3.1-2.9 6.3-5.3 8.2-0.9 0.7-2.2 1.1-3.4 1.1m-0.1-17.8c1.1 0 2.1 0.2 3.2 0.7"/><path d="m4 9.7c-0.2-1.8-0.3-3.7 0.5-5.5s2.2-2.6 3.9-3m11.6 8.5c0.2-1.9 0.3-3.7-0.5-5.5s-2.2-2.6-3.9-3"/><path d="m12 21.2-2.4 0.4m2.4-0.4 2.4 0.4"/></g></svg>	f	{}
23	mrpack	<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>	f	{}
10	paper	<svg xml:space="preserve" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="1.5" clip-rule="evenodd" viewBox="0 0 24 24">\n  <path fill="none" d="M0 0h24v24H0z"/>\n  <path fill="none" stroke="currentColor" stroke-width="2" d="m12 18 6 2 3-17L2 14l6 2"/>\n  <path stroke="currentColor" stroke-width="2" d="m9 21-1-5 4 2-3 3Z"/>\n  <path fill="currentColor" d="m12 18-4-2 10-9-6 11Z"/>\n</svg>	f	{"platform": false}
11	purpur	<svg xml:space="preserve" fill-rule="evenodd" stroke-linecap="round" stroke-linejoin="round" stroke-miterlimit="1.5" clip-rule="evenodd" viewBox="0 0 24 24">\n  <defs>\n    <path id="purpur" fill="none" stroke="currentColor" stroke-width="1.68" d="m264 41.95 8-4v8l-8 4v-8Z"></path>\n  </defs>\n  <path fill="none" d="M0 0h24v24H0z"></path>\n  <path fill="none" stroke="currentColor" stroke-width="1.77" d="m264 29.95-8 4 8 4.42 8-4.42-8-4Z" transform="matrix(1.125 0 0 1.1372 -285 -31.69)"></path>\n  <path fill="none" stroke="currentColor" stroke-width="1.77" d="m272 38.37-8 4.42-8-4.42" transform="matrix(1.125 0 0 1.1372 -285 -31.69)"></path>\n  <path fill="none" stroke="currentColor" stroke-width="1.77" d="m260 31.95 8 4.21V45" transform="matrix(1.125 0 0 1.1372 -285 -31.69)"></path>\n  <path fill="none" stroke="currentColor" stroke-width="1.77" d="M260 45v-8.84l8-4.21" transform="matrix(1.125 0 0 1.1372 -285 -31.69)"></path>\n  <use xlink:href="#purpur" stroke-width="1.68" transform="matrix(1.125 0 0 1.2569 -285 -40.78)"></use>\n  <use xlink:href="#purpur" stroke-width="1.68" transform="matrix(-1.125 0 0 1.2569 309 -40.78)"></use>\n</svg>	f	{"platform": false}
8	bukkit	<svg viewBox="0 0 292 319" style="fill-rule:evenodd;clip-rule:evenodd;stroke-linecap:round;stroke-linejoin:round;" stroke="currentColor"><g transform="matrix(1,0,0,1,0,-5)"><path d="M12,109.5L12,155L34.5,224L57.5,224L57.5,271L81,294L160,294L160,172L259.087,172L265,155L265,109.5M12,109.5L12,64L34.5,64L34.5,41L81,17L195.5,17L241,41L241,64L265,64L265,109.5M12,109.5L81,109.5L81,132L195.5,132L195.5,109.5L265,109.5M264.087,204L264.087,244M207.5,272L207.5,312M250,272L250,312L280,312L280,272L250,272ZM192.5,204L192.5,244L222.5,244L222.5,204L192.5,204Z" style="fill:none;fill-rule:nonzero;stroke-width:24px;"/></g></svg>	f	{"platform": false}
12	bungeecord	<svg viewBox="0 0 24 24" version="1.1" xml:space="preserve" style="fill-rule:evenodd;clip-rule:evenodd;stroke-linecap:round;stroke-linejoin:round;">\n    <rect id="Bungeecord" x="-0" y="0" width="24" height="24" style="fill:none;"/>\n    <path d="M3.778,19.778C3.778,21.004 4.774,22 6,22C7.226,22 8.222,21.004 8.222,19.778L8.222,16.444C8.222,15.218 7.226,14.222 6,14.222L6,7.556C6,5.727 7.171,4.222 9,4.222C10.829,4.222 12,5.727 12,7.556L12,16.444" style="fill:none;fill-rule:nonzero;stroke:currentColor;stroke-width:2px;"/>\n    <path d="M7,15L6,13L5,15L7,15" style="fill:none;stroke:currentColor;stroke-width:2px;stroke-miterlimit:1.5;"/>\n    <path d="M20.222,4.444C20.222,3.218 19.226,2.222 18,2.222C16.774,2.222 15.778,3.218 15.778,4.444L15.778,7.778C15.778,9.004 16.774,10 18,10L18,16.667C18,18.495 16.829,20 15,20C13.171,20 12,18.495 12,16.667L12,7.778" style="fill:none;fill-rule:nonzero;stroke:currentColor;stroke-width:2px;"/>\n    <path d="M17,9.222L18,11.222L19,9.222L17,9.222" style="fill:none;stroke:currentColor;stroke-width:2px;stroke-miterlimit:1.5;"/>\n</svg>	f	{"platform": true}
14	velocity	<svg viewBox="0 0 500 500" fill="currentColor"><path d="M236.25 232.55l-54.08-73.79a11.86 11.86 0 00-11.91-4.62L84 171.57a11.88 11.88 0 00-8 5.88l-42.64 77.07a11.84 11.84 0 00.81 12.75l54.21 74a11.86 11.86 0 0011.91 4.62l86-17.37a11.85 11.85 0 008-5.89l42.78-77.3a11.86 11.86 0 00-.82-12.78zm-59.45 74.21a9.57 9.57 0 01-13.39-2.06l-31-42.24a16 16 0 00-16-6.21l-52.58 10.63a9.58 9.58 0 01-11.29-7.49A9.58 9.58 0 0160 248.1l57-11.52a16 16 0 0010.81-7.92L156.42 177a9.58 9.58 0 0113-3.75 9.58 9.58 0 013.75 13L146.81 234a16 16 0 001.09 17.16l31 42.23a9.58 9.58 0 01-2.1 13.37z"/><circle cx="416.44" cy="236.11" r="9.83"/><path d="M458.29 265.6H280.52a9.83 9.83 0 110-19.66h106.22a9.84 9.84 0 000-19.67h-70.2a9.83 9.83 0 110-19.66H422.9a9.84 9.84 0 000-19.67H202.83l33.42 45.61a11.86 11.86 0 01.81 12.75l-42.78 77.3a11.75 11.75 0 01-1.4 2h212.29a9.83 9.83 0 100-19.66h-53.53a9.84 9.84 0 110-19.67h106.65a9.84 9.84 0 100-19.67z"/></svg>	f	{"platform": true}
24	babric	<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12.35 5.89001L12.34 5.90001C10.59 7.37001 5.67003 11.13 2.64003 13.76C2.09003 14.23 1.97003 15.38 2.44003 15.93C4.24003 17.97 6.24003 20.2 6.89003 20.97C7.52003 21.69 8.91003 22.39 10 21.49C11.8 20 16.78 16.01 19.62 13.7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M19.59 13.66C19.34 13.33 18.02 11.53 19.05 10.25C19.52 9.63998 20.61 9.20998 21.3 9.98998C21.87 10.62 22.23 11.55 21.01 12.56C20.66 12.85 20.18 13.24 19.62 13.7C19.61 13.69 19.6 13.67 19.59 13.66Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M19.63 13.71L19.62 13.7C19.61 13.69 19.6 13.67 19.59 13.66C18.65 12.59 14.44 8.13999 12.34 5.89999C11.76 5.28999 11.33 4.83999 11.18 4.66999C10.91 4.36999 9.91004 3.64999 11.63 2.46999C12.98 1.54999 13.48 1.97999 13.88 2.37999L21.3 9.97999" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M10.6352 14.7743C10.7992 14.517 10.8811 14.2194 10.8811 13.8978C10.8811 12 9.2582 12 8.06967 12C7.61886 12 7.25 12.3538 7.25 12.8041V17.1948C7.25 17.637 7.61886 17.9989 8.06967 17.9989C8.1552 17.9965 9.54283 18.0068 9.57789 17.9909C10.3894 17.9668 11.25 17.6531 11.25 15.9886C11.25 15.7232 11.1762 15.1925 10.6352 14.7743ZM9.61886 13.8978C9.61886 14.0506 9.53686 14.1953 9.40574 14.2918H9.39753C9.29097 14.3722 9.15164 14.4205 9.00411 14.4205H8.47951V13.3831C8.56539 13.3831 8.92616 13.3832 9.00411 13.3831C9.34015 13.3831 9.61886 13.6163 9.61886 13.8978ZM9.29918 16.7927H8.47951V15.7554C8.68401 15.7555 9.09478 15.7553 9.29918 15.7554C9.71537 15.7517 10.0548 16.1404 9.85655 16.4871C9.7664 16.6641 9.54507 16.7927 9.29918 16.7927Z" fill="currentColor" stroke="currentColor" stroke-width="0.25"/></svg>	f	{}
25	bta-babric	<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12.35 5.89001L12.34 5.90001C10.59 7.37001 5.67003 11.13 2.64003 13.76C2.09003 14.23 1.97003 15.38 2.44003 15.93C4.24003 17.97 6.24003 20.2 6.89003 20.97C7.52003 21.69 8.91003 22.39 10 21.49C11.8 20 16.78 16.01 19.62 13.7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M19.59 13.66C19.34 13.33 18.02 11.53 19.05 10.25C19.52 9.63998 20.61 9.20998 21.3 9.98998C21.87 10.62 22.23 11.55 21.01 12.56C20.66 12.85 20.18 13.24 19.62 13.7C19.61 13.69 19.6 13.67 19.59 13.66Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M19.63 13.71L19.62 13.7C19.61 13.69 19.6 13.67 19.59 13.66C18.65 12.59 14.44 8.13999 12.34 5.89999C11.76 5.28999 11.33 4.83999 11.18 4.66999C10.91 4.36999 9.91004 3.64999 11.63 2.46999C12.98 1.54999 13.48 1.97999 13.88 2.37999L21.3 9.97999" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M10.6352 14.7743C10.7992 14.517 10.8811 14.2194 10.8811 13.8978C10.8811 12 9.2582 12 8.06967 12C7.61886 12 7.25 12.3538 7.25 12.8041V17.1948C7.25 17.637 7.61886 17.9989 8.06967 17.9989C8.1552 17.9965 9.54283 18.0068 9.57789 17.9909C10.3894 17.9668 11.25 17.6531 11.25 15.9886C11.25 15.7232 11.1762 15.1925 10.6352 14.7743ZM9.61886 13.8978C9.61886 14.0506 9.53686 14.1953 9.40574 14.2918H9.39753C9.29097 14.3722 9.15164 14.4205 9.00411 14.4205H8.47951V13.3831C8.56539 13.3831 8.92616 13.3832 9.00411 13.3831C9.34015 13.3831 9.61886 13.6163 9.61886 13.8978ZM9.29918 16.7927H8.47951V15.7554C8.68401 15.7555 9.09478 15.7553 9.29918 15.7554C9.71537 15.7517 10.0548 16.1404 9.85655 16.4871C9.7664 16.6641 9.54507 16.7927 9.29918 16.7927Z" fill="currentColor" stroke="currentColor" stroke-width="0.25"/><path d="M13.2991 11V14" stroke="currentColor" stroke-linecap="round"/><path d="M12 13.25L14.5981 11.75" stroke="currentColor" stroke-linecap="round"/><path d="M14.5981 13.25L12.0001 11.75" stroke="currentColor" stroke-linecap="round"/></svg>	f	{}
26	java-agent	<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M18 2L22 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M17 7L20 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M19 9L8.69995 19.3C7.69995 20.3 6.19995 20.3 5.29995 19.3L4.69995 18.7C3.69995 17.7 3.69995 16.2 4.69995 15.3L15 5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M9 11L13 15" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M5 19L2 22" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M14 4L20 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>	f	{}
27	legacy-fabric	<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_6351_12952)"><path d="M21.3022 9.9787L13.8798 2.38379C13.4809 1.9763 12.978 1.55147 11.634 2.47049C9.90847 3.64961 10.9056 4.36921 11.1831 4.67266C11.8941 5.45296 18.4754 12.389 19.6113 13.6895C19.8281 13.9322 17.8511 11.7387 19.0477 10.2475C19.5159 9.64057 20.6085 9.20707 21.3022 9.98737C21.8658 10.6203 22.23 11.548 21.0074 12.5624C18.8656 14.331 12.1629 19.7064 9.99518 21.4925C8.91131 22.3855 7.52395 21.6919 6.89096 20.9723C6.24064 20.2006 4.23764 17.9724 2.44274 15.9263C1.96584 15.3801 2.08723 14.227 2.64218 13.7588C5.67703 11.1318 10.6108 7.36036 12.345 5.88646" stroke="currentColor" stroke-width="1.99422" stroke-linecap="round" stroke-linejoin="round"/><path d="M8 13V17H10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M8 13V17H10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></g><defs><clipPath id="clip0_6351_12952"><rect width="24" height="24" fill="white"/></clipPath></defs></svg>	f	{}
28	nilloader	<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><ellipse cx="12" cy="11" rx="5" ry="8" stroke="currentColor" stroke-width="2"/><path d="M16.563 2.72485L6.75577 19.7114L12.3865 22.9624" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>	f	{}
29	ornithe	<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M8 7H7.99" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M20.6 18H12C9.87827 18 7.84344 17.1572 6.34315 15.6569C4.84285 14.1566 4 12.1217 4 10V7.00001C3.99775 6.14792 4.26766 5.31737 4.7704 4.6294C5.27315 3.94142 5.98245 3.43197 6.79496 3.17527C7.60747 2.91857 8.48072 2.92804 9.28746 3.2023C10.0942 3.47657 10.7923 4.00129 11.28 4.70001L22 20" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M4 7L2 7.5L4 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M14 18V21" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M10 17.75V21" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M17 18C15.7669 18 14.5637 17.62 13.5543 16.9117C12.5448 16.2035 11.7781 15.2014 11.3584 14.0419C10.9388 12.8824 10.8866 11.6218 11.2089 10.4315C11.5313 9.24128 12.2126 8.17927 13.16 7.39001" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>	f	{}
\.


TRUNCATE table public.loader_field_enum_values RESTART IDENTITY CASCADE;
COPY public.loader_field_enum_values (id, enum_id, value, ordering, created, metadata) FROM stdin;
2	1	unsupported	\N	2025-04-03 21:55:46.229944+00	\N
3	1	unknown	\N	2025-04-03 21:55:46.229944+00	\N
4	1	optional	\N	2025-04-03 21:55:46.229944+00	\N
691	2	1.7.2	\N	2013-10-25 13:00:00+00	{"type": "release", "major": false}
692	2	1.7.1	\N	2013-10-23 12:01:07+00	{"type": "snapshot", "major": false}
693	2	1.7	\N	2013-10-22 15:04:05+00	{"type": "snapshot", "major": false}
694	2	13w43a	\N	2013-10-21 16:34:47+00	{"type": "snapshot", "major": false}
695	2	13w42b	\N	2013-10-18 16:34:08+00	{"type": "snapshot", "major": false}
17005268	4	client_and_server	\N	2025-06-19 09:41:54.646734+00	\N
17005269	4	client_only	\N	2025-06-19 09:41:54.646734+00	\N
17005270	4	client_only_server_optional	\N	2025-06-19 09:41:54.646734+00	\N
696	2	13w42a	\N	2013-10-17 18:33:05+00	{"type": "snapshot", "major": false}
697	2	13w41b	\N	2013-10-11 15:09:17+00	{"type": "snapshot", "major": false}
698	2	13w41a	\N	2013-10-10 14:21:43+00	{"type": "snapshot", "major": false}
699	2	13w39b	\N	2013-09-27 12:15:58+00	{"type": "snapshot", "major": false}
700	2	13w39a	\N	2013-09-26 15:11:19+00	{"type": "snapshot", "major": false}
701	2	13w38c	\N	2013-09-20 15:11:34+00	{"type": "snapshot", "major": false}
17005271	4	singleplayer_only	\N	2025-06-19 09:41:54.646734+00	\N
17005272	4	server_only	\N	2025-06-19 09:41:54.646734+00	\N
17005273	4	server_only_client_optional	\N	2025-06-19 09:41:54.646734+00	\N
17005274	4	dedicated_server_only	\N	2025-06-19 09:41:54.646734+00	\N
17005275	4	client_or_server	\N	2025-06-19 09:41:54.646734+00	\N
17005276	4	client_or_server_prefers_both	\N	2025-06-19 09:41:54.646734+00	\N
17005277	4	unknown	\N	2025-06-19 09:41:54.646734+00	\N
702	2	13w38b	\N	2013-09-20 13:45:40+00	{"type": "snapshot", "major": false}
703	2	13w38a	\N	2013-09-19 16:34:21+00	{"type": "snapshot", "major": false}
20988520	2	25w32a	\N	2025-08-05 12:21:45+00	{"type": "snapshot", "major": false}
19322115	2	25w31a	\N	2025-07-29 11:29:33+00	{"type": "snapshot", "major": false}
704	2	1.6.4	\N	2013-09-19 15:52:37+00	{"type": "release", "major": false}
706	2	13w37b	\N	2013-09-13 10:54:42+00	{"type": "snapshot", "major": false}
17019240	2	1.21.7-rc1	\N	2025-06-25 12:41:59+00	{"type": "snapshot", "major": false}
8	2	1.21.5-rc1	\N	2025-03-20 13:45:48+00	{"type": "snapshot", "major": false}
224	2	21w39a	\N	2021-09-29 16:27:05+00	{"type": "snapshot", "major": false}
231	2	1.17.1-pre2	\N	2021-06-29 15:14:12+00	{"type": "snapshot", "major": false}
264	2	20w45a	\N	2020-11-04 16:42:00+00	{"type": "snapshot", "major": false}
303	2	20w13b	\N	2020-03-26 13:00:34+00	{"type": "snapshot", "major": false}
705	2	1.6.3	\N	2013-09-13 10:54:41+00	{"type": "snapshot", "major": false}
312	2	1.15.2	\N	2020-01-17 10:03:52+00	{"type": "release", "major": false}
357	2	1.14.2-pre1	\N	2019-05-16 15:40:25+00	{"type": "snapshot", "major": false}
418	2	1.13-pre4	\N	2018-06-26 13:00:55+00	{"type": "snapshot", "major": false}
437	2	18w10d	\N	2018-03-09 15:19:12+00	{"type": "snapshot", "major": false}
447	2	18w06a	\N	2018-02-09 12:09:55+00	{"type": "snapshot", "major": false}
600	2	1.8.2-pre7	\N	2015-02-16 13:01:35+00	{"type": "snapshot", "major": false}
602	2	1.8.2-pre5	\N	2015-01-26 15:03:24+00	{"type": "snapshot", "major": false}
669	2	14w07a	\N	2014-02-14 11:05:07+00	{"type": "snapshot", "major": false}
671	2	14w06a	\N	2014-02-06 14:30:17+00	{"type": "snapshot", "major": false}
681	2	1.7.4	\N	2013-12-09 12:28:10+00	{"type": "release", "major": false}
682	2	1.7.3	\N	2013-12-06 13:55:34+00	{"type": "release", "major": false}
683	2	13w49a	\N	2013-12-05 14:34:41+00	{"type": "snapshot", "major": false}
688	2	13w47c	\N	2013-11-21 17:10:33+00	{"type": "snapshot", "major": false}
689	2	13w47b	\N	2013-11-21 16:57:41+00	{"type": "snapshot", "major": false}
225	2	21w38a	\N	2021-09-23 14:36:06+00	{"type": "snapshot", "major": false}
438	2	18w10c	\N	2018-03-08 15:29:23+00	{"type": "snapshot", "major": false}
472	2	1.12-pre6	\N	2017-05-29 11:45:12+00	{"type": "snapshot", "major": false}
684	2	13w48b	\N	2013-11-26 18:36:08+00	{"type": "snapshot", "major": false}
685	2	13w48a	\N	2013-11-25 16:53:39+00	{"type": "snapshot", "major": false}
686	2	13w47e	\N	2013-11-22 15:16:38+00	{"type": "snapshot", "major": false}
687	2	13w47d	\N	2013-11-22 13:51:15+00	{"type": "snapshot", "major": false}
743	2	1.3.1	\N	2012-07-31 22:00:00+00	{"type": "release", "major": false}
744	2	1.3	\N	2012-07-25 22:00:00+00	{"type": "snapshot", "major": false}
746	2	1.2.4	\N	2012-03-21 22:00:00+00	{"type": "release", "major": false}
747	2	1.2.3	\N	2012-03-01 22:00:00+00	{"type": "release", "major": false}
749	2	1.2.1	\N	2012-02-29 22:00:00+00	{"type": "release", "major": false}
750	2	1.1	\N	2012-01-11 22:00:00+00	{"type": "release", "major": false}
752	2	b1.8.1	\N	2011-09-18 22:00:00+00	{"type": "beta", "major": false}
753	2	b1.8	\N	2011-09-14 22:00:00+00	{"type": "beta", "major": false}
755	2	b1.7.2	\N	2011-06-30 22:00:00+00	{"type": "beta", "major": false}
756	2	b1.7	\N	2011-06-29 22:00:00+00	{"type": "beta", "major": false}
758	2	b1.6.5	\N	2011-05-27 22:00:00+00	{"type": "beta", "major": false}
707	2	13w37a	\N	2013-09-12 14:23:14+00	{"type": "snapshot", "major": false}
709	2	13w36a	\N	2013-09-05 13:05:40+00	{"type": "snapshot", "major": false}
712	2	1.6	\N	2013-06-25 13:08:56+00	{"type": "snapshot", "major": false}
715	2	13w25b	\N	2013-06-18 15:13:27+00	{"type": "snapshot", "major": false}
718	2	13w24a	\N	2013-06-13 15:32:23+00	{"type": "snapshot", "major": false}
721	2	13w22a	\N	2013-05-30 14:38:40+00	{"type": "snapshot", "major": false}
724	2	13w19a	\N	2013-05-10 14:48:02+00	{"type": "snapshot", "major": false}
727	2	13w18a	\N	2013-05-02 15:45:59+00	{"type": "snapshot", "major": false}
741	2	1.4	\N	2012-11-18 22:00:00+00	{"type": "snapshot", "major": false}
759	2	b1.6.4	\N	2011-05-25 22:00:04+00	{"type": "beta", "major": false}
761	2	b1.6.2	\N	2011-05-25 22:00:02+00	{"type": "beta", "major": false}
762	2	b1.6.1	\N	2011-05-25 22:00:01+00	{"type": "beta", "major": false}
764	2	b1.5_01	\N	2011-04-19 22:00:00+00	{"type": "beta", "major": false}
765	2	b1.5	\N	2011-04-18 22:00:00+00	{"type": "beta", "major": false}
79	2	24w09a	\N	2024-02-28 12:38:12+00	{"type": "snapshot", "major": false}
133	2	23w12a	\N	2023-03-22 13:28:42+00	{"type": "snapshot", "major": false}
112	2	23w35a	\N	2023-08-30 11:24:35+00	{"type": "snapshot", "major": false}
82	2	24w05b	\N	2024-02-01 12:55:14+00	{"type": "snapshot", "major": false}
85	2	24w03b	\N	2024-01-18 12:42:37+00	{"type": "snapshot", "major": false}
88	2	23w51a	\N	2023-12-18 13:36:46+00	{"type": "snapshot", "major": false}
91	2	1.20.3	\N	2023-12-04 12:10:32+00	{"type": "release", "major": false}
94	2	1.20.3-pre3	\N	2023-11-27 14:24:36+00	{"type": "snapshot", "major": false}
97	2	23w46a	\N	2023-11-16 14:11:33+00	{"type": "snapshot", "major": false}
100	2	23w43b	\N	2023-10-26 13:46:16+00	{"type": "snapshot", "major": false}
103	2	23w41a	\N	2023-10-11 12:32:46+00	{"type": "snapshot", "major": false}
106	2	1.20.2-rc2	\N	2023-09-18 12:34:57+00	{"type": "snapshot", "major": false}
109	2	1.20.2-pre3	\N	2023-09-12 12:15:08+00	{"type": "snapshot", "major": false}
115	2	23w31a	\N	2023-08-01 10:03:13+00	{"type": "snapshot", "major": false}
118	2	1.20	\N	2023-06-02 08:36:17+00	{"type": "release", "major": false}
124	2	1.20-pre3	\N	2023-05-19 11:39:46+00	{"type": "snapshot", "major": false}
127	2	23w18a	\N	2023-05-03 11:29:26+00	{"type": "snapshot", "major": false}
136	2	1.19.4-rc2	\N	2023-03-10 12:42:54+00	{"type": "snapshot", "major": false}
139	2	1.19.4-pre3	\N	2023-03-01 14:11:05+00	{"type": "snapshot", "major": false}
142	2	23w07a	\N	2023-02-15 14:32:29+00	{"type": "snapshot", "major": false}
145	2	23w04a	\N	2023-01-24 15:19:06+00	{"type": "snapshot", "major": false}
148	2	1.19.3-rc3	\N	2022-12-06 10:24:01+00	{"type": "snapshot", "major": false}
151	2	1.19.3-pre3	\N	2022-11-29 14:28:08+00	{"type": "snapshot", "major": false}
154	2	22w46a	\N	2022-11-16 13:32:50+00	{"type": "snapshot", "major": false}
157	2	22w43a	\N	2022-10-26 11:55:59+00	{"type": "snapshot", "major": false}
160	2	1.19.2-rc2	\N	2022-08-04 15:19:44+00	{"type": "snapshot", "major": false}
163	2	1.19.1-rc3	\N	2022-07-26 15:34:35+00	{"type": "snapshot", "major": false}
166	2	1.19.1-pre5	\N	2022-07-15 11:51:44+00	{"type": "snapshot", "major": false}
169	2	1.19.1-pre2	\N	2022-06-30 15:57:20+00	{"type": "snapshot", "major": false}
175	2	1.19-rc1	\N	2022-06-02 12:12:52+00	{"type": "snapshot", "major": false}
178	2	1.19-pre3	\N	2022-05-25 09:56:47+00	{"type": "snapshot", "major": false}
181	2	22w19a	\N	2022-05-12 15:36:11+00	{"type": "snapshot", "major": false}
184	2	22w16b	\N	2022-04-20 17:25:32+00	{"type": "snapshot", "major": false}
187	2	22w14a	\N	2022-04-06 13:37:12+00	{"type": "snapshot", "major": false}
190	2	22w12a	\N	2022-03-24 16:15:02+00	{"type": "snapshot", "major": false}
193	2	1.18.2-rc1	\N	2022-02-25 13:25:40+00	{"type": "snapshot", "major": false}
199	2	22w05a	\N	2022-02-02 16:08:39+00	{"type": "snapshot", "major": false}
202	2	1.18.1-rc3	\N	2021-12-10 03:36:38+00	{"type": "snapshot", "major": false}
205	2	1.18.1-pre1	\N	2021-12-03 13:45:38+00	{"type": "snapshot", "major": false}
208	2	1.18-rc3	\N	2021-11-26 15:51:56+00	{"type": "snapshot", "major": false}
17022532	2	1.21.8-rc1	\N	2025-07-15 13:42:50+00	{"type": "snapshot", "major": false}
17022533	2	1.21.7	\N	2025-06-30 09:32:16+00	{"type": "release", "major": false}
17019239	2	1.21.7-rc2	\N	2025-06-26 13:59:20+00	{"type": "snapshot", "major": false}
16819722	2	1.21.6	\N	2025-06-17 11:10:28+00	{"type": "release", "major": false}
15475421	2	1.21.6-pre4	\N	2025-06-10 12:22:36+00	{"type": "snapshot", "major": false}
13610951	2	1.21.6-pre2	\N	2025-06-02 13:40:47+00	{"type": "snapshot", "major": false}
10555651	2	25w21a	\N	2025-05-20 12:09:09+00	{"type": "snapshot", "major": false}
7275618	2	25w19a	\N	2025-05-06 12:57:57+00	{"type": "snapshot", "major": false}
4002026	2	25w17a	\N	2025-04-22 12:51:30+00	{"type": "snapshot", "major": false}
10	2	1.21.5-pre2	\N	2025-03-12 12:36:02+00	{"type": "snapshot", "major": false}
14	2	25w09a	\N	2025-02-26 15:16:02+00	{"type": "snapshot", "major": false}
16	2	25w07a	\N	2025-02-13 12:55:37+00	{"type": "snapshot", "major": false}
18	2	25w05a	\N	2025-01-29 14:03:54+00	{"type": "snapshot", "major": false}
20	2	25w03a	\N	2025-01-15 14:28:04+00	{"type": "snapshot", "major": false}
22	2	1.21.4	\N	2024-12-03 10:12:57+00	{"type": "release", "major": false}
24	2	1.21.4-rc2	\N	2024-11-29 10:33:13+00	{"type": "snapshot", "major": false}
26	2	1.21.4-pre3	\N	2024-11-26 15:07:29+00	{"type": "snapshot", "major": false}
28	2	1.21.4-pre1	\N	2024-11-20 13:45:00+00	{"type": "snapshot", "major": false}
30	2	24w45a	\N	2024-11-06 13:31:58+00	{"type": "snapshot", "major": false}
32	2	1.21.3	\N	2024-10-23 12:28:15+00	{"type": "release", "major": false}
34	2	1.21.2-rc2	\N	2024-10-21 15:53:05+00	{"type": "snapshot", "major": false}
36	2	1.21.2-pre5	\N	2024-10-16 13:30:35+00	{"type": "snapshot", "major": false}
38	2	1.21.2-pre3	\N	2024-10-11 12:32:27+00	{"type": "snapshot", "major": false}
40	2	1.21.2-pre1	\N	2024-10-08 13:22:12+00	{"type": "snapshot", "major": false}
42	2	24w39a	\N	2024-09-25 13:08:41+00	{"type": "snapshot", "major": false}
44	2	24w37a	\N	2024-09-11 13:01:31+00	{"type": "snapshot", "major": false}
46	2	24w35a	\N	2024-08-28 12:25:10+00	{"type": "snapshot", "major": false}
48	2	24w33a	\N	2024-08-15 12:39:34+00	{"type": "snapshot", "major": false}
52	2	1.21-rc1	\N	2024-06-10 12:24:08+00	{"type": "snapshot", "major": false}
54	2	1.21-pre3	\N	2024-06-05 08:51:44+00	{"type": "snapshot", "major": false}
56	2	1.21-pre1	\N	2024-05-29 12:04:43+00	{"type": "snapshot", "major": false}
61	2	24w19a	\N	2024-05-10 12:15:31+00	{"type": "snapshot", "major": false}
64	2	1.20.6-rc1	\N	2024-04-26 10:12:17+00	{"type": "snapshot", "major": false}
67	2	1.20.5-rc2	\N	2024-04-19 13:13:15+00	{"type": "snapshot", "major": false}
70	2	1.20.5-pre3	\N	2024-04-16 11:57:30+00	{"type": "snapshot", "major": false}
73	2	24w14a	\N	2024-04-03 11:49:39+00	{"type": "snapshot", "major": false}
76	2	24w12a	\N	2024-03-20 14:38:37+00	{"type": "snapshot", "major": false}
130	2	23w14a	\N	2023-04-05 12:05:17+00	{"type": "snapshot", "major": false}
172	2	22w24a	\N	2022-06-15 16:21:49+00	{"type": "snapshot", "major": false}
8913813	2	25w20a	\N	2025-05-13 11:46:28+00	{"type": "snapshot", "major": false}
788	2	a1.2.1	\N	2010-11-04 22:00:00+00	{"type": "alpha", "major": false}
15947165	2	1.21.6-rc1	\N	2025-06-12 12:04:37+00	{"type": "snapshot", "major": false}
12391031	2	1.21.6-pre1	\N	2025-05-28 09:34:04+00	{"type": "snapshot", "major": false}
596	2	1.8.4	\N	2015-04-17 11:37:50+00	{"type": "release", "major": false}
584	2	15w33b	\N	2015-08-12 15:29:11+00	{"type": "snapshot", "major": false}
587	2	15w32b	\N	2015-08-06 13:51:47+00	{"type": "snapshot", "major": false}
597	2	15w14a	\N	2015-04-01 07:08:00+00	{"type": "snapshot", "major": false}
599	2	1.8.2	\N	2015-02-19 15:47:29+00	{"type": "release", "major": false}
780	2	a1.2.4_01	\N	2010-11-29 22:00:00+00	{"type": "alpha", "major": false}
782	2	a1.2.3_02	\N	2010-11-24 22:00:00+00	{"type": "alpha", "major": false}
783	2	a1.2.3_01	\N	2010-11-23 22:00:01+00	{"type": "alpha", "major": false}
785	2	a1.2.2b	\N	2010-11-09 22:00:01+00	{"type": "alpha", "major": false}
786	2	a1.2.2a	\N	2010-11-09 22:00:00+00	{"type": "alpha", "major": false}
14076641	2	1.21.6-pre3	\N	2025-06-04 13:33:25+00	{"type": "snapshot", "major": false}
5637002	2	25w18a	\N	2025-04-29 12:21:01+00	{"type": "snapshot", "major": false}
2369876	2	25w16a	\N	2025-04-15 12:01:58+00	{"type": "snapshot", "major": false}
1083550	2	25w15a	\N	2025-04-08 12:16:59+00	{"type": "snapshot", "major": false}
5	2	25w14craftmine	\N	2025-04-01 15:50:09+00	{"type": "snapshot", "major": false}
7	2	1.21.5-rc2	\N	2025-03-24 13:07:03+00	{"type": "snapshot", "major": false}
9	2	1.21.5-pre3	\N	2025-03-18 13:58:30+00	{"type": "snapshot", "major": false}
13	2	25w09b	\N	2025-02-27 11:07:08+00	{"type": "snapshot", "major": false}
15	2	25w08a	\N	2025-02-19 13:41:43+00	{"type": "snapshot", "major": false}
17	2	25w06a	\N	2025-02-05 12:41:17+00	{"type": "snapshot", "major": false}
19	2	25w04a	\N	2025-01-22 13:14:44+00	{"type": "snapshot", "major": false}
21	2	25w02a	\N	2025-01-08 13:42:18+00	{"type": "snapshot", "major": false}
23	2	1.21.4-rc3	\N	2024-11-29 17:02:53+00	{"type": "snapshot", "major": false}
27	2	1.21.4-pre2	\N	2024-11-25 13:18:35+00	{"type": "snapshot", "major": false}
29	2	24w46a	\N	2024-11-13 13:12:38+00	{"type": "snapshot", "major": false}
31	2	24w44a	\N	2024-10-30 12:53:55+00	{"type": "snapshot", "major": false}
33	2	1.21.2	\N	2024-10-22 09:58:55+00	{"type": "release", "major": false}
35	2	1.21.2-rc1	\N	2024-10-17 12:43:18+00	{"type": "snapshot", "major": false}
37	2	1.21.2-pre4	\N	2024-10-15 11:59:11+00	{"type": "snapshot", "major": false}
39	2	1.21.2-pre2	\N	2024-10-10 12:59:14+00	{"type": "snapshot", "major": false}
41	2	24w40a	\N	2024-10-02 13:15:42+00	{"type": "snapshot", "major": false}
43	2	24w38a	\N	2024-09-18 12:32:07+00	{"type": "snapshot", "major": false}
45	2	24w36a	\N	2024-09-04 12:44:12+00	{"type": "snapshot", "major": false}
47	2	24w34a	\N	2024-08-21 14:14:13+00	{"type": "snapshot", "major": false}
49	2	1.21.1	\N	2024-08-08 12:24:45+00	{"type": "release", "major": false}
51	2	1.21	\N	2024-06-13 08:24:03+00	{"type": "release", "major": false}
53	2	1.21-pre4	\N	2024-06-07 12:00:15+00	{"type": "snapshot", "major": false}
58	2	24w21a	\N	2024-05-22 14:18:26+00	{"type": "snapshot", "major": false}
59	2	24w20a	\N	2024-05-15 12:00:35+00	{"type": "snapshot", "major": false}
60	2	24w19b	\N	2024-05-10 14:32:42+00	{"type": "snapshot", "major": false}
62	2	24w18a	\N	2024-05-03 12:08:27+00	{"type": "snapshot", "major": false}
546	2	15w49b	\N	2015-12-03 15:23:22+00	{"type": "snapshot", "major": false}
548	2	15w49a	\N	2015-12-02 15:09:37+00	{"type": "snapshot", "major": false}
549	2	15w47c	\N	2015-11-20 12:46:56+00	{"type": "snapshot", "major": false}
551	2	15w47a	\N	2015-11-18 15:53:41+00	{"type": "snapshot", "major": false}
552	2	15w46a	\N	2015-11-12 12:11:47+00	{"type": "snapshot", "major": false}
554	2	15w44b	\N	2015-10-30 11:23:17+00	{"type": "snapshot", "major": false}
555	2	15w44a	\N	2015-10-28 15:09:36+00	{"type": "snapshot", "major": false}
558	2	15w43a	\N	2015-10-21 15:28:52+00	{"type": "snapshot", "major": false}
560	2	15w41b	\N	2015-10-07 14:07:26+00	{"type": "snapshot", "major": false}
561	2	15w41a	\N	2015-10-07 13:19:53+00	{"type": "snapshot", "major": false}
563	2	15w40a	\N	2015-09-30 13:13:54+00	{"type": "snapshot", "major": false}
564	2	15w39c	\N	2015-09-23 13:13:54+00	{"type": "snapshot", "major": false}
566	2	15w39a	\N	2015-09-21 13:16:32+00	{"type": "snapshot", "major": false}
567	2	15w38b	\N	2015-09-17 14:22:31+00	{"type": "snapshot", "major": false}
569	2	15w37a	\N	2015-09-10 14:22:31+00	{"type": "snapshot", "major": false}
570	2	15w36d	\N	2015-09-04 14:22:31+00	{"type": "snapshot", "major": false}
572	2	15w36b	\N	2015-09-02 15:36:25+00	{"type": "snapshot", "major": false}
573	2	15w36a	\N	2015-09-02 14:46:40+00	{"type": "snapshot", "major": false}
575	2	15w35d	\N	2015-08-28 16:25:35+00	{"type": "snapshot", "major": false}
576	2	15w35c	\N	2015-08-28 11:21:00+00	{"type": "snapshot", "major": false}
578	2	15w35a	\N	2015-08-24 14:19:31+00	{"type": "snapshot", "major": false}
582	2	15w34a	\N	2015-08-19 12:56:01+00	{"type": "snapshot", "major": false}
588	2	15w32a	\N	2015-08-05 12:22:42+00	{"type": "snapshot", "major": false}
590	2	15w31b	\N	2015-07-30 13:38:32+00	{"type": "snapshot", "major": false}
591	2	15w31a	\N	2015-07-29 13:24:33+00	{"type": "snapshot", "major": false}
593	2	1.8.7	\N	2015-06-05 10:10:44+00	{"type": "release", "major": false}
25	2	1.21.4-rc1	\N	2024-11-28 10:19:01+00	{"type": "snapshot", "major": false}
579	2	15w34d	\N	2015-08-21 15:27:55+00	{"type": "snapshot", "major": false}
581	2	15w34b	\N	2015-08-20 14:00:03+00	{"type": "snapshot", "major": false}
585	2	15w33a	\N	2015-08-12 14:05:07+00	{"type": "snapshot", "major": false}
6	2	1.21.5	\N	2025-03-25 12:14:58+00	{"type": "release", "major": false}
55	2	1.21-pre2	\N	2024-05-31 12:44:56+00	{"type": "snapshot", "major": false}
557	2	15w43b	\N	2015-10-22 14:11:58+00	{"type": "snapshot", "major": false}
57	2	24w21b	\N	2024-05-22 16:25:41+00	{"type": "snapshot", "major": false}
63	2	1.20.6	\N	2024-04-29 12:40:45+00	{"type": "release", "major": false}
65	2	1.20.5	\N	2024-04-23 11:54:12+00	{"type": "release", "major": false}
66	2	1.20.5-rc3	\N	2024-04-22 13:42:34+00	{"type": "snapshot", "major": false}
68	2	1.20.5-rc1	\N	2024-04-18 11:45:40+00	{"type": "snapshot", "major": false}
69	2	1.20.5-pre4	\N	2024-04-17 11:56:02+00	{"type": "snapshot", "major": false}
71	2	1.20.5-pre2	\N	2024-04-15 12:36:05+00	{"type": "snapshot", "major": false}
72	2	1.20.5-pre1	\N	2024-04-10 12:44:25+00	{"type": "snapshot", "major": false}
74	2	24w14potato	\N	2024-04-01 11:07:19+00	{"type": "snapshot", "major": false}
75	2	24w13a	\N	2024-03-27 14:30:20+00	{"type": "snapshot", "major": false}
77	2	24w11a	\N	2024-03-14 14:21:33+00	{"type": "snapshot", "major": false}
78	2	24w10a	\N	2024-03-06 10:37:35+00	{"type": "snapshot", "major": false}
80	2	24w07a	\N	2024-02-14 12:51:01+00	{"type": "snapshot", "major": false}
81	2	24w06a	\N	2024-02-07 14:47:18+00	{"type": "snapshot", "major": false}
83	2	24w05a	\N	2024-01-31 13:05:26+00	{"type": "snapshot", "major": false}
84	2	24w04a	\N	2024-01-24 13:42:45+00	{"type": "snapshot", "major": false}
99	2	23w44a	\N	2023-11-01 12:30:52+00	{"type": "snapshot", "major": false}
101	2	23w43a	\N	2023-10-25 13:34:37+00	{"type": "snapshot", "major": false}
102	2	23w42a	\N	2023-10-18 11:37:28+00	{"type": "snapshot", "major": false}
104	2	23w40a	\N	2023-10-04 12:48:53+00	{"type": "snapshot", "major": false}
105	2	1.20.2	\N	2023-09-20 09:02:57+00	{"type": "release", "major": false}
107	2	1.20.2-rc1	\N	2023-09-15 13:10:30+00	{"type": "snapshot", "major": false}
108	2	1.20.2-pre4	\N	2023-09-13 15:06:51+00	{"type": "snapshot", "major": false}
110	2	1.20.2-pre2	\N	2023-09-07 12:42:32+00	{"type": "snapshot", "major": false}
111	2	1.20.2-pre1	\N	2023-09-05 12:06:20+00	{"type": "snapshot", "major": false}
113	2	23w33a	\N	2023-08-17 11:39:08+00	{"type": "snapshot", "major": false}
114	2	23w32a	\N	2023-08-09 12:14:25+00	{"type": "snapshot", "major": false}
116	2	1.20.1	\N	2023-06-12 13:25:51+00	{"type": "release", "major": false}
117	2	1.20.1-rc1	\N	2023-06-09 14:15:49+00	{"type": "snapshot", "major": false}
119	2	1.20-rc1	\N	2023-05-31 12:33:33+00	{"type": "snapshot", "major": false}
120	2	1.20-pre7	\N	2023-05-29 13:44:34+00	{"type": "snapshot", "major": false}
122	2	1.20-pre5	\N	2023-05-23 12:22:52+00	{"type": "snapshot", "major": false}
123	2	1.20-pre4	\N	2023-05-19 13:13:45+00	{"type": "snapshot", "major": false}
125	2	1.20-pre2	\N	2023-05-16 11:34:54+00	{"type": "snapshot", "major": false}
126	2	1.20-pre1	\N	2023-05-10 12:19:34+00	{"type": "snapshot", "major": false}
128	2	23w17a	\N	2023-04-26 12:09:48+00	{"type": "snapshot", "major": false}
129	2	23w16a	\N	2023-04-20 11:55:19+00	{"type": "snapshot", "major": false}
131	2	23w13a_or_b	\N	2023-04-01 12:52:18+00	{"type": "snapshot", "major": false}
132	2	23w13a	\N	2023-03-29 13:54:16+00	{"type": "snapshot", "major": false}
134	2	1.19.4	\N	2023-03-14 12:56:18+00	{"type": "release", "major": false}
86	2	24w03a	\N	2024-01-17 13:19:20+00	{"type": "snapshot", "major": false}
87	2	23w51b	\N	2023-12-18 15:39:14+00	{"type": "snapshot", "major": false}
89	2	1.20.4	\N	2023-12-07 12:56:20+00	{"type": "release", "major": false}
90	2	1.20.4-rc1	\N	2023-12-06 14:38:01+00	{"type": "snapshot", "major": false}
92	2	1.20.3-rc1	\N	2023-11-30 13:41:45+00	{"type": "snapshot", "major": false}
93	2	1.20.3-pre4	\N	2023-11-28 13:47:32+00	{"type": "snapshot", "major": false}
95	2	1.20.3-pre2	\N	2023-11-22 12:21:26+00	{"type": "snapshot", "major": false}
96	2	1.20.3-pre1	\N	2023-11-20 15:40:14+00	{"type": "snapshot", "major": false}
98	2	23w45a	\N	2023-11-08 13:59:58+00	{"type": "snapshot", "major": false}
168	2	1.19.1-pre3	\N	2022-07-06 14:50:46+00	{"type": "snapshot", "major": false}
170	2	1.19.1-rc1	\N	2022-06-23 16:32:41+00	{"type": "snapshot", "major": false}
171	2	1.19.1-pre1	\N	2022-06-21 17:13:59+00	{"type": "snapshot", "major": false}
173	2	1.19	\N	2022-06-07 09:42:18+00	{"type": "release", "major": false}
242	2	21w19a	\N	2021-05-12 11:19:15+00	{"type": "snapshot", "major": false}
243	2	21w18a	\N	2021-05-05 15:24:35+00	{"type": "snapshot", "major": false}
255	2	21w05b	\N	2021-02-04 15:09:29+00	{"type": "snapshot", "major": false}
174	2	1.19-rc2	\N	2022-06-03 11:47:25+00	{"type": "snapshot", "major": false}
177	2	1.19-pre4	\N	2022-05-30 14:43:01+00	{"type": "snapshot", "major": false}
179	2	1.19-pre2	\N	2022-05-23 14:54:00+00	{"type": "snapshot", "major": false}
180	2	1.19-pre1	\N	2022-05-18 13:51:54+00	{"type": "snapshot", "major": false}
182	2	22w18a	\N	2022-05-04 14:41:35+00	{"type": "snapshot", "major": false}
185	2	22w16a	\N	2022-04-20 14:37:07+00	{"type": "snapshot", "major": false}
186	2	22w15a	\N	2022-04-13 15:41:17+00	{"type": "snapshot", "major": false}
188	2	22w13oneblockatatime	\N	2022-04-01 11:56:58+00	{"type": "snapshot", "major": false}
189	2	22w13a	\N	2022-03-31 14:53:25+00	{"type": "snapshot", "major": false}
192	2	1.18.2	\N	2022-02-28 10:42:45+00	{"type": "release", "major": false}
200	2	22w03a	\N	2022-01-19 16:04:59+00	{"type": "snapshot", "major": false}
201	2	1.18.1	\N	2021-12-10 08:23:00+00	{"type": "release", "major": false}
203	2	1.18.1-rc2	\N	2021-12-08 12:29:36+00	{"type": "snapshot", "major": false}
204	2	1.18.1-rc1	\N	2021-12-07 15:52:47+00	{"type": "snapshot", "major": false}
206	2	1.18	\N	2021-11-30 09:16:29+00	{"type": "release", "major": false}
207	2	1.18-rc4	\N	2021-11-29 13:43:42+00	{"type": "snapshot", "major": false}
209	2	1.18-rc2	\N	2021-11-26 10:02:04+00	{"type": "snapshot", "major": false}
210	2	1.18-rc1	\N	2021-11-25 14:28:49+00	{"type": "snapshot", "major": false}
212	2	1.18-pre7	\N	2021-11-23 16:37:41+00	{"type": "snapshot", "major": false}
215	2	1.18-pre4	\N	2021-11-17 18:07:56+00	{"type": "snapshot", "major": false}
216	2	1.18-pre3	\N	2021-11-17 16:04:25+00	{"type": "snapshot", "major": false}
218	2	1.18-pre1	\N	2021-11-11 16:14:06+00	{"type": "snapshot", "major": false}
219	2	21w44a	\N	2021-11-03 16:14:34+00	{"type": "snapshot", "major": false}
221	2	21w42a	\N	2021-10-20 12:41:25+00	{"type": "snapshot", "major": false}
222	2	21w41a	\N	2021-10-13 15:23:23+00	{"type": "snapshot", "major": false}
227	2	1.17.1	\N	2021-07-06 12:01:34+00	{"type": "release", "major": false}
228	2	1.17.1-rc2	\N	2021-07-05 12:58:01+00	{"type": "snapshot", "major": false}
230	2	1.17.1-pre3	\N	2021-06-30 15:43:16+00	{"type": "snapshot", "major": false}
233	2	1.17	\N	2021-06-08 11:00:40+00	{"type": "release", "major": false}
234	2	1.17-rc2	\N	2021-06-07 11:46:28+00	{"type": "snapshot", "major": false}
236	2	1.17-pre5	\N	2021-06-03 17:01:28+00	{"type": "snapshot", "major": false}
237	2	1.17-pre4	\N	2021-06-02 16:15:43+00	{"type": "snapshot", "major": false}
239	2	1.17-pre2	\N	2021-05-31 15:54:05+00	{"type": "snapshot", "major": false}
245	2	21w16a	\N	2021-04-21 16:41:14+00	{"type": "snapshot", "major": false}
246	2	21w15a	\N	2021-04-14 13:41:34+00	{"type": "snapshot", "major": false}
248	2	21w13a	\N	2021-03-31 16:17:46+00	{"type": "snapshot", "major": false}
249	2	21w11a	\N	2021-03-17 15:05:50+00	{"type": "snapshot", "major": false}
251	2	21w08b	\N	2021-02-25 11:46:34+00	{"type": "snapshot", "major": false}
252	2	21w08a	\N	2021-02-24 14:38:51+00	{"type": "snapshot", "major": false}
257	2	21w03a	\N	2021-01-20 14:56:29+00	{"type": "snapshot", "major": false}
258	2	1.16.5	\N	2021-01-14 16:05:32+00	{"type": "release", "major": false}
260	2	20w51a	\N	2020-12-16 16:27:57+00	{"type": "snapshot", "major": false}
261	2	20w49a	\N	2020-12-02 16:47:20+00	{"type": "snapshot", "major": false}
137	2	1.19.4-rc1	\N	2023-03-09 14:35:50+00	{"type": "snapshot", "major": false}
140	2	1.19.4-pre2	\N	2023-02-27 13:38:53+00	{"type": "snapshot", "major": false}
141	2	1.19.4-pre1	\N	2023-02-22 16:00:34+00	{"type": "snapshot", "major": false}
143	2	23w06a	\N	2023-02-08 15:00:04+00	{"type": "snapshot", "major": false}
144	2	23w05a	\N	2023-02-01 14:20:33+00	{"type": "snapshot", "major": false}
146	2	23w03a	\N	2023-01-18 13:10:31+00	{"type": "snapshot", "major": false}
147	2	1.19.3	\N	2022-12-07 08:17:18+00	{"type": "release", "major": false}
149	2	1.19.3-rc2	\N	2022-12-05 13:21:34+00	{"type": "snapshot", "major": false}
150	2	1.19.3-rc1	\N	2022-12-01 13:45:18+00	{"type": "snapshot", "major": false}
152	2	1.19.3-pre2	\N	2022-11-23 16:12:25+00	{"type": "snapshot", "major": false}
153	2	1.19.3-pre1	\N	2022-11-22 13:59:37+00	{"type": "snapshot", "major": false}
155	2	22w45a	\N	2022-11-09 14:30:16+00	{"type": "snapshot", "major": false}
156	2	22w44a	\N	2022-11-02 13:15:43+00	{"type": "snapshot", "major": false}
158	2	22w42a	\N	2022-10-19 09:34:22+00	{"type": "snapshot", "major": false}
159	2	1.19.2	\N	2022-08-05 11:57:05+00	{"type": "release", "major": false}
161	2	1.19.2-rc1	\N	2022-08-04 10:07:26+00	{"type": "snapshot", "major": false}
162	2	1.19.1	\N	2022-07-27 09:25:33+00	{"type": "release", "major": false}
164	2	1.19.1-rc2	\N	2022-07-21 16:25:50+00	{"type": "snapshot", "major": false}
165	2	1.19.1-pre6	\N	2022-07-20 15:49:31+00	{"type": "snapshot", "major": false}
167	2	1.19.1-pre4	\N	2022-07-08 11:41:59+00	{"type": "snapshot", "major": false}
183	2	22w17a	\N	2022-04-27 15:54:15+00	{"type": "snapshot", "major": false}
191	2	22w11a	\N	2022-03-16 15:55:38+00	{"type": "snapshot", "major": false}
240	2	1.17-pre1	\N	2021-05-27 09:39:21+00	{"type": "snapshot", "major": false}
194	2	1.18.2-pre3	\N	2022-02-23 15:23:12+00	{"type": "snapshot", "major": false}
195	2	1.18.2-pre2	\N	2022-02-21 15:26:19+00	{"type": "snapshot", "major": false}
197	2	22w07a	\N	2022-02-16 16:13:58+00	{"type": "snapshot", "major": false}
198	2	22w06a	\N	2022-02-09 16:47:48+00	{"type": "snapshot", "major": false}
430	2	18w19b	\N	2018-05-09 10:00:51+00	{"type": "snapshot", "major": false}
433	2	18w15a	\N	2018-04-11 14:54:22+00	{"type": "snapshot", "major": false}
436	2	18w11a	\N	2018-03-13 15:10:59+00	{"type": "snapshot", "major": false}
439	2	18w10b	\N	2018-03-07 15:56:01+00	{"type": "snapshot", "major": false}
334	2	19w38b	\N	2019-09-18 14:59:13+00	{"type": "snapshot", "major": false}
442	2	18w08b	\N	2018-02-22 15:44:49+00	{"type": "snapshot", "major": false}
445	2	18w07b	\N	2018-02-15 14:28:42+00	{"type": "snapshot", "major": false}
448	2	18w05a	\N	2018-01-31 13:32:09+00	{"type": "snapshot", "major": false}
451	2	18w02a	\N	2018-01-10 11:54:55+00	{"type": "snapshot", "major": false}
454	2	17w49b	\N	2017-12-07 15:29:54+00	{"type": "snapshot", "major": false}
457	2	17w47b	\N	2017-11-23 15:30:12+00	{"type": "snapshot", "major": false}
460	2	17w45b	\N	2017-11-10 10:07:02+00	{"type": "snapshot", "major": false}
466	2	1.12.2-pre1	\N	2017-09-13 13:33:31+00	{"type": "snapshot", "major": false}
214	2	1.18-pre5	\N	2021-11-19 15:47:09+00	{"type": "snapshot", "major": false}
217	2	1.18-pre2	\N	2021-11-16 17:04:48+00	{"type": "snapshot", "major": false}
220	2	21w43a	\N	2021-10-27 14:38:55+00	{"type": "snapshot", "major": false}
223	2	21w40a	\N	2021-10-07 11:17:50+00	{"type": "snapshot", "major": false}
226	2	21w37a	\N	2021-09-15 16:04:30+00	{"type": "snapshot", "major": false}
229	2	1.17.1-rc1	\N	2021-07-01 15:23:37+00	{"type": "snapshot", "major": false}
232	2	1.17.1-pre1	\N	2021-06-18 12:24:40+00	{"type": "snapshot", "major": false}
235	2	1.17-rc1	\N	2021-06-04 13:24:48+00	{"type": "snapshot", "major": false}
241	2	21w20a	\N	2021-05-19 15:22:02+00	{"type": "snapshot", "major": false}
244	2	21w17a	\N	2021-04-28 13:54:05+00	{"type": "snapshot", "major": false}
247	2	21w14a	\N	2021-04-07 14:04:09+00	{"type": "snapshot", "major": false}
250	2	21w10a	\N	2021-03-10 15:24:38+00	{"type": "snapshot", "major": false}
253	2	21w07a	\N	2021-02-17 16:35:40+00	{"type": "snapshot", "major": false}
256	2	21w05a	\N	2021-02-03 15:56:54+00	{"type": "snapshot", "major": false}
259	2	1.16.5-rc1	\N	2021-01-13 15:58:55+00	{"type": "snapshot", "major": false}
262	2	20w48a	\N	2020-11-25 15:42:24+00	{"type": "snapshot", "major": false}
265	2	1.16.4	\N	2020-10-29 15:49:37+00	{"type": "release", "major": false}
268	2	1.16.4-pre1	\N	2020-10-13 14:36:07+00	{"type": "snapshot", "major": false}
271	2	1.16.2	\N	2020-08-11 10:13:46+00	{"type": "release", "major": false}
274	2	1.16.2-pre3	\N	2020-08-06 16:44:52+00	{"type": "snapshot", "major": false}
277	2	20w30a	\N	2020-07-22 15:05:15+00	{"type": "snapshot", "major": false}
283	2	1.16-rc1	\N	2020-06-18 12:49:28+00	{"type": "snapshot", "major": false}
286	2	1.16-pre6	\N	2020-06-15 16:57:57+00	{"type": "snapshot", "major": false}
289	2	1.16-pre3	\N	2020-06-10 14:57:43+00	{"type": "snapshot", "major": false}
292	2	20w22a	\N	2020-05-29 11:25:02+00	{"type": "snapshot", "major": false}
295	2	20w20a	\N	2020-05-13 15:11:43+00	{"type": "snapshot", "major": false}
298	2	20w17a	\N	2020-04-22 13:47:50+00	{"type": "snapshot", "major": false}
301	2	20w14a	\N	2020-04-02 14:28:06+00	{"type": "snapshot", "major": false}
304	2	20w13a	\N	2020-03-25 17:05:33+00	{"type": "snapshot", "major": false}
310	2	20w07a	\N	2020-02-14 13:20:49+00	{"type": "snapshot", "major": false}
316	2	1.15.1-pre1	\N	2019-12-12 14:02:30+00	{"type": "snapshot", "major": false}
319	2	1.15-pre6	\N	2019-12-06 12:04:30+00	{"type": "snapshot", "major": false}
322	2	1.15-pre3	\N	2019-11-28 17:17:50+00	{"type": "snapshot", "major": false}
325	2	19w46b	\N	2019-11-14 13:29:24+00	{"type": "snapshot", "major": false}
328	2	19w45a	\N	2019-11-07 16:19:20+00	{"type": "snapshot", "major": false}
331	2	19w41a	\N	2019-10-09 15:21:35+00	{"type": "snapshot", "major": false}
337	2	19w36a	\N	2019-09-04 11:19:34+00	{"type": "snapshot", "major": false}
340	2	1.14.4	\N	2019-07-19 09:25:47+00	{"type": "release", "major": false}
343	2	1.14.4-pre5	\N	2019-07-11 10:52:33+00	{"type": "snapshot", "major": false}
346	2	1.14.4-pre2	\N	2019-07-04 14:41:05+00	{"type": "snapshot", "major": false}
349	2	1.14.3-pre4	\N	2019-06-19 11:44:29+00	{"type": "snapshot", "major": false}
352	2	1.14.3-pre1	\N	2019-06-03 14:34:20+00	{"type": "snapshot", "major": false}
355	2	1.14.2-pre3	\N	2019-05-22 13:12:51+00	{"type": "snapshot", "major": false}
358	2	1.14.1	\N	2019-05-13 11:10:12+00	{"type": "release", "major": false}
361	2	1.14	\N	2019-04-23 14:52:44+00	{"type": "release", "major": false}
364	2	1.14-pre3	\N	2019-04-16 13:57:10+00	{"type": "snapshot", "major": false}
367	2	19w14b	\N	2019-04-05 10:33:58+00	{"type": "snapshot", "major": false}
370	2	19w13b	\N	2019-03-29 16:53:22+00	{"type": "snapshot", "major": false}
373	2	19w12a	\N	2019-03-20 16:47:34+00	{"type": "snapshot", "major": false}
376	2	19w09a	\N	2019-02-27 14:44:30+00	{"type": "snapshot", "major": false}
379	2	19w07a	\N	2019-02-13 16:12:08+00	{"type": "snapshot", "major": false}
382	2	19w04b	\N	2019-01-25 12:20:15+00	{"type": "snapshot", "major": false}
385	2	19w03b	\N	2019-01-17 16:43:27+00	{"type": "snapshot", "major": false}
388	2	18w50a	\N	2018-12-12 14:58:13+00	{"type": "snapshot", "major": false}
391	2	18w48a	\N	2018-11-29 13:11:38+00	{"type": "snapshot", "major": false}
397	2	18w43c	\N	2018-10-26 08:40:46+00	{"type": "snapshot", "major": false}
400	2	1.13.2	\N	2018-10-22 11:41:07+00	{"type": "release", "major": false}
403	2	1.13.1	\N	2018-08-22 14:03:42+00	{"type": "release", "major": false}
406	2	18w33a	\N	2018-08-15 14:28:56+00	{"type": "snapshot", "major": false}
409	2	18w30b	\N	2018-07-26 16:06:57+00	{"type": "snapshot", "major": false}
412	2	1.13-pre10	\N	2018-07-17 14:48:06+00	{"type": "snapshot", "major": false}
415	2	1.13-pre7	\N	2018-07-10 14:21:42+00	{"type": "snapshot", "major": false}
421	2	1.13-pre1	\N	2018-06-04 15:17:34+00	{"type": "snapshot", "major": false}
424	2	18w22a	\N	2018-05-29 13:23:55+00	{"type": "snapshot", "major": false}
427	2	18w20c	\N	2018-05-17 14:06:56+00	{"type": "snapshot", "major": false}
280	2	20w27a	\N	2020-07-01 15:07:35+00	{"type": "snapshot", "major": false}
307	2	20w10a	\N	2020-03-04 16:21:41+00	{"type": "snapshot", "major": false}
365	2	1.14-pre2	\N	2019-04-12 11:38:53+00	{"type": "snapshot", "major": false}
266	2	1.16.4-rc1	\N	2020-10-27 16:31:08+00	{"type": "snapshot", "major": false}
318	2	1.15-pre7	\N	2019-12-09 12:14:11+00	{"type": "snapshot", "major": false}
275	2	1.16.2-pre2	\N	2020-08-05 15:30:50+00	{"type": "snapshot", "major": false}
267	2	1.16.4-pre2	\N	2020-10-22 15:32:17+00	{"type": "snapshot", "major": false}
269	2	1.16.3	\N	2020-09-10 13:42:37+00	{"type": "release", "major": false}
272	2	1.16.2-rc2	\N	2020-08-10 11:43:36+00	{"type": "snapshot", "major": false}
276	2	1.16.2-pre1	\N	2020-07-29 13:19:05+00	{"type": "snapshot", "major": false}
278	2	20w29a	\N	2020-07-15 14:13:47+00	{"type": "snapshot", "major": false}
279	2	20w28a	\N	2020-07-08 15:10:40+00	{"type": "snapshot", "major": false}
282	2	1.16	\N	2020-06-23 16:20:52+00	{"type": "release", "major": false}
285	2	1.16-pre7	\N	2020-06-16 15:31:35+00	{"type": "snapshot", "major": false}
287	2	1.16-pre5	\N	2020-06-12 14:33:59+00	{"type": "snapshot", "major": false}
290	2	1.16-pre2	\N	2020-06-05 10:47:59+00	{"type": "snapshot", "major": false}
291	2	1.16-pre1	\N	2020-06-04 18:17:51+00	{"type": "snapshot", "major": false}
293	2	20w21a	\N	2020-05-20 12:07:18+00	{"type": "snapshot", "major": false}
294	2	20w20b	\N	2020-05-14 08:16:26+00	{"type": "snapshot", "major": false}
296	2	20w19a	\N	2020-05-06 16:23:24+00	{"type": "snapshot", "major": false}
297	2	20w18a	\N	2020-04-29 15:16:34+00	{"type": "snapshot", "major": false}
300	2	20w15a	\N	2020-04-08 12:29:24+00	{"type": "snapshot", "major": false}
302	2	20w14infinite	\N	2020-04-01 12:47:08+00	{"type": "snapshot", "major": false}
305	2	20w12a	\N	2020-03-18 16:42:06+00	{"type": "snapshot", "major": false}
306	2	20w11a	\N	2020-03-11 16:28:27+00	{"type": "snapshot", "major": false}
309	2	20w08a	\N	2020-02-19 13:30:09+00	{"type": "snapshot", "major": false}
311	2	20w06a	\N	2020-02-05 16:05:22+00	{"type": "snapshot", "major": false}
315	2	1.15.1	\N	2019-12-16 10:29:47+00	{"type": "release", "major": false}
317	2	1.15	\N	2019-12-09 13:13:38+00	{"type": "release", "major": false}
320	2	1.15-pre5	\N	2019-12-05 13:20:00+00	{"type": "snapshot", "major": false}
321	2	1.15-pre4	\N	2019-12-03 12:24:24+00	{"type": "snapshot", "major": false}
323	2	1.15-pre2	\N	2019-11-25 18:09:38+00	{"type": "snapshot", "major": false}
324	2	1.15-pre1	\N	2019-11-21 17:01:17+00	{"type": "snapshot", "major": false}
326	2	19w46a	\N	2019-11-13 16:37:46+00	{"type": "snapshot", "major": false}
327	2	19w45b	\N	2019-11-08 12:42:44+00	{"type": "snapshot", "major": false}
333	2	19w39a	\N	2019-09-27 10:13:33+00	{"type": "snapshot", "major": false}
335	2	19w38a	\N	2019-09-18 10:03:22+00	{"type": "snapshot", "major": false}
336	2	19w37a	\N	2019-09-11 11:46:44+00	{"type": "snapshot", "major": false}
338	2	19w35a	\N	2019-08-28 15:01:44+00	{"type": "snapshot", "major": false}
339	2	19w34a	\N	2019-08-22 12:06:21+00	{"type": "snapshot", "major": false}
341	2	1.14.4-pre7	\N	2019-07-18 11:32:36+00	{"type": "snapshot", "major": false}
342	2	1.14.4-pre6	\N	2019-07-15 12:39:49+00	{"type": "snapshot", "major": false}
344	2	1.14.4-pre4	\N	2019-07-10 12:53:29+00	{"type": "snapshot", "major": false}
345	2	1.14.4-pre3	\N	2019-07-08 11:21:42+00	{"type": "snapshot", "major": false}
347	2	1.14.4-pre1	\N	2019-07-03 13:01:01+00	{"type": "snapshot", "major": false}
348	2	1.14.3	\N	2019-06-24 12:52:52+00	{"type": "release", "major": false}
351	2	1.14.3-pre2	\N	2019-06-07 09:11:29+00	{"type": "snapshot", "major": false}
353	2	1.14.2	\N	2019-05-27 11:48:25+00	{"type": "release", "major": false}
354	2	1.14.2-pre4	\N	2019-05-27 07:21:11+00	{"type": "snapshot", "major": false}
356	2	1.14.2-pre2	\N	2019-05-17 12:21:03+00	{"type": "snapshot", "major": false}
359	2	1.14.1-pre2	\N	2019-05-09 14:01:04+00	{"type": "snapshot", "major": false}
360	2	1.14.1-pre1	\N	2019-05-07 14:44:42+00	{"type": "snapshot", "major": false}
362	2	1.14-pre5	\N	2019-04-18 11:05:19+00	{"type": "snapshot", "major": false}
363	2	1.14-pre4	\N	2019-04-17 15:31:12+00	{"type": "snapshot", "major": false}
366	2	1.14-pre1	\N	2019-04-10 14:24:16+00	{"type": "snapshot", "major": false}
368	2	19w14a	\N	2019-04-03 13:45:00+00	{"type": "snapshot", "major": false}
371	2	19w13a	\N	2019-03-27 15:15:31+00	{"type": "snapshot", "major": false}
372	2	19w12b	\N	2019-03-21 15:20:01+00	{"type": "snapshot", "major": false}
374	2	19w11b	\N	2019-03-14 14:26:23+00	{"type": "snapshot", "major": false}
375	2	19w11a	\N	2019-03-13 13:59:29+00	{"type": "snapshot", "major": false}
377	2	19w08b	\N	2019-02-21 13:38:09+00	{"type": "snapshot", "major": false}
378	2	19w08a	\N	2019-02-20 14:56:58+00	{"type": "snapshot", "major": false}
380	2	19w06a	\N	2019-02-06 16:24:13+00	{"type": "snapshot", "major": false}
381	2	19w05a	\N	2019-01-30 15:16:49+00	{"type": "snapshot", "major": false}
383	2	19w04a	\N	2019-01-24 15:31:52+00	{"type": "snapshot", "major": false}
384	2	19w03c	\N	2019-01-18 11:27:13+00	{"type": "snapshot", "major": false}
386	2	19w03a	\N	2019-01-16 16:45:02+00	{"type": "snapshot", "major": false}
387	2	19w02a	\N	2019-01-09 15:52:07+00	{"type": "snapshot", "major": false}
389	2	18w49a	\N	2018-12-05 12:24:30+00	{"type": "snapshot", "major": false}
270	2	1.16.3-rc1	\N	2020-09-07 12:34:06+00	{"type": "snapshot", "major": false}
273	2	1.16.2-rc1	\N	2020-08-07 14:35:39+00	{"type": "snapshot", "major": false}
284	2	1.16-pre8	\N	2020-06-17 14:45:23+00	{"type": "snapshot", "major": false}
288	2	1.16-pre4	\N	2020-06-11 15:45:55+00	{"type": "snapshot", "major": false}
299	2	20w16a	\N	2020-04-15 14:13:01+00	{"type": "snapshot", "major": false}
308	2	20w09a	\N	2020-02-26 16:43:08+00	{"type": "snapshot", "major": false}
329	2	19w44a	\N	2019-10-30 15:31:44+00	{"type": "snapshot", "major": false}
330	2	19w42a	\N	2019-10-16 15:30:39+00	{"type": "snapshot", "major": false}
332	2	19w40a	\N	2019-10-02 13:40:26+00	{"type": "snapshot", "major": false}
369	2	3D-Shareware-v1.34	\N	2019-04-01 11:18:08+00	{"type": "snapshot", "major": false}
417	2	1.13-pre5	\N	2018-06-28 13:58:53+00	{"type": "snapshot", "major": false}
420	2	1.13-pre2	\N	2018-06-15 09:20:00+00	{"type": "snapshot", "major": false}
422	2	18w22c	\N	2018-05-31 13:53:15+00	{"type": "snapshot", "major": false}
503	2	16w36a	\N	2016-09-08 14:55:10+00	{"type": "snapshot", "major": false}
507	2	16w32a	\N	2016-08-10 12:30:10+00	{"type": "snapshot", "major": false}
509	2	1.10.1	\N	2016-06-22 10:13:22+00	{"type": "release", "major": false}
423	2	18w22b	\N	2018-05-30 13:48:58+00	{"type": "snapshot", "major": false}
426	2	18w21a	\N	2018-05-23 13:11:49+00	{"type": "snapshot", "major": false}
431	2	18w19a	\N	2018-05-08 13:05:19+00	{"type": "snapshot", "major": false}
432	2	18w16a	\N	2018-04-19 14:46:35+00	{"type": "snapshot", "major": false}
434	2	18w14b	\N	2018-04-05 14:44:02+00	{"type": "snapshot", "major": false}
435	2	18w14a	\N	2018-04-04 14:36:14+00	{"type": "snapshot", "major": false}
440	2	18w10a	\N	2018-03-06 15:54:24+00	{"type": "snapshot", "major": false}
441	2	18w09a	\N	2018-03-01 14:15:10+00	{"type": "snapshot", "major": false}
443	2	18w08a	\N	2018-02-21 14:59:00+00	{"type": "snapshot", "major": false}
444	2	18w07c	\N	2018-02-16 13:23:32+00	{"type": "snapshot", "major": false}
446	2	18w07a	\N	2018-02-14 17:34:13+00	{"type": "snapshot", "major": false}
450	2	18w03a	\N	2018-01-17 14:25:24+00	{"type": "snapshot", "major": false}
452	2	18w01a	\N	2018-01-03 13:29:30+00	{"type": "snapshot", "major": false}
453	2	17w50a	\N	2017-12-11 15:28:08+00	{"type": "snapshot", "major": false}
455	2	17w49a	\N	2017-12-06 14:24:30+00	{"type": "snapshot", "major": false}
456	2	17w48a	\N	2017-11-27 15:36:33+00	{"type": "snapshot", "major": false}
458	2	17w47a	\N	2017-11-22 12:40:05+00	{"type": "snapshot", "major": false}
459	2	17w46a	\N	2017-11-15 15:21:55+00	{"type": "snapshot", "major": false}
461	2	17w45a	\N	2017-11-08 15:48:00+00	{"type": "snapshot", "major": false}
464	2	1.12.2	\N	2017-09-18 08:39:46+00	{"type": "release", "major": false}
467	2	1.12.1	\N	2017-08-03 12:40:39+00	{"type": "release", "major": false}
468	2	1.12.1-pre1	\N	2017-08-02 10:53:55+00	{"type": "snapshot", "major": false}
470	2	1.12	\N	2017-06-02 13:50:27+00	{"type": "release", "major": false}
471	2	1.12-pre7	\N	2017-05-31 10:56:41+00	{"type": "snapshot", "major": false}
473	2	1.12-pre5	\N	2017-05-19 07:43:28+00	{"type": "snapshot", "major": false}
474	2	1.12-pre4	\N	2017-05-18 12:28:16+00	{"type": "snapshot", "major": false}
476	2	1.12-pre2	\N	2017-05-11 12:11:12+00	{"type": "snapshot", "major": false}
477	2	1.12-pre1	\N	2017-05-10 11:37:17+00	{"type": "snapshot", "major": false}
479	2	17w18a	\N	2017-05-03 14:50:23+00	{"type": "snapshot", "major": false}
480	2	17w17b	\N	2017-04-27 13:24:23+00	{"type": "snapshot", "major": false}
482	2	17w16b	\N	2017-04-21 12:02:59+00	{"type": "snapshot", "major": false}
485	2	17w14a	\N	2017-04-05 13:58:01+00	{"type": "snapshot", "major": false}
486	2	17w13b	\N	2017-03-31 11:06:35+00	{"type": "snapshot", "major": false}
488	2	17w06a	\N	2017-02-08 13:16:29+00	{"type": "snapshot", "major": false}
489	2	1.11.2	\N	2016-12-21 09:29:12+00	{"type": "release", "major": false}
491	2	16w50a	\N	2016-12-15 14:38:52+00	{"type": "snapshot", "major": false}
492	2	1.11	\N	2016-11-14 14:34:40+00	{"type": "release", "major": false}
494	2	16w44a	\N	2016-11-03 14:17:11+00	{"type": "snapshot", "major": false}
495	2	16w43a	\N	2016-10-27 09:00:51+00	{"type": "snapshot", "major": false}
498	2	16w40a	\N	2016-10-06 13:57:59+00	{"type": "snapshot", "major": false}
500	2	16w39b	\N	2016-09-29 14:39:39+00	{"type": "snapshot", "major": false}
501	2	16w39a	\N	2016-09-28 13:32:06+00	{"type": "snapshot", "major": false}
510	2	1.10	\N	2016-06-08 13:06:18+00	{"type": "release", "major": false}
512	2	1.10-pre1	\N	2016-06-02 14:45:16+00	{"type": "snapshot", "major": false}
513	2	16w21b	\N	2016-05-26 12:47:22+00	{"type": "snapshot", "major": false}
515	2	16w20a	\N	2016-05-18 12:45:14+00	{"type": "snapshot", "major": false}
516	2	1.9.4	\N	2016-05-10 10:17:16+00	{"type": "release", "major": false}
518	2	1.9.3-pre3	\N	2016-05-03 09:28:11+00	{"type": "snapshot", "major": false}
519	2	1.9.3-pre2	\N	2016-04-27 13:33:20+00	{"type": "snapshot", "major": false}
392	2	18w47b	\N	2018-11-23 10:46:41+00	{"type": "snapshot", "major": false}
395	2	18w45a	\N	2018-11-07 14:40:06+00	{"type": "snapshot", "major": false}
396	2	18w44a	\N	2018-10-31 15:29:16+00	{"type": "snapshot", "major": false}
398	2	18w43b	\N	2018-10-24 15:02:30+00	{"type": "snapshot", "major": false}
399	2	18w43a	\N	2018-10-24 10:52:16+00	{"type": "snapshot", "major": false}
401	2	1.13.2-pre2	\N	2018-10-18 14:46:12+00	{"type": "snapshot", "major": false}
402	2	1.13.2-pre1	\N	2018-10-16 13:40:58+00	{"type": "snapshot", "major": false}
404	2	1.13.1-pre2	\N	2018-08-20 13:52:09+00	{"type": "snapshot", "major": false}
405	2	1.13.1-pre1	\N	2018-08-16 13:08:44+00	{"type": "snapshot", "major": false}
407	2	18w32a	\N	2018-08-08 13:16:57+00	{"type": "snapshot", "major": false}
408	2	18w31a	\N	2018-08-01 12:54:44+00	{"type": "snapshot", "major": false}
410	2	18w30a	\N	2018-07-25 14:29:31+00	{"type": "snapshot", "major": false}
411	2	1.13	\N	2018-07-18 15:11:46+00	{"type": "release", "major": false}
413	2	1.13-pre9	\N	2018-07-16 14:17:42+00	{"type": "snapshot", "major": false}
414	2	1.13-pre8	\N	2018-07-13 11:45:00+00	{"type": "snapshot", "major": false}
416	2	1.13-pre6	\N	2018-07-04 12:36:00+00	{"type": "snapshot", "major": false}
419	2	1.13-pre3	\N	2018-06-21 12:57:11+00	{"type": "snapshot", "major": false}
425	2	18w21b	\N	2018-05-25 10:09:09+00	{"type": "snapshot", "major": false}
429	2	18w20a	\N	2018-05-15 14:02:25+00	{"type": "snapshot", "major": false}
449	2	18w03b	\N	2018-01-17 15:09:14+00	{"type": "snapshot", "major": false}
465	2	1.12.2-pre2	\N	2017-09-15 08:21:17+00	{"type": "snapshot", "major": false}
504	2	16w35a	\N	2016-09-01 13:13:38+00	{"type": "snapshot", "major": false}
506	2	16w32b	\N	2016-08-11 14:34:29+00	{"type": "snapshot", "major": false}
483	2	17w16a	\N	2017-04-20 13:58:35+00	{"type": "snapshot", "major": false}
589	2	15w31c	\N	2015-07-31 13:45:08+00	{"type": "snapshot", "major": false}
622	2	14w33b	\N	2014-08-15 16:23:51+00	{"type": "snapshot", "major": false}
526	2	1.9.1	\N	2016-03-30 13:43:07+00	{"type": "release", "major": false}
556	2	15w43c	\N	2015-10-23 15:35:55+00	{"type": "snapshot", "major": false}
559	2	15w42a	\N	2015-10-14 13:25:14+00	{"type": "snapshot", "major": false}
625	2	14w32c	\N	2014-08-08 14:11:20+00	{"type": "snapshot", "major": false}
493	2	1.11-pre1	\N	2016-11-08 13:42:50+00	{"type": "snapshot", "major": false}
529	2	1.9.1-pre1	\N	2016-03-09 16:27:29+00	{"type": "snapshot", "major": false}
652	2	14w19a	\N	2014-05-08 14:24:19+00	{"type": "snapshot", "major": false}
655	2	14w17a	\N	2014-04-24 15:44:49+00	{"type": "snapshot", "major": false}
658	2	1.7.8	\N	2014-04-09 07:58:16+00	{"type": "release", "major": false}
661	2	14w11a	\N	2014-03-13 14:02:50+00	{"type": "snapshot", "major": false}
496	2	16w42a	\N	2016-10-19 11:17:47+00	{"type": "snapshot", "major": false}
499	2	16w39c	\N	2016-09-30 14:11:48+00	{"type": "snapshot", "major": false}
502	2	16w38a	\N	2016-09-20 12:40:49+00	{"type": "snapshot", "major": false}
532	2	1.9-pre3	\N	2016-02-24 15:52:36+00	{"type": "snapshot", "major": false}
535	2	16w07b	\N	2016-02-16 15:22:39+00	{"type": "snapshot", "major": false}
562	2	15w40b	\N	2015-09-30 14:13:54+00	{"type": "snapshot", "major": false}
565	2	15w39b	\N	2015-09-21 15:09:52+00	{"type": "snapshot", "major": false}
592	2	1.8.8	\N	2015-07-27 10:31:28+00	{"type": "release", "major": false}
595	2	1.8.5	\N	2015-05-22 11:15:28+00	{"type": "release", "major": false}
601	2	1.8.2-pre6	\N	2015-01-30 11:58:24+00	{"type": "snapshot", "major": false}
628	2	14w31a	\N	2014-07-30 15:38:05+00	{"type": "snapshot", "major": false}
631	2	14w30a	\N	2014-07-23 13:15:42+00	{"type": "snapshot", "major": false}
604	2	1.8.2-pre3	\N	2015-01-15 16:44:33+00	{"type": "snapshot", "major": false}
505	2	16w33a	\N	2016-08-17 12:48:57+00	{"type": "snapshot", "major": false}
634	2	14w28b	\N	2014-07-10 14:28:48+00	{"type": "snapshot", "major": false}
664	2	14w10c	\N	2014-03-07 13:49:55+00	{"type": "snapshot", "major": false}
670	2	14w06b	\N	2014-02-06 17:30:42+00	{"type": "snapshot", "major": false}
508	2	1.10.2	\N	2016-06-23 09:17:32+00	{"type": "release", "major": false}
538	2	16w05b	\N	2016-02-04 15:28:02+00	{"type": "snapshot", "major": false}
568	2	15w38a	\N	2015-09-16 14:22:31+00	{"type": "snapshot", "major": false}
571	2	15w36c	\N	2015-09-02 16:07:22+00	{"type": "snapshot", "major": false}
511	2	1.10-pre2	\N	2016-06-07 14:56:34+00	{"type": "snapshot", "major": false}
514	2	16w21a	\N	2016-05-25 13:12:09+00	{"type": "snapshot", "major": false}
541	2	16w03a	\N	2016-01-20 14:29:24+00	{"type": "snapshot", "major": false}
607	2	1.8.1	\N	2014-11-24 14:13:31+00	{"type": "release", "major": false}
610	2	1.8.1-pre3	\N	2014-10-23 12:59:42+00	{"type": "snapshot", "major": false}
637	2	14w27a	\N	2014-07-02 16:07:20+00	{"type": "snapshot", "major": false}
475	2	1.12-pre3	\N	2017-05-17 14:09:18+00	{"type": "snapshot", "major": false}
574	2	15w35e	\N	2015-08-28 18:14:02+00	{"type": "snapshot", "major": false}
577	2	15w35b	\N	2015-08-24 15:39:18+00	{"type": "snapshot", "major": false}
640	2	14w26a	\N	2014-06-25 13:59:27+00	{"type": "snapshot", "major": false}
643	2	14w21b	\N	2014-05-22 15:17:55+00	{"type": "snapshot", "major": false}
646	2	14w20a	\N	2014-05-15 14:01:20+00	{"type": "snapshot", "major": false}
649	2	1.7.10-pre3	\N	2014-05-14 15:29:23+00	{"type": "snapshot", "major": false}
673	2	14w05a	\N	2014-01-30 15:32:41+00	{"type": "snapshot", "major": false}
517	2	1.9.3	\N	2016-05-10 08:33:35+00	{"type": "release", "major": false}
580	2	15w34c	\N	2015-08-21 12:45:20+00	{"type": "snapshot", "major": false}
583	2	15w33c	\N	2015-08-14 13:10:46+00	{"type": "snapshot", "major": false}
613	2	1.8	\N	2014-09-02 08:24:35+00	{"type": "release", "major": false}
478	2	17w18b	\N	2017-05-04 13:40:22+00	{"type": "snapshot", "major": false}
481	2	17w17a	\N	2017-04-26 13:48:23+00	{"type": "snapshot", "major": false}
484	2	17w15a	\N	2017-04-12 09:30:50+00	{"type": "snapshot", "major": false}
523	2	16w14a	\N	2016-04-07 12:47:51+00	{"type": "snapshot", "major": false}
544	2	15w51a	\N	2015-12-17 14:02:37+00	{"type": "snapshot", "major": false}
547	2	1.8.9	\N	2015-12-03 09:24:39+00	{"type": "release", "major": false}
550	2	15w47b	\N	2015-11-19 14:48:03+00	{"type": "snapshot", "major": false}
553	2	15w45a	\N	2015-11-05 13:04:07+00	{"type": "snapshot", "major": false}
586	2	15w32c	\N	2015-08-07 14:08:17+00	{"type": "snapshot", "major": false}
616	2	1.8-pre1	\N	2014-08-21 13:56:26+00	{"type": "snapshot", "major": false}
619	2	14w34b	\N	2014-08-18 15:14:28+00	{"type": "snapshot", "major": false}
676	2	14w03b	\N	2014-01-16 16:36:19+00	{"type": "snapshot", "major": false}
679	2	14w02b	\N	2014-01-09 15:45:55+00	{"type": "snapshot", "major": false}
487	2	17w13a	\N	2017-03-30 09:32:19+00	{"type": "snapshot", "major": false}
490	2	1.11.1	\N	2016-12-20 14:05:34+00	{"type": "release", "major": false}
641	2	14w25b	\N	2014-06-19 12:29:48+00	{"type": "snapshot", "major": false}
525	2	1.9.2	\N	2016-03-30 15:23:55+00	{"type": "release", "major": false}
662	2	1.7.6-pre2	\N	2014-03-08 11:00:01+00	{"type": "snapshot", "major": false}
545	2	15w50a	\N	2015-12-09 15:35:57+00	{"type": "snapshot", "major": false}
621	2	14w33c	\N	2014-08-15 18:00:26+00	{"type": "snapshot", "major": false}
663	2	1.7.6-pre1	\N	2014-03-08 11:00:00+00	{"type": "snapshot", "major": false}
527	2	1.9.1-pre3	\N	2016-03-11 09:20:36+00	{"type": "snapshot", "major": false}
665	2	14w10b	\N	2014-03-06 16:25:39+00	{"type": "snapshot", "major": false}
528	2	1.9.1-pre2	\N	2016-03-10 15:06:03+00	{"type": "snapshot", "major": false}
623	2	14w33a	\N	2014-08-13 15:08:14+00	{"type": "snapshot", "major": false}
624	2	14w32d	\N	2014-08-08 15:13:41+00	{"type": "snapshot", "major": false}
626	2	14w32b	\N	2014-08-07 14:45:17+00	{"type": "snapshot", "major": false}
666	2	14w10a	\N	2014-03-06 14:23:04+00	{"type": "snapshot", "major": false}
668	2	1.7.5	\N	2014-02-26 09:22:17+00	{"type": "release", "major": false}
672	2	14w05b	\N	2014-01-31 14:05:50+00	{"type": "snapshot", "major": false}
530	2	1.9	\N	2016-02-29 13:49:54+00	{"type": "release", "major": false}
531	2	1.9-pre4	\N	2016-02-26 15:21:11+00	{"type": "snapshot", "major": false}
533	2	1.9-pre2	\N	2016-02-18 17:41:00+00	{"type": "snapshot", "major": false}
534	2	1.9-pre1	\N	2016-02-17 15:23:19+00	{"type": "snapshot", "major": false}
536	2	16w07a	\N	2016-02-15 15:48:46+00	{"type": "snapshot", "major": false}
537	2	16w06a	\N	2016-02-10 15:06:41+00	{"type": "snapshot", "major": false}
627	2	14w32a	\N	2014-08-06 14:01:16+00	{"type": "snapshot", "major": false}
629	2	14w30c	\N	2014-07-24 14:39:09+00	{"type": "snapshot", "major": false}
674	2	14w04b	\N	2014-01-24 15:48:46+00	{"type": "snapshot", "major": false}
675	2	14w04a	\N	2014-01-23 15:26:13+00	{"type": "snapshot", "major": false}
677	2	14w03a	\N	2014-01-16 14:45:13+00	{"type": "snapshot", "major": false}
539	2	16w05a	\N	2016-02-03 15:48:38+00	{"type": "snapshot", "major": false}
540	2	16w04a	\N	2016-01-28 15:37:24+00	{"type": "snapshot", "major": false}
630	2	14w30b	\N	2014-07-23 15:03:03+00	{"type": "snapshot", "major": false}
632	2	14w29b	\N	2014-07-16 17:27:40+00	{"type": "snapshot", "major": false}
633	2	14w29a	\N	2014-07-16 15:18:17+00	{"type": "snapshot", "major": false}
635	2	14w28a	\N	2014-07-09 15:42:36+00	{"type": "snapshot", "major": false}
636	2	14w27b	\N	2014-07-02 18:34:56+00	{"type": "snapshot", "major": false}
678	2	14w02c	\N	2014-01-10 15:42:36+00	{"type": "snapshot", "major": false}
542	2	16w02a	\N	2016-01-13 15:15:16+00	{"type": "snapshot", "major": false}
603	2	1.8.2-pre4	\N	2015-01-16 14:19:59+00	{"type": "snapshot", "major": false}
638	2	14w26c	\N	2014-06-26 15:05:03+00	{"type": "snapshot", "major": false}
642	2	14w25a	\N	2014-06-18 15:52:28+00	{"type": "snapshot", "major": false}
644	2	14w21a	\N	2014-05-22 14:44:33+00	{"type": "snapshot", "major": false}
645	2	14w20b	\N	2014-05-15 16:47:21+00	{"type": "snapshot", "major": false}
680	2	14w02a	\N	2014-01-09 14:44:41+00	{"type": "snapshot", "major": false}
605	2	1.8.2-pre2	\N	2015-01-15 15:07:31+00	{"type": "snapshot", "major": false}
606	2	1.8.2-pre1	\N	2014-12-18 11:29:41+00	{"type": "snapshot", "major": false}
608	2	1.8.1-pre5	\N	2014-11-19 14:30:48+00	{"type": "snapshot", "major": false}
609	2	1.8.1-pre4	\N	2014-11-06 14:10:50+00	{"type": "snapshot", "major": false}
647	2	1.7.10	\N	2014-05-14 17:29:23+00	{"type": "release", "major": false}
648	2	1.7.10-pre4	\N	2014-05-14 16:29:23+00	{"type": "snapshot", "major": false}
611	2	1.8.1-pre2	\N	2014-10-16 14:19:27+00	{"type": "snapshot", "major": false}
650	2	1.7.10-pre2	\N	2014-05-14 14:29:23+00	{"type": "snapshot", "major": false}
612	2	1.8.1-pre1	\N	2014-10-15 13:25:11+00	{"type": "snapshot", "major": false}
614	2	1.8-pre3	\N	2014-08-28 09:40:54+00	{"type": "snapshot", "major": false}
617	2	14w34d	\N	2014-08-20 12:46:59+00	{"type": "snapshot", "major": false}
651	2	1.7.10-pre1	\N	2014-05-14 13:29:23+00	{"type": "snapshot", "major": false}
653	2	14w18b	\N	2014-05-02 11:38:17+00	{"type": "snapshot", "major": false}
654	2	14w18a	\N	2014-04-30 10:25:35+00	{"type": "snapshot", "major": false}
618	2	14w34c	\N	2014-08-19 15:31:24+00	{"type": "snapshot", "major": false}
657	2	1.7.9	\N	2014-04-14 13:29:23+00	{"type": "release", "major": false}
620	2	14w34a	\N	2014-08-18 14:14:11+00	{"type": "snapshot", "major": false}
639	2	14w26b	\N	2014-06-25 15:08:39+00	{"type": "snapshot", "major": false}
659	2	1.7.7	\N	2014-04-09 07:52:16+00	{"type": "release", "major": false}
522	2	16w15a	\N	2016-04-11 14:38:28+00	{"type": "snapshot", "major": false}
543	2	15w51b	\N	2015-12-17 15:30:41+00	{"type": "snapshot", "major": false}
660	2	1.7.6	\N	2014-04-09 07:52:06+00	{"type": "release", "major": false}
710	2	1.6.2	\N	2013-07-05 13:09:02+00	{"type": "release", "major": false}
711	2	1.6.1	\N	2013-06-28 14:48:41+00	{"type": "release", "major": false}
713	2	13w26a	\N	2013-06-24 16:06:06+00	{"type": "snapshot", "major": false}
714	2	13w25c	\N	2013-06-20 15:23:37+00	{"type": "snapshot", "major": false}
716	2	13w25a	\N	2013-06-17 14:08:06+00	{"type": "snapshot", "major": false}
717	2	13w24b	\N	2013-06-14 12:19:13+00	{"type": "snapshot", "major": false}
719	2	13w23b	\N	2013-06-08 00:32:01+00	{"type": "snapshot", "major": false}
720	2	13w23a	\N	2013-06-07 16:04:20+00	{"type": "snapshot", "major": false}
722	2	13w21b	\N	2013-05-27 08:50:42+00	{"type": "snapshot", "major": false}
723	2	13w21a	\N	2013-05-23 15:38:28+00	{"type": "snapshot", "major": false}
725	2	13w18c	\N	2013-05-03 09:19:35+00	{"type": "snapshot", "major": false}
726	2	13w18b	\N	2013-05-02 17:12:25+00	{"type": "snapshot", "major": false}
728	2	13w17a	\N	2013-04-25 15:50:00+00	{"type": "snapshot", "major": false}
729	2	1.5.2	\N	2013-04-25 15:45:00+00	{"type": "release", "major": false}
731	2	13w16a	\N	2013-04-21 12:49:30+00	{"type": "snapshot", "major": false}
732	2	1.5.1	\N	2013-03-20 10:00:00+00	{"type": "release", "major": false}
734	2	1.4.7	\N	2012-12-27 22:00:00+00	{"type": "release", "major": false}
735	2	1.4.5	\N	2012-12-19 22:00:00+00	{"type": "release", "major": false}
737	2	1.4.4	\N	2012-12-13 22:00:00+00	{"type": "release", "major": false}
738	2	1.4.3	\N	2012-11-30 22:00:00+00	{"type": "snapshot", "major": false}
740	2	1.4.1	\N	2012-11-22 22:00:00+00	{"type": "snapshot", "major": false}
767	2	b1.4	\N	2011-03-30 22:00:00+00	{"type": "beta", "major": false}
768	2	b1.3_01	\N	2011-02-22 22:00:00+00	{"type": "beta", "major": false}
770	2	b1.2_02	\N	2011-01-20 22:00:00+00	{"type": "beta", "major": false}
771	2	b1.2_01	\N	2011-01-13 22:00:00+00	{"type": "beta", "major": false}
773	2	b1.1_02	\N	2010-12-21 22:00:01+00	{"type": "beta", "major": false}
774	2	b1.1_01	\N	2010-12-21 22:00:00+00	{"type": "beta", "major": false}
776	2	b1.0_01	\N	2010-12-19 22:00:01+00	{"type": "beta", "major": false}
777	2	b1.0	\N	2010-12-19 22:00:00+00	{"type": "beta", "major": false}
779	2	a1.2.5	\N	2010-11-30 22:00:00+00	{"type": "alpha", "major": false}
17022531	2	1.21.8	\N	2025-07-17 12:04:02+00	{"type": "release", "major": false}
121	2	1.20-pre6	\N	2023-05-25 12:22:00+00	{"type": "snapshot", "major": false}
135	2	1.19.4-rc3	\N	2023-03-13 10:03:11+00	{"type": "snapshot", "major": false}
211	2	1.18-pre8	\N	2021-11-24 14:57:32+00	{"type": "snapshot", "major": false}
213	2	1.18-pre6	\N	2021-11-22 17:09:05+00	{"type": "snapshot", "major": false}
263	2	20w46a	\N	2020-11-11 15:30:32+00	{"type": "snapshot", "major": false}
281	2	1.16.1	\N	2020-06-24 10:31:40+00	{"type": "release", "major": false}
313	2	1.15.2-pre2	\N	2020-01-16 12:35:57+00	{"type": "snapshot", "major": false}
393	2	18w47a	\N	2018-11-21 15:45:22+00	{"type": "snapshot", "major": false}
463	2	17w43a	\N	2017-10-25 14:43:50+00	{"type": "snapshot", "major": false}
469	2	17w31a	\N	2017-08-01 09:41:23+00	{"type": "snapshot", "major": false}
497	2	16w41a	\N	2016-10-13 14:28:35+00	{"type": "snapshot", "major": false}
521	2	16w15b	\N	2016-04-13 13:56:41+00	{"type": "snapshot", "major": false}
615	2	1.8-pre2	\N	2014-08-25 14:52:18+00	{"type": "snapshot", "major": false}
667	2	14w08a	\N	2014-02-26 17:00:00+00	{"type": "snapshot", "major": false}
708	2	13w36b	\N	2013-09-06 12:31:58+00	{"type": "snapshot", "major": false}
811	2	rd-132328	\N	2009-05-13 21:28:00+00	{"type": "alpha", "major": false}
594	2	1.8.6	\N	2015-05-25 10:31:19+00	{"type": "release", "major": false}
598	2	1.8.3	\N	2015-02-20 14:00:09+00	{"type": "release", "major": false}
656	2	14w11b	\N	2014-04-14 14:36:19+00	{"type": "snapshot", "major": false}
690	2	13w47a	\N	2013-11-21 15:59:58+00	{"type": "snapshot", "major": false}
730	2	13w16b	\N	2013-04-23 21:51:22+00	{"type": "snapshot", "major": false}
733	2	1.5	\N	2013-03-06 22:00:00+00	{"type": "snapshot", "major": false}
736	2	1.4.6	\N	2012-12-19 22:00:01+00	{"type": "release", "major": false}
739	2	1.4.2	\N	2012-11-24 22:00:00+00	{"type": "release", "major": false}
742	2	1.3.2	\N	2012-08-15 22:00:00+00	{"type": "release", "major": false}
745	2	1.2.5	\N	2012-03-29 22:00:00+00	{"type": "release", "major": false}
748	2	1.2.2	\N	2012-02-29 22:00:01+00	{"type": "release", "major": false}
751	2	1.0	\N	2011-11-17 22:00:00+00	{"type": "release", "major": false}
754	2	b1.7.3	\N	2011-07-07 22:00:00+00	{"type": "beta", "major": false}
757	2	b1.6.6	\N	2011-05-30 22:00:00+00	{"type": "beta", "major": false}
760	2	b1.6.3	\N	2011-05-25 22:00:03+00	{"type": "beta", "major": false}
50	2	1.21.1-rc1	\N	2024-08-07 14:29:18+00	{"type": "snapshot", "major": false}
138	2	1.19.4-pre4	\N	2023-03-08 13:08:22+00	{"type": "snapshot", "major": false}
176	2	1.19-pre5	\N	2022-06-01 10:56:23+00	{"type": "snapshot", "major": false}
196	2	1.18.2-pre1	\N	2022-02-18 16:00:32+00	{"type": "snapshot", "major": false}
238	2	1.17-pre3	\N	2021-06-01 15:43:46+00	{"type": "snapshot", "major": false}
254	2	21w06a	\N	2021-02-10 17:13:54+00	{"type": "snapshot", "major": false}
314	2	1.15.2-pre1	\N	2020-01-14 16:19:31+00	{"type": "snapshot", "major": false}
350	2	1.14.3-pre3	\N	2019-06-14 08:03:33+00	{"type": "snapshot", "major": false}
390	2	18w48b	\N	2018-11-30 10:37:31+00	{"type": "snapshot", "major": false}
394	2	18w46a	\N	2018-11-15 13:43:14+00	{"type": "snapshot", "major": false}
428	2	18w20b	\N	2018-05-16 14:35:35+00	{"type": "snapshot", "major": false}
462	2	17w43b	\N	2017-10-26 13:36:22+00	{"type": "snapshot", "major": false}
520	2	1.9.3-pre1	\N	2016-04-21 12:41:42+00	{"type": "snapshot", "major": false}
524	2	1.RV-Pre1	\N	2016-03-31 16:18:53+00	{"type": "snapshot", "major": false}
763	2	b1.6	\N	2011-05-25 22:00:00+00	{"type": "beta", "major": false}
766	2	b1.4_01	\N	2011-04-04 22:00:00+00	{"type": "beta", "major": false}
769	2	b1.3b	\N	2011-02-21 22:00:00+00	{"type": "beta", "major": false}
772	2	b1.2	\N	2011-01-12 22:00:00+00	{"type": "beta", "major": false}
775	2	b1.0.2	\N	2010-12-20 22:00:00+00	{"type": "beta", "major": false}
778	2	a1.2.6	\N	2010-12-02 22:00:00+00	{"type": "alpha", "major": false}
781	2	a1.2.3_04	\N	2010-11-25 22:00:00+00	{"type": "alpha", "major": false}
784	2	a1.2.3	\N	2010-11-23 22:00:00+00	{"type": "alpha", "major": false}
787	2	a1.2.1_01	\N	2010-11-04 22:00:01+00	{"type": "alpha", "major": false}
790	2	a1.2.0_01	\N	2010-10-30 22:00:00+00	{"type": "alpha", "major": false}
793	2	a1.1.2	\N	2010-09-19 22:00:00+00	{"type": "alpha", "major": false}
796	2	a1.0.17_02	\N	2010-08-19 22:00:00+00	{"type": "alpha", "major": false}
799	2	a1.0.14	\N	2010-07-29 22:00:00+00	{"type": "alpha", "major": false}
802	2	a1.0.4	\N	2010-07-08 22:00:00+00	{"type": "alpha", "major": false}
805	2	c0.0.13a	\N	2009-05-30 22:00:00+00	{"type": "alpha", "major": false}
808	2	rd-161348	\N	2009-05-16 11:48:00+00	{"type": "alpha", "major": false}
789	2	a1.2.0_02	\N	2010-11-03 22:00:00+00	{"type": "alpha", "major": false}
791	2	a1.2.0	\N	2010-10-29 22:00:00+00	{"type": "alpha", "major": false}
792	2	a1.1.2_01	\N	2010-09-22 22:00:00+00	{"type": "alpha", "major": false}
794	2	a1.1.0	\N	2010-09-12 22:00:00+00	{"type": "alpha", "major": false}
795	2	a1.0.17_04	\N	2010-08-22 22:00:00+00	{"type": "alpha", "major": false}
797	2	a1.0.16	\N	2010-08-11 22:00:00+00	{"type": "alpha", "major": false}
798	2	a1.0.15	\N	2010-08-03 22:00:00+00	{"type": "alpha", "major": false}
800	2	a1.0.11	\N	2010-07-22 22:00:00+00	{"type": "alpha", "major": false}
801	2	a1.0.5_01	\N	2010-07-12 22:00:00+00	{"type": "alpha", "major": false}
803	2	inf-20100618	\N	2010-06-15 22:00:00+00	{"type": "alpha", "major": false}
804	2	c0.30_01c	\N	2009-12-21 22:00:00+00	{"type": "alpha", "major": false}
806	2	c0.0.13a_03	\N	2009-05-21 22:00:00+00	{"type": "alpha", "major": false}
807	2	c0.0.11a	\N	2009-05-16 22:00:00+00	{"type": "alpha", "major": false}
809	2	rd-160052	\N	2009-05-15 22:52:00+00	{"type": "alpha", "major": false}
810	2	rd-20090515	\N	2009-05-14 22:00:00+00	{"type": "alpha", "major": false}
812	2	rd-132211	\N	2009-05-13 20:11:00+00	{"type": "alpha", "major": false}
11	2	1.21.5-pre1	\N	2025-03-11 12:49:44+00	{"type": "snapshot", "major": false}
12	2	25w10a	\N	2025-03-05 13:11:13+00	{"type": "snapshot", "major": false}
\.


COPY public.loader_fields_loaders (loader_id, loader_field_id) FROM stdin;
1	3
3	3
2	3
4	3
5	3
6	3
22	3
23	4
23	3
7	3
9	3
16	3
17	3
18	3
19	3
19	9
20	3
15	3
21	3
13	3
10	3
11	3
8	3
12	3
14	3
24	3
25	3
26	3
27	3
28	3
29	3
1	9
3	9
2	9
4	9
5	9
6	9
22	9
23	9
24	9
25	9
26	9
27	9
28	9
29	9
\.


-- For some reason, the actual staging DB has different IDs for project types
-- from ID 3 onwards. For consistency, let's keep them as-is and just add the
-- new ones from staging not present in new databases
COPY public.project_types (id, name) FROM stdin;
5	resourcepack
6	shader
\.


-- Mapping of loader IDs from staging to a new DB:
-- 3 (resourcepack) -> 5
-- 4 (shader) -> 6
-- 5 (plugin) -> 3
-- 6 (datapack) -> 4
COPY public.loaders_project_types (joining_loader_id, joining_project_type_id) FROM stdin;
1	1
2	1
3	1
4	1
5	1
6	1
7	5
16	6
17	6
18	6
19	6
22	1
23	2
20	4
9	3
15	3
21	3
13	3
10	3
11	3
8	3
12	3
14	3
24	1
25	1
26	1
27	1
28	1
29	1
\.


COPY public.loaders_project_types_games (loader_id, project_type_id, game_id) FROM stdin;
1	1	1
2	1	1
3	1	1
4	1	1
5	1	1
6	1	1
7	5	1
8	1	1
9	1	1
10	1	1
11	1	1
12	1	1
13	1	1
14	1	1
15	1	1
16	6	1
17	6	1
18	6	1
19	6	1
20	1	1
21	1	1
22	1	1
23	2	1
20	6	1
9	3	1
15	3	1
21	3	1
13	3	1
10	3	1
11	3	1
8	3	1
12	3	1
14	3	1
24	1	1
25	1	1
26	1	1
27	1	1
28	1	1
29	1	1
\.


COPY public.users (id, github_id, username, email, avatar_url, bio, created, role, badges, balance, discord_id, gitlab_id, google_id, steam_id, microsoft_id, password, email_verified, totp_secret, paypal_country, paypal_email, paypal_id, venmo_handle, stripe_customer_id, raw_avatar_url, allow_friend_requests) FROM stdin;
103587649610509	\N	Default admin user	admin@modrinth.invalid	https://avatars.githubusercontent.com/u/106493074	$ chmod 777 labrinth	2020-07-18 16:03:00.000000+00	admin	0	0.00000000000000000000	\N	\N	\N	\N	\N	$argon2i$v=19$m=4096,t=3,p=1$c2FsdEl0V2l0aFNhbHQ$xTGvQNICqetaNA0Wu1GwFmYhQjAreRcjBz6ornhaFXA	t	\N	\N	\N	\N	\N	\N	https://avatars.githubusercontent.com/u/106493074	t
\.

INSERT INTO sessions (id, session, user_id, created, last_login, expires, refresh_expires, city, country, ip, os, platform, user_agent)
VALUES (93083445641246, 'mra_admin', 103587649610509, '2025-10-20 14:58:53.128901+00', '2025-10-20 14:58:53.128901+00', '2025-11-03 14:58:53.128901+00', '2025-12-19 14:58:53.128901+00', '', '', '127.0.0.1', 'Linux', 'Chrome', 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36');

COMMIT;
