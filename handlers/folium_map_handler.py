from collections import defaultdict
from dataclasses import dataclass
from typing import Sequence
from folium.plugins import (
    BeautifyIcon,
    MarkerCluster,
)
from folium import (
    Map,
    Marker,
    FeatureGroup,
    Popup,
    LayerControl,
)

import colorsys
import random

from .file_handler import Metadata

VIDEO_MIME_TYPES = {
    "mp4": "video/mp4",
    "mov": "video/quicktime",
    "mkv": "video/x-matroska",
    "webm": "video/webm",
    "avi": "video/x-msvideo",
    "flv": "video/x-flv",
    "wmv": "video/x-ms-wmv",
    "m4v": "video/x-m4v",
    "mpeg": "video/mpeg",
}


MEDIA_STYLE = """
    max-width: 100%;
    max-height: 100%;
    width: auto;
    height: auto;
    object-fit: contain;
    display: block;
"""

POPUP_STYLE = """
    overflow: auto;
"""

HEADER_MIN_WIDTH = "15vw"

VIDEO_HTML = """
<div>
    <video controls style='{MEDIA_STYLE}'>
        <source src='{path}' type='{mime}'>
    </video>
    <div>
        <a href='{path}' target='_blank' rel='noopener noreferrer'>
            Open video in new tab
        </a>
    </div>
</div>
"""

IMAGE_HTML = """
<div>
    <a href='{path}' target='_blank' rel='noopener noreferrer'>
        <img src='{path}' style='{MEDIA_STYLE}'>
    </a>
</div>
"""

POPUP_HTML = """
<div style='{POPUP_STYLE}'>
    <div style='min-width: {HEADER_MIN_WIDTH};'>
        <h1 style='font-weight: bold'>
            Image at this location:
        </h1>
        <h4>
            Note that you can click on the media below to open it in a new tab
        </h4>
        <h4>
            You can click on the additional details below to view
        </h4>
    </div>

    {MEDIA_HTML}

    <details>
        <summary style='font-weight: bold'>
            ⬘ Additional Details:
        </summary>
        <div>
            Photo Taken: {photo_date}
        </div>
        <div>
            Photo Coordinates: {photo_coords}
        </div>
    </details>
</div>
"""


@dataclass
class FeatureGroupData:
    feature_number: int
    feature_color: str


class FoliumMapHandler:
    def __init__(
        self, starting_location: Sequence[float] | None = [0, 0], zoom: int = 3
    ):
        self.folium_map = Map(
            location=starting_location, min_zoom=zoom, zoom_start=zoom
        )

        self.feature_group_data: dict[FeatureGroup, FeatureGroupData] = {}
        self.feature_group = None
        self.total_feature_count = 0

    def _get_random_feature_color(self):
        h = random.random()
        s = 0.8
        v = 0.9
        r, g, b = [int(x * 255) for x in colorsys.hsv_to_rgb(h, s, v)]
        return f"#{r:02x}{g:02x}{b:02x}"

    def add_feature_group_and_set_context(self, group_name: str, show_by_default=False):
        """
        Creates a new feature group and sets it as the current feature group context.
        This group context will not change until a new feature group is added
        """
        feature_group = FeatureGroup(name=group_name, show=show_by_default).add_to(
            self.folium_map
        )

        self.total_feature_count += 1
        self.feature_group = feature_group

        feature_data = FeatureGroupData(
            feature_number=self.total_feature_count,
            feature_color=self._get_random_feature_color(),
        )
        self.feature_group_data[feature_group] = feature_data

    def set_feature_group_context(self, new_group: FeatureGroup):
        if new_group not in self.feature_group_data.keys():
            raise IndexError("New feature group not found in created feature groups")

        self.feature_group = new_group

    def _make_media_html(self, metadata):
        """Return clickable <img> or <video> HTML depending on file type."""
        path = metadata.Path
        ext = path.lower().rsplit(".", 1)[-1]

        mime = VIDEO_MIME_TYPES.get(ext)
        if mime:
            return VIDEO_HTML.format(MEDIA_STYLE=MEDIA_STYLE, path=path, mime=mime)

        return IMAGE_HTML.format(
            MEDIA_STYLE=MEDIA_STYLE,
            path=path,
        )

    def _build_popup_html(self, metadata: Metadata):
        media_html = self._make_media_html(metadata)
        photo_date = metadata.ParsedDate.strftime("%B %d, %Y at %I:%M %p %Z")
        photo_coords = f"{metadata.GPS[0]:<.5f}, {metadata.GPS[1]:<.5f}"

        popup_html = POPUP_HTML.format(
            POPUP_STYLE=POPUP_STYLE,
            HEADER_MIN_WIDTH=HEADER_MIN_WIDTH,
            MEDIA_HTML=media_html,
            photo_date=photo_date,
            photo_coords=photo_coords,
        )
        return popup_html

    def add_gps_coords_as_markers(self, metadata: dict[Metadata]):
        feature_data = self.feature_group_data[self.feature_group]

        marker_cluster = MarkerCluster().add_to(self.feature_group)
        for index, data in enumerate(metadata, 1):
            html = self._build_popup_html(data)  # Make HTML Here

            # Create objects for the map
            popup = Popup(html, max_width="800%", lazy=True)
            icon = BeautifyIcon(
                border_color=feature_data.feature_color,
                text_color="#00ABDC",
                number=index,
                inner_icon_style="margin-top:0;",
            )

            # Final Marker
            Marker(location=data.GPS, popup=popup, icon=icon).add_to(marker_cluster)

    def finalize_map(self):
        LayerControl().add_to(self.folium_map)

    def render_map(self, save_path):
        self.folium_map.save(save_path)

    def preview_map(self):
        self.folium_map.show_in_browser()
