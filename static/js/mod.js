let currentlySelected = document.getElementById("description-bar");
let currentlySelectedDiv = document.getElementById("description");

function toggleSection(element) {
    currentlySelected.classList.remove("mod-bar-active");
    currentlySelected = element;
    currentlySelected.classList.add("mod-bar-active");
}