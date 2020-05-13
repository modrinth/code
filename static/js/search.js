let input = document.getElementById("search-input");

let category_inputs = {
    "technology": false,
    "adventure": false,
    "magic": false,
    "utility": false,
    "decoration": false,
    "library": false,
    "worldgen": false,
    "cursed": false,
    "forge": false,
    "fabric": false,
}

let resultContainer = document.getElementById("results");

window.onload = function () {
    let categories = document.getElementsByClassName("category-badge");

    for (let category of categories) {
        let ghost = document.createElement('div');
        ghost.className = "category-ghost";
        ghost.id = category.id + "-ghost";

        category.appendChild(ghost);
    }
}

function activateCategory(element) {
    category_inputs[element.id] = !category_inputs[element.id]

    if (category_inputs[element.id]) {
        element.style.width = "155px";
        element.style.boxShadow = "10px 0 " + element.style.color;

        document.getElementById(element.id + "-ghost").className = "";
    } else {
        element.style.width = "165px";
        element.style.boxShadow = "0 0";

        document.getElementById(element.id + "-ghost").className = "category-ghost";
    }

    handleSearch();
}

function handleSearch() {
    let safeName = encodeURIComponent(input.value).replace(/%20/g,'+');

    let queryString = "search?q=" + safeName;
    let filterString = "";

    for (let key in category_inputs) {
        if (category_inputs.hasOwnProperty(key)) {

            if(category_inputs[key])
                filterString += key + " AND keywords=";
        }
    }

    let takeOffLength = " AND keywords=".length;

    if(filterString.length > takeOffLength) {
        filterString = filterString.substring(0, filterString.length - takeOffLength)
        queryString += "&f=" + encodeURIComponent( "keywords=" + filterString).replace(/%20/g,'+');
    }

    let xmlHttp = new XMLHttpRequest();

    xmlHttp.onreadystatechange = function() {
        if (xmlHttp.readyState === 4 && xmlHttp.status === 200) {
            resultContainer.innerHTML = xmlHttp.responseText;
        }
    }

    xmlHttp.open("POST", queryString, true);
    xmlHttp.send(null);

    window.history.pushState('Search', 'Search', queryString);
}