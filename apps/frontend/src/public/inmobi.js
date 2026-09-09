;(function () {
	document.documentElement.classList.add('modrinth-cmp-summary-hidden')

	var host = 'modrinth.com'
	// var host = window.location.hostname;
	var element = document.createElement('script')
	var firstScript = document.getElementsByTagName('script')[0]
	var url = 'https://cmp.inmobi.com'.concat(
		'/choice/',
		'GANCBjEfRH5Fe',
		'/',
		host,
		'/choice.js?tag_version=V3',
	)
	var uspTries = 0
	var uspTriesLimit = 3
	element.async = true
	element.type = 'text/javascript'
	element.src = url

	firstScript.parentNode.insertBefore(element, firstScript)

	function makeStub() {
		var TCF_LOCATOR_NAME = '__tcfapiLocator'
		var queue = []
		var win = window
		var cmpFrame

		function addFrame() {
			var doc = win.document
			var otherCMP = !!win.frames[TCF_LOCATOR_NAME]

			if (!otherCMP) {
				if (doc.body) {
					var iframe = doc.createElement('iframe')

					iframe.style.cssText = 'display:none'
					iframe.name = TCF_LOCATOR_NAME
					doc.body.appendChild(iframe)
				} else {
					setTimeout(addFrame, 5)
				}
			}
			return !otherCMP
		}

		function tcfAPIHandler() {
			var gdprApplies
			var args = arguments

			if (!args.length) {
				return queue
			} else if (args[0] === 'setGdprApplies') {
				if (args.length > 3 && args[2] === 2 && typeof args[3] === 'boolean') {
					gdprApplies = args[3]
					if (typeof args[2] === 'function') {
						args[2]('set', true)
					}
				}
			} else if (args[0] === 'ping') {
				var retr = {
					gdprApplies: gdprApplies,
					cmpLoaded: false,
					cmpStatus: 'stub',
				}

				if (typeof args[2] === 'function') {
					args[2](retr)
				}
			} else {
				if (args[0] === 'init' && typeof args[3] === 'object') {
					args[3] = Object.assign(args[3], { tag_version: 'V3' })
				}
				queue.push(args)
			}
		}

		function postMessageEventHandler(event) {
			var msgIsString = typeof event.data === 'string'
			var json = {}

			try {
				if (msgIsString) {
					json = JSON.parse(event.data)
				} else {
					json = event.data
				}
			} catch (ignore) {}

			var payload = json.__tcfapiCall

			if (payload) {
				window.__tcfapi(
					payload.command,
					payload.version,
					function (retValue, success) {
						var returnMsg = {
							__tcfapiReturn: {
								returnValue: retValue,
								success: success,
								callId: payload.callId,
							},
						}
						if (msgIsString) {
							returnMsg = JSON.stringify(returnMsg)
						}
						if (event && event.source && event.source.postMessage) {
							event.source.postMessage(returnMsg, '*')
						}
					},
					payload.parameter,
				)
			}
		}

		while (win) {
			try {
				if (win.frames[TCF_LOCATOR_NAME]) {
					cmpFrame = win
					break
				}
			} catch (ignore) {}

			if (win === window.top) {
				break
			}
			win = win.parent
		}
		if (!cmpFrame) {
			addFrame()
			win.__tcfapi = tcfAPIHandler
			win.addEventListener('message', postMessageEventHandler, false)
		}
	}

	makeStub()

	function makeGppStub() {
		const CMP_ID = 10
		const SUPPORTED_APIS = [
			'2:tcfeuv2',
			'6:uspv1',
			'7:usnatv1',
			'8:usca',
			'9:usvav1',
			'10:uscov1',
			'11:usutv1',
			'12:usctv1',
		]

		window.__gpp_addFrame = function (n) {
			if (!window.frames[n]) {
				if (document.body) {
					var i = document.createElement('iframe')
					i.style.cssText = 'display:none'
					i.name = n
					document.body.appendChild(i)
				} else {
					window.setTimeout(window.__gpp_addFrame, 10, n)
				}
			}
		}
		window.__gpp_stub = function () {
			var b = arguments
			__gpp.queue = __gpp.queue || []
			__gpp.events = __gpp.events || []

			if (!b.length || (b.length == 1 && b[0] == 'queue')) {
				return __gpp.queue
			}

			if (b.length == 1 && b[0] == 'events') {
				return __gpp.events
			}

			var cmd = b[0]
			var clb = b.length > 1 ? b[1] : null
			var par = b.length > 2 ? b[2] : null
			if (cmd === 'ping') {
				clb(
					{
						gppVersion: '1.1', // must be "Version.Subversion", current: "1.1"
						cmpStatus: 'stub', // possible values: stub, loading, loaded, error
						cmpDisplayStatus: 'hidden', // possible values: hidden, visible, disabled
						signalStatus: 'not ready', // possible values: not ready, ready
						supportedAPIs: SUPPORTED_APIS, // list of supported APIs
						cmpId: CMP_ID, // IAB assigned CMP ID, may be 0 during stub/loading
						sectionList: [],
						applicableSections: [-1],
						gppString: '',
						parsedSections: {},
					},
					true,
				)
			} else if (cmd === 'addEventListener') {
				if (!('lastId' in __gpp)) {
					__gpp.lastId = 0
				}
				__gpp.lastId++
				var lnr = __gpp.lastId
				__gpp.events.push({
					id: lnr,
					callback: clb,
					parameter: par,
				})
				clb(
					{
						eventName: 'listenerRegistered',
						listenerId: lnr, // Registered ID of the listener
						data: true, // positive signal
						pingData: {
							gppVersion: '1.1', // must be "Version.Subversion", current: "1.1"
							cmpStatus: 'stub', // possible values: stub, loading, loaded, error
							cmpDisplayStatus: 'hidden', // possible values: hidden, visible, disabled
							signalStatus: 'not ready', // possible values: not ready, ready
							supportedAPIs: SUPPORTED_APIS, // list of supported APIs
							cmpId: CMP_ID, // list of supported APIs
							sectionList: [],
							applicableSections: [-1],
							gppString: '',
							parsedSections: {},
						},
					},
					true,
				)
			} else if (cmd === 'removeEventListener') {
				var success = false
				for (var i = 0; i < __gpp.events.length; i++) {
					if (__gpp.events[i].id == par) {
						__gpp.events.splice(i, 1)
						success = true
						break
					}
				}
				clb(
					{
						eventName: 'listenerRemoved',
						listenerId: par, // Registered ID of the listener
						data: success, // status info
						pingData: {
							gppVersion: '1.1', // must be "Version.Subversion", current: "1.1"
							cmpStatus: 'stub', // possible values: stub, loading, loaded, error
							cmpDisplayStatus: 'hidden', // possible values: hidden, visible, disabled
							signalStatus: 'not ready', // possible values: not ready, ready
							supportedAPIs: SUPPORTED_APIS, // list of supported APIs
							cmpId: CMP_ID, // CMP ID
							sectionList: [],
							applicableSections: [-1],
							gppString: '',
							parsedSections: {},
						},
					},
					true,
				)
			} else if (cmd === 'hasSection') {
				clb(false, true)
			} else if (cmd === 'getSection' || cmd === 'getField') {
				clb(null, true)
			}
			//queue all other commands
			else {
				__gpp.queue.push([].slice.apply(b))
			}
		}
		window.__gpp_msghandler = function (event) {
			var msgIsString = typeof event.data === 'string'
			try {
				var json = msgIsString ? JSON.parse(event.data) : event.data
			} catch (e) {
				var json = null
			}
			if (typeof json === 'object' && json !== null && '__gppCall' in json) {
				var i = json.__gppCall
				window.__gpp(
					i.command,
					function (retValue, success) {
						var returnMsg = {
							__gppReturn: {
								returnValue: retValue,
								success: success,
								callId: i.callId,
							},
						}
						event.source.postMessage(msgIsString ? JSON.stringify(returnMsg) : returnMsg, '*')
					},
					'parameter' in i ? i.parameter : null,
					'version' in i ? i.version : '1.1',
				)
			}
		}
		if (!('__gpp' in window) || typeof window.__gpp !== 'function') {
			window.__gpp = window.__gpp_stub
			window.addEventListener('message', window.__gpp_msghandler, false)
			window.__gpp_addFrame('__gppLocator')
		}
	}

	makeGppStub()

	window.dispatchEvent(new Event('modrinth-cmp-ready'))

	var uspStubFunction = function () {
		var arg = arguments
		if (typeof window.__uspapi !== uspStubFunction) {
			setTimeout(function () {
				if (typeof window.__uspapi !== 'undefined') {
					window.__uspapi.apply(window.__uspapi, arg)
				}
			}, 500)
		}
	}

	var checkIfUspIsReady = function () {
		uspTries++
		if (window.__uspapi === uspStubFunction && uspTries < uspTriesLimit) {
			console.warn('USP is not accessible')
		} else {
			clearInterval(uspInterval)
		}
	}

	if (typeof window.__uspapi === 'undefined') {
		window.__uspapi = uspStubFunction
		var uspInterval = setInterval(checkIfUspIsReady, 6000)
	}
})()
