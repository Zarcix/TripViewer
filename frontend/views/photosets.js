import { request } from "../api.js";
import { getServer, getToken } from "../localstorage.js";

let currentPath = "";
let historyStack = [];

export async function initPhotoSets() {
    // document.getElementById("createBtn").onclick = () => createPhotoSet();
    document.getElementById("backBtn").onclick = () => navigateBack();
    renderPhotoSets(currentPath);
}


async function renderPhotoSets(path) {
    currentPath = path;
    document.getElementById("photosetPath").textContent = "/" + path;
    const server_ip = getServer();
    const token = getToken();

    // Get Photos
    let list = [];
    try {
        const res = await request("GET", `/${token}/photos/PhotoSets/` + path);
        list = await res.json();
    } catch {
        list = [];
    }

    // Render Photos
    const container = document.getElementById("photosetList");
    let preview_cont = document.getElementById("photosetPreview");
    preview_cont.hidden = true;
    container.innerHTML = "";

    if (list.length === 0) {
        preview_cont.hidden = false;
        preview_cont.innerHTML = `
<iframe 
    src="${server_ip}/${token}/photos/PhotoSets/${currentPath}"
    class="embedded-photo-frame"
    title="Photo Browser">
</iframe>`;
        return;
    }

    list.forEach(name => {
        const newPath = path ? `${path}/${name}` : name;
        const card = document.createElement("div");

        // --- Step 1: try image ---
        const img = document.createElement("img");
        img.src = `${server_ip}/${token}/photos/PhotoSets/${newPath}`;
        img.width = 180;
        img.style.objectFit = "cover";
        img.style.marginBottom = "8px";

        img.onload = () => {
            // Loaded as image
            const del = document.createElement("button");
            del.textContent = "Delete";
            del.style.marginTop = "4px";
            del.onclick = () => deletePhoto(newPath);

            card.appendChild(img);
            card.appendChild(del);
        };

        img.onerror = () => {
            // --- Step 2: try video ---
            const video = document.createElement("video");
            video.src = `${server_ip}/${token}/photos/PhotoSets/${newPath}`;
            video.controls = true;
            video.width = 180;

            video.onloadeddata = () => {
                // Loaded as video
                const del = document.createElement("button");
                del.textContent = "Delete";
                del.style.marginTop = "4px";
                del.onclick = () => deletePhoto(newPath);

                card.appendChild(video);
                card.appendChild(del);
            };

            video.onerror = () => {
                // --- Step 3: fallback to photoset ---
                card.innerHTML = `
                    <div class="photoset-name"><b>${name}</b></div>
                    <div class="photoset-actions">
                        <button class="openBtn">Open</button>
                        <button class="renameBtn">Rename</button>
                        <button class="deleteBtn">Delete</button>
                    </div>
                `;
                card.querySelector(".openBtn").onclick = () => {
                    historyStack.push(currentPath);
                    renderPhotoSets(newPath);
                };
                card.querySelector(".renameBtn").onclick = () => renamePhotoSet(newPath);
                card.querySelector(".deleteBtn").onclick = () => deletePhotoSet(newPath);
            };
        };

        container.appendChild(card);
    });
}

async function renamePhotoSet(path) {
    const newName = prompt("New Name:");
    if (!newName) return;

    const form = new FormData();
    form.append("new_name", newName);

    try {
        const res = await request("PATCH", "/api/photoset/" + path, form, true);
        if (!res.ok) alert("Rename failed");
        else {
            const parent = path.split("/").slice(0, -1).join("/");
            renderPhotoSets(parent || "root");
        }
    } catch (e) {
        alert("Rename failed: " + e.message);
    }
}

function navigateBack() {
    if (historyStack.length === 0) return;
    const previous = historyStack.pop();
    renderPhotoSets(previous);
}