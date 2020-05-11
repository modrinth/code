let input = document.getElementById("search-input");
let resultContainer = document.getElementById("results");

function handleSearch() {
    let safeName = encodeURIComponent(input.value).replace(/%20/g,'+');

    let xmlHttp = new XMLHttpRequest();
    xmlHttp.onreadystatechange = function() {
        if (xmlHttp.readyState === 4 && xmlHttp.status === 200) {
            let parsedResponse = JSON.parse(xmlHttp.responseText);
            let contentToSet = "";

            for (let result of parsedResponse.results) {
                contentToSet += `
                        <div class="result gray-border rounded-border">
                            <img src="..." width="75px" height="75px">
                            <div class="result-info">
                                <h2>${result.title}</h2>
                                <p>${result.description}</p>
                            </div>
                        </div>
                    `
            }

            resultContainer.innerHTML = contentToSet;
        }
    }
    xmlHttp.open("POST", "search?q=" + safeName, true);
    xmlHttp.send(null);

    window.history.pushState('Search', 'Search', '/search?q=' + safeName);
}