import { get_apiPath, get_photosets, head_photosets, create_photoset, update_photoset, put_photoset, delete_photoset } from "../api.js";
import { getToken } from "../localstorage.js";

// --- Auth Helper ---
// Generates a URL with the token attached for <img> and <video> tags
function getAuthMediaUrl(path) {
    const token = getToken();
    const url = get_apiPath() + path;
    return token ? `${url}?token=${encodeURIComponent(token)}` : url;
}

// --- History Management ---
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
    historyStack = [];
    currentDir = "/";

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


// --- Upload Logic ---
function createUploadRow(file) {
    const row = document.createElement("div");
    row.className = "upload-row";

    const name = document.createElement("span");
    name.textContent = file.name;

    const progress = document.createElement("progress");
    progress.value = 0;
    progress.max = 100;

    const status = document.createElement("span");
    status.textContent = "0%";

    const cancelBtn = document.createElement("button");
    cancelBtn.textContent = "Cancel";

    row.appendChild(name);
    row.appendChild(progress);
    row.appendChild(status);
    row.appendChild(cancelBtn);

    return { row, progress, status, cancelBtn };
}

async function uploadPhotoset() {
    const input = document.getElementById("fileUpload");
    const container = document.getElementById("uploadContainer");
    const uploadButton = document.getElementById("uploadBtn");

    const files = input.files;

    if (!files || files.length === 0) {
        console.warn("No files selected");
        return;
    }

    container.hidden = false;
    uploadButton.hidden = true;
    container.innerHTML = ""; // reset UI

    const normalizedDir = currentDir.endsWith("/")
        ? currentDir
        : currentDir + "/";

    const uploadPromises = [...files].map(file => {
        const targetPath = normalizedDir + file.name;

        // 🔧 Create UI for this file
        const { row, progress, status, cancelBtn } = createUploadRow(file);
        container.appendChild(row);

        let lastLoaded = 0;
        let cancelled = false;

        const { promise, xhr } = put_photoset(
            targetPath,
            file,
            (loaded, total) => {
                if (total > 0) {
                    const percent = Math.round((loaded / total) * 100);
                    progress.value = percent;
                    status.textContent = `${percent}%`;
                }
                lastLoaded = loaded;
            }
        );

        // 🔴 Per-file cancel
        cancelBtn.onclick = () => {
            cancelled = true;
            xhr.abort();
            status.textContent = "Cancelled";
            progress.value = 0;
        };

        return promise
            .then(res => {
                if (!cancelled) {
                    if (res.ok) {
                        progress.value = 100;
                        status.textContent = "Done";
                    } else {
                        status.textContent = `Error (${res.status})`;
                    }
                }

                return {
                    fileName: file.name,
                    ok: res.ok,
                    status: res.status
                };
            })
            .catch(err => {
                if (!cancelled) {
                    status.textContent = "Failed";
                }

                return {
                    fileName: file.name,
                    ok: false,
                    status: err.message
                };
            });
    });

    const results = await Promise.all(uploadPromises);

    // Log failures
    for (const result of results) {
        if (!result.ok) {
            console.error(
                `Could not upload ${result.fileName}, status=${result.status}`
            );
        }
    }

    input.value = "";
    await loadPhotoset();
    uploadButton.hidden = false;

    // Optional: auto-hide if everything succeeded
    const allOk = results.every(r => r.ok);
    if (allOk) {
        setTimeout(() => {
            container.hidden = true;
        }, 800);
    }
}

// --- Open Logic ---
async function openPhotoset(name) {
    pushToHistory(name);

    // Load the new photoset
    await loadPhotoset();
}

// --- Create Logic ---
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

// --- Delete Logic ---
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

// --- Update Logic ---
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

// --- Loading Logic ---
const mediaObserver = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        const container = entry.target;
        const entryName = container.dataset.entryName;

        if (entry.isIntersecting) {
            if (container.dataset.loaded === "false") {
                // Create a new controller for this specific load attempt
                const controller = new AbortController();
                container._abortController = controller; // Store it on the element
                
                renderMediaIntoContainer(entryName, container, controller.signal);
                container.dataset.loaded = "true";
            }
        } else {
            if (container.dataset.loaded === "true") {
                // 1. If there is an active fetch, cancel it immediately
                if (container._abortController) {
                    container._abortController.abort();
                    container._abortController = null;
                }

                // 2. Clear the DOM
                unloadMediaFromContainer(container);
                container.dataset.loaded = "false";
            }
        }
    });
}, {
    rootMargin: '1000px 0px 1000px 0px' 
});

function unloadMediaFromContainer(container) {
    // Optimization: If there's a video, pause it and clear src before removing
    const video = container.querySelector('video');
    if (video) {
        video.pause();
        video.src = "";
        video.load();
    }
    
    // Clear the DOM and show the placeholder text again
    container.innerHTML = "Scrolling into view...";
}

async function renderMediaIntoContainer(entryName, container, signal) {
    const entryPath = currentDir + (currentDir.endsWith("/") ? "" : "/") + entryName;
    
    try {
        // Pass the signal to your API call
        // Note: You need to update your head_photosets function to accept a signal!
        const res = await head_photosets(entryPath, { signal });
        
        const contentType = res.headers.get("Content-Type") || "";
        const mediaUrl = getAuthMediaUrl(entryPath);

        // Check if we were aborted while waiting for the HEAD response
        if (signal.aborted) return;

        container.textContent = "";

        if (contentType.startsWith("image/")) {
            const img = document.createElement("img");
            // For images, we can't easily cancel the 'src' fetch via AbortSignal,
            // but by not appending it to the DOM if aborted, we save some overhead.
            img.src = mediaUrl;
            img.loading = "lazy";
            img.className = "photoset-image";
            container.appendChild(img);
        } else if (contentType.startsWith("video/")) {
            const video = document.createElement("video");
            video.src = mediaUrl;
            video.controls = true;
            video.preload = "metadata";
            video.className = "photoset-video";
            container.appendChild(video);
        }
        
        // Clear the controller reference once done successfully
        container._abortController = null;

    } catch (err) {
        if (err.name === 'AbortError') {
            console.log(`Fetch cancelled for: ${entryName}`);
        } else {
            container.textContent = "Error loading";
            container.dataset.loaded = "false";
        }
    }
}

async function loadPhotosetDir(response) {
    const photosetJson = await response.json();
    const fragment = document.createDocumentFragment();

    photosetJson.entries.forEach(entry => {
        const entryDiv = document.createElement('div');
        entryDiv.className = 'photoset-entry';

        const topBar = document.createElement('div');
        topBar.className = 'photoset-header';
        topBar.innerHTML = `<span class="photoset-name">${entry.name}</span>`;

        const btnGroup = document.createElement('div');
        btnGroup.className = 'photoset-buttons';
        
        const openBtn = document.createElement('button');
        openBtn.textContent = 'Open';
        openBtn.onclick = () => openPhotoset(entry.name);

        const updateBtn = document.createElement('button');
        updateBtn.textContent = 'Update';
        updateBtn.addEventListener('click', () => updatePhotoset(entry.name));

        const delBtn = document.createElement('button');
        delBtn.textContent = 'Delete';
        delBtn.onclick = () => deletePhotoset(entry.name);

        btnGroup.append(openBtn, updateBtn, delBtn);
        entryDiv.append(topBar, btnGroup);

        // Placeholder for the sliding window
        const mediaContainer = document.createElement("div");
        mediaContainer.className = "photoset-media-container";
        mediaContainer.dataset.entryName = entry.name;
        mediaContainer.dataset.loaded = "false";
        mediaContainer.textContent = "Loading...";
        
        entryDiv.appendChild(mediaContainer);
        fragment.appendChild(entryDiv);

        mediaObserver.observe(mediaContainer);
    });

    return fragment;
}

function createMediaElement(path, contentType = "") {
    const authenticatedUrl = getAuthMediaUrl(path);
    const lower = path.toLowerCase();

    // 1. Images
    if (contentType.startsWith("image/") || /\.(jpg|jpeg|png|gif|webp|svg)$/i.test(lower)) {
        const img = document.createElement("img");
        img.src = authenticatedUrl;
        img.loading = "lazy";
        img.className = "photoset-image";
        return img;
    }

    // 2. Videos
    if (contentType.startsWith("video/") || /\.(mp4|webm|mov|ogg)$/i.test(lower)) {
        const video = document.createElement("video");
        video.src = authenticatedUrl;
        video.controls = true;
        video.preload = "metadata";
        video.playsInline = true;
        video.className = "photoset-video";
        return video;
    }

    // 3. Documents (Your updated iframe logic)
    if (contentType.includes("text/html") || contentType.includes("text/plain") || /\.(txt|html?)$/i.test(lower)) {
        const iframe = document.createElement("iframe");
        iframe.src = authenticatedUrl;
        iframe.classList.add("photoset-iframe");

        return iframe;
    }

    // 4. Fallback: Download Link
    const link = document.createElement("a");
    link.href = authenticatedUrl;
    link.textContent = `Download ${path.split('/').pop()}`;
    link.className = "photoset-download-link";
    link.download = "";
    return link;
}

export async function loadPhotoset() {
    const photosetList = document.getElementById("photosetList");
    photosetList.innerHTML = "Processing...";
    
    try {
        const photosetInfo = await head_photosets(currentDir);
        const contentType = photosetInfo.headers.get("Content-Type") || "";
        photosetList.innerHTML = ""; 

        let newContent;
        if (contentType.includes("application/json")) {
            const response = await get_photosets(currentDir);
            newContent = await loadPhotosetDir(response);
            photosetList.classList.add("grid");
        } else {
            // Single File View
            newContent = document.createElement("div");
            newContent.className = "single-media-wrapper";
            newContent.appendChild(createMediaElement(currentDir, contentType));
            photosetList.classList.remove("grid");
        }

        photosetList.appendChild(newContent);
    } catch (err) {
        photosetList.textContent = "Failed to load photoset.";
    }
}

async function navBack() {
    popFromHistory();
    loadPhotoset();
}