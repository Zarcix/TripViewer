import { initPhotos } from './views/photos.js';
// import { initPhotoSets } from './js/photosets.js';
import { initSettings } from './views/settings.js';

const content = document.getElementById("app");

async function loadView(name) {
    const res = await fetch(`views/${name}.html`);
    const html = await res.text();
    content.innerHTML = html;

    switch (name) {
        case "photos": {
            await initPhotos();
            break;
        }
        // case "photosets": {
        //     initPhotoSets();
        //     break;
        // }
        case "settings": {
            initSettings();
            break;
        }
    }
}

document.querySelectorAll("[data-view]").forEach(btn => {
    btn.addEventListener("click", () => {
        loadView(btn.dataset.view);
    });
});

// default view
loadView("photos");