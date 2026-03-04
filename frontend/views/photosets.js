import { get_apiPath, get_photosets, head_photosets, create_photoset, update_photoset, put_photoset, delete_photoset } from "../api.js";

let historyStack = [];
let currentDir = "/"; // root

function pushToHistory(folderName) {
    historyStack.push(folderName);
    updateCurrentDir();
}

function popFromHistory() {
    if (historyStack.length > 0) {
        historyStack.pop();
        updateCurrentDir();
    }
}

// Rebuild currentDir from historyStack
function updateCurrentDir() {
    if (historyStack.length === 0) {
        currentDir = "/";
    } else {
        currentDir = "/" + historyStack.join("/");
    }

    let photosetPath = document.getElementById("photosetPath");
    if (photosetPath) {
        photosetPath.value = currentDir;
    }
}

export async function initPhotoSets() {
    // Path Input
    let photosetPath = document.getElementById("photosetPath");
    photosetPath.value = currentDir;
    photosetPath.addEventListener("keypress", function(event) {
        if (event.key === "Enter") {
            event.preventDefault();

            // Normalize input: remove leading/trailing slashes and split
            const path = photosetPath.value.trim();
            const parts = path.split("/").filter(Boolean); // remove empty parts

            // Reset history stack to reflect the input path
            historyStack = [...parts];

            updateCurrentDir();

            // Load the new photoset
            loadPhotoset();
        }
    });

    // Back Button
    let backButton = document.getElementById("backBtn");
    backButton.onclick = () => navBack();

    // Reload Button
    let reloadButton = document.getElementById("reloadBtn");
    reloadButton.onclick = async () => await loadPhotoset();

    // Create Button
    let createButton = document.getElementById("createBtn")
    createButton.onclick = async () => await createPhotoset();

    // Upload Button
    let uploadButton = document.getElementById("uploadBtn");
    uploadButton.onclick = async () => await uploadPhotoset();

    await loadPhotoset();
}

async function uploadPhotoset() {
    const input = document.getElementById("fileUpload");
    const files = input.files;

    if (!files || files.length === 0) {
        console.warn("No files selected");
        return;
    }

    const normalizedDir = currentDir.endsWith("/")
        ? currentDir
        : currentDir + "/";

    // Spawn all uploads immediately
    const uploadPromises = [...files].map(file => {
        const targetPath = normalizedDir + file.name;

        return put_photoset(targetPath, file)
            .then(res => ({
                fileName: file.name,
                ok: res.ok,
                status: res.status
            }))
            .catch(err => ({
                fileName: file.name,
                ok: false,
                status: "network error",
                error: err
            }));
    });

    // Wait for all to complete
    const results = await Promise.all(uploadPromises);

    // Process failures
    for (const result of results) {
        if (!result.ok) {
            console.error(
                `Could not upload ${result.fileName}, status=${result.status}`
            );
        }
    }

    input.value = "";
    await loadPhotoset();
}

async function openPhotoset(name) {
    pushToHistory(name);

    // Load the new photoset
    await loadPhotoset();
}

async function createPhotoset() {
    let name = document.getElementById("createName").value;

    const createDir = currentDir + "/" + name;
    console.log("Creating on " + createDir);
    let create_res = await create_photoset(createDir);
    if (create_res.ok) {
        return await loadPhotoset();
    }

    switch (create_res.status) {
        case 400:
            alert("Invalid Photoset Name.")
            break;
        default:
            console.error("Could not create Photoset at " + createDir);
            break;
    }
}

async function deletePhotoset(name) {
    const deleteDir = "/" + historyStack.join("/") + "/" + name;

    let res = await delete_photoset(deleteDir);
    if (res.ok) {
        await loadPhotoset();
        return;
    }

    switch (res.status) {
        case 409:
            alert("Photoset not empty.")
            break;
        default:
            console.error("Could not delete photoset.")
            break;
    }
}

async function updatePhotoset(entry) {
    let answer = prompt("New Name");
    if (answer == null) {
        return;
    }

    let entryPath = currentDir + "/" + entry;

    let res = await update_photoset(entryPath, answer);
    if (res.ok) {
        return await loadPhotoset();
    }

    switch (res.status) {
        case 403:
            alert("That name is not allowed.")
            break;
        default:
            console.error("Could not rename photoset.")
            break;
    }
}

function loadPhotosetImage(image_path) {
    let mediaElement = document.createElement("img");
    mediaElement.src = image_path;
    mediaElement.loading = "lazy";
    mediaElement.classList.add("photoset-image");
    return mediaElement;
}

function loadPhotosetVideo(video_path) {
    let mediaElement = document.createElement("video");
    mediaElement.src = video_path;
    mediaElement.controls = true;
    mediaElement.preload = "metadata";
    mediaElement.playsInline = true;
    mediaElement.classList.add("photoset-video");

    return mediaElement
}

function loadPhotosetFile(file_path) {
    const fragment = document.createDocumentFragment();

    const lower = file_path.toLowerCase();

    let mediaElement;

    if (/\.(jpg|jpeg|png|gif|webp|bmp|svg)$/i.test(lower)) {
        mediaElement = loadPhotosetImage(file_path);
    }
    else if (/\.(mp4|webm|ogg|mov)$/i.test(lower)) {
        mediaElement = loadPhotosetVideo(file_path);
    }
    else if (/\.(txt|html?)$/i.test(lower)) {
        mediaElement = document.createElement("iframe");
        mediaElement.src = file_path;
        mediaElement.classList.add("photoset-iframe");
        mediaElement.style.width = "100%";
        mediaElement.style.height = "600px";
        mediaElement.style.border = "none";
    }
    else {
        const link = document.createElement("a");
        link.href = file_path;
        link.textContent = "Download file";
        link.download = "";
        fragment.appendChild(link);
        return fragment;
    }

    fragment.appendChild(mediaElement);
    return fragment;
}

async function loadMediaPreview(entryName, container) {
    let entryPath = currentDir + "/" + entryName;
    try {
        const headResponse = await head_photosets(entryPath);
        const contentType = headResponse.headers.get("Content-Type");

        container.textContent = ""; // clear placeholder

        if (contentType && contentType.startsWith("image/")) {
            const img = loadPhotosetImage(get_apiPath() + entryPath);
            container.appendChild(img);
        } 
        else if (contentType && contentType.startsWith("video/")) {
            const video = loadPhotosetVideo(get_apiPath() + entryPath);
            container.appendChild(video);
        } 
        else {
            container.textContent = "";
        }

    } catch (err) {
        container.textContent = "Failed to load preview";
        console.error("Media preview error:", err);
    }
}

async function loadPhotosetDir(response) {
    const photosetJson = await response.json();
    const entries = photosetJson.entries;

    const fragment = document.createDocumentFragment();

    for (const entry of entries) {
        const entryDiv = document.createElement('div');
        entryDiv.classList.add('photoset-entry');

        const nameSpan = document.createElement('span');
        nameSpan.textContent = entry.name;
        nameSpan.classList.add('photoset-name');

        const buttonDiv = document.createElement('div');
        buttonDiv.classList.add('photoset-buttons');

        const openBtn = document.createElement('button');
        openBtn.textContent = 'Open';
        openBtn.addEventListener('click', () => openPhotoset(entry.name));

        const updateBtn = document.createElement('button');
        updateBtn.textContent = 'Update';
        updateBtn.addEventListener('click', () => updatePhotoset(entry.name));

        const deleteBtn = document.createElement('button');
        deleteBtn.textContent = 'Delete';
        deleteBtn.addEventListener('click', () => deletePhotoset(entry.name));

        buttonDiv.append(openBtn, updateBtn, deleteBtn);
        entryDiv.append(nameSpan, buttonDiv);
        fragment.appendChild(entryDiv);

        const mediaContainer = document.createElement("div");
        mediaContainer.classList.add("photoset-media");
        mediaContainer.textContent = "Loading preview...";

        entryDiv.append(nameSpan, buttonDiv, mediaContainer);
        fragment.appendChild(entryDiv);
        loadMediaPreview(entry.name, mediaContainer);
    }

    return fragment;
}

async function loadPhotoset() {
    let photosetInfo = await head_photosets(currentDir);
    let contentType = photosetInfo.headers.get("content-type");

    let photosetList = document.getElementById("photosetList");
    photosetList.classList = []
    let newContent = null;

    if (contentType != null && contentType.indexOf("application/json") !== -1) {
        let photosetDir = await get_photosets(currentDir);
        newContent = await loadPhotosetDir(photosetDir);
        photosetList.classList.add("grid")
    } else {
        let serverPath = get_apiPath() + currentDir;
        newContent = loadPhotosetFile(serverPath);
    }

    photosetList.replaceChildren(newContent);
}

async function navBack() {
    popFromHistory();
    loadPhotoset();
}