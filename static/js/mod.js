let currentlySelected = document.getElementById("description-bar");
let currentlySelectedDiv = document.getElementById("description");

function toggleSection(element) {
    currentlySelected.classList.remove("mod-bar-active");
    currentlySelected = element;
    currentlySelected.classList.add("mod-bar-active");

    currentlySelectedDiv.classList.remove("mod-show");
    currentlySelectedDiv.classList.add("mod-hide");

    currentlySelectedDiv = document.getElementById(element.id.replace("-bar", ""));

    currentlySelectedDiv.classList.remove("mod-hide");
    currentlySelectedDiv.classList.add("mod-show");
}

hljs.initHighlightingOnLoad();