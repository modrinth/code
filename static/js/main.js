window.onload = function () {
	if (localStorage.getItem("data-theme")) {
		document.documentElement.setAttribute("data-theme", localStorage.getItem("data-theme"));
	}
}

function closePopup(e) {
	e.parentElement.outerHTML = "";
}