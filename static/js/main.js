window.onload = function () {
	if (localStorage.getItem("data-theme")) {
		document.documentElement.setAttribute("data-theme", localStorage.getItem("data-theme"));
	}
}

function switchThemes() {
	let switchTheme = localStorage.getItem("data-theme") === "light" ? "dark" : "light";

	localStorage.setItem("data-theme", switchTheme);
	document.documentElement.setAttribute("data-theme", switchTheme);
}

function closePopup(e) {
	e.parentElement.outerHTML = "";
}