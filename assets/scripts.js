window.addEventListener("load", () => {
    document.body.addEventListener("htmx:beforeOnLoad", (e) => {
        if (e.detail.xhr.status === 422) {
            e.detail.shouldSwap = true;
            e.detail.isError = false;
        }
    });
});
