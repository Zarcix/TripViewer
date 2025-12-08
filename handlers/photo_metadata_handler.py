from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from dataclasses import dataclass

import exiftool
import os


@dataclass
class Metadata:
    ParsedDate: str
    GPS: list[float, float]
    Path: str


class PhotoMetadataHandler:
    def __init__(self):
        pass

    def grab_metadata(self, files) -> list[Metadata]:
        et = exiftool.ExifToolHelper()

        def __parse_metadata(metadata):
            dt = metadata.get("EXIF:DateTimeOriginal") or metadata.get(
                "QuickTime:CreateDate"
            )
            ofs = metadata.get("EXIF:OffsetTimeDigitized") or "+00:00"

            # Skip invalid EXIF cases
            if not dt:
                metadata["ParsedDate"] = None
                return metadata

            dt = dt.replace("T", " ").replace("Z", "")
            combined = f"{dt}{ofs}"

            try:
                local_dt = datetime.strptime(combined, "%Y:%m:%d %H:%M:%S%z")
                utc_dt = local_dt.astimezone(timezone.utc)
                metadata["ParsedDate"] = utc_dt
            except Exception:
                metadata["ParsedDate"] = None

            return metadata

        breakpoint()
        metadatas = et.get_metadata(files)
        with ThreadPoolExecutor(max_workers=os.cpu_count()) as executor:
            metadatas = list(executor.map(__parse_metadata, metadatas))

        metadatas = sorted(
            metadatas,
            key=lambda m: m.get("ParsedDate")
            or datetime.min.replace(tzinfo=timezone.utc),
        )

        filtered = [
            Metadata(
                ParsedDate=m.get("ParsedDate"),
                GPS=list(
                    map(
                        float,
                        (m.get("Composite:GPSPosition") or "0.0000 0.0000").split(" "),
                    )
                ),
                Path=m.get("SourceFile"),
            )
            for m in metadatas
        ]

        return filtered
