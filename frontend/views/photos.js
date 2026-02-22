import { request } from "../api.js"
import { getServer } from "../localstorage.js"

async function loadPhotos() {
    const res = await request("GET", "/photos/Photos/");
    const list = await res.json();

    const grid = document.getElementById("photoGrid");
    grid.innerHTML = "";

    const server_ip = getServer();

    list.forEach(name => {
        const container = document.createElement("div");
        container.className = "card";

        // Universal preview container
        const media = document.createElement("img");
        media.src = `${server_ip}/photos/Photos/${name}`;
        media.width = 180;

        // fallback if not image
        media.onerror = () => {
            const video = document.createElement("video");
            video.src = `${server_ip}/photos/Photos/${name}`;
            video.controls = true;
            video.width = 180;

            container.replaceChild(video, media);
        };

        const del = document.createElement("button");
        del.textContent = "Delete";
        del.onclick = () => deletePhoto(name);

        container.appendChild(media);
        container.appendChild(del);
        grid.appendChild(container);
    });
}

export async function initPhotos() {
    loadPhotos();
}