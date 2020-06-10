//TODO Reduce repeated code

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

let currentlyBuildSelected = document.getElementById("gradle-code-bar");
let currentlyBuildSelectedDiv = document.getElementById("gradle-code");

function toggleBuildSection(element) {
    currentlyBuildSelected.classList.remove("mod-bar-active");
    currentlyBuildSelected = element;
    currentlyBuildSelected.classList.add("mod-bar-active");

    currentlyBuildSelectedDiv.classList.remove("mod-show");
    currentlyBuildSelectedDiv.classList.add("mod-hide");

    currentlyBuildSelectedDiv = document.getElementById(element.id.replace("-bar", ""));

    currentlyBuildSelectedDiv.classList.remove("mod-hide");
    currentlyBuildSelectedDiv.classList.add("mod-show");
}

let currentlyApiSelected = document.getElementById("curl-code-bar");
let currentlyApiSelectedDiv = document.getElementById("curl-code");

function toggleApiSection(element) {
    currentlyApiSelected.classList.remove("mod-bar-active");
    currentlyApiSelected = element;
    currentlyApiSelected.classList.add("mod-bar-active");

    currentlyApiSelectedDiv.classList.remove("mod-show");
    currentlyApiSelectedDiv.classList.add("mod-hide");

    currentlyApiSelectedDiv = document.getElementById(element.id.replace("-bar", ""));

    currentlyApiSelectedDiv.classList.remove("mod-hide");
    currentlyApiSelectedDiv.classList.add("mod-show");
}

for (let block of document.getElementsByClassName("api-code-block")) {
    hljs.highlightBlock(block);
}