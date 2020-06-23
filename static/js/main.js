window.onload = function () {
	if (localStorage.getItem("theme")) {
		document.documentElement.setAttribute("theme", localStorage.getItem("theme"));
	}
}