from collections import defaultdict

from handlers.photo_metadata_handler import PhotoMetadataHandler, Metadata
from handlers.folium_map_handler import FoliumMapHandler

import os
import webbrowser
import re
import tkinter
from tkinter import filedialog

photoHandler = PhotoMetadataHandler()
renderHandler = FoliumMapHandler()


def natural_sort(l):
    # Stolen from https://stackoverflow.com/questions/4836710/is-there-a-built-in-function-for-string-natural-sort
    convert = lambda text: int(text) if text.isdigit() else text.lower()
    alphanum_key = lambda key: [convert(c) for c in re.split("([0-9]+)", key)]
    return sorted(l, key=alphanum_key)


def folder_picker():
    root = tkinter.Tk()
    root.withdraw()
    file_path = filedialog.askdirectory()
    return file_path


def get_files_by_folder(root_dir):
    result = {}

    # Iterate only the first level of folders
    top_level_folders = natural_sort(
        [e for e in os.listdir(root_dir) if os.path.isdir(os.path.join(root_dir, e))]
    )

    for folder_name in top_level_folders:
        folder_path = os.path.join(root_dir, folder_name)
        files = []

        for dirpath, _, filenames in os.walk(folder_path):
            for fname in filenames:
                files.append(os.path.join(dirpath, fname))

        result[folder_name] = files

    return result


def main():
    # Get Images
    directory = folder_picker()
    folder_files = get_files_by_folder(directory)

    # Get Metadata
    metadata_by_folder: dict[str, list[Metadata]] = {}
    for folder, files in folder_files.items():
        metadata_by_folder[folder] = photoHandler.grab_metadata(files, directory)

    # Build The Map
    for folder_name, file_metadata in metadata_by_folder.items():
        gps_groups = defaultdict(list)
        for idx, metadata in enumerate(file_metadata):
            gps_groups[tuple(metadata.GPS)].append((idx, metadata))

        renderHandler.add_feature_group_and_set_context(folder_name)
        renderHandler.add_gps_coords_as_markers(gps_groups)

    renderHandler.finalize_map()

    # Render / Save the Map
    file_path = f"{directory}/map.html"

    renderHandler.render_map(file_path)

    webbrowser.open(f"file://{os.path.abspath(file_path)}", new=2)


if __name__ == "__main__":
    main()
