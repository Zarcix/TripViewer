import { request } from "../api.js";
import { getServer } from "../localstorage.js";

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

    // Get Photos
    let list = [];
    try {
        const res = await request("GET", "/photos/PhotoSets/" + path);
        list = await res.json();
    } catch {
        list = [];
    }

    // Render Photos
    const container = document.getElementById("photosetList");
    container.innerHTML = "";

    if (list.length === 0) {
        container.innerHTML = `<iframe src="${server_ip}/photos/PhotoSets/${currentPath}" width="500" height="300" title="Embedded Page">
  <p>Your browser does not support iframes.</p>
</iframe>`;
        return;
    }

    list.forEach(name => {
        const newPath = path ? `${path}/${name}` : name;
        const card = document.createElement("div");
        card.className = "photoset-card";
        card.style.display = "flex";
        card.style.flexDirection = "column";
        card.style.alignItems = "center";
        card.style.border = "1px solid #ccc";
        card.style.padding = "8px";
        card.style.borderRadius = "4px";
        card.style.backgroundColor = "#f9f9f9";

        // --- Step 1: try image ---
        const img = document.createElement("img");
        img.src = `${server_ip}/photos/PhotoSets/${newPath}`;
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
            video.src = `${server_ip}/photos/PhotoSets/${newPath}`;
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