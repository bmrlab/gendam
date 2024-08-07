/// These mapping are generated by Claude, there might be error.
use content_metadata::ContentType;
use phf::phf_map;

static EXTENSION_TO_MIME: phf::Map<&'static str, &'static str> = phf_map! {
    "jpg" => "image/jpeg",
    "jpeg" => "image/jpeg",
    "png" => "image/png",
    "gif" => "image/gif",
    "bmp" => "image/bmp",
    "tiff" => "image/tiff",
    "webp" => "image/webp",
    "mp3" => "audio/mpeg",
    "wav" => "audio/wav",
    "ogg" => "audio/ogg",
    "flac" => "audio/flac",
    "m4a" => "audio/mp4",
    "mp4" => "video/mp4",
    "avi" => "video/x-msvideo",
    "mov" => "video/quicktime",
    "wmv" => "video/x-ms-wmv",
    "mkv" => "video/x-matroska",
    "flv" => "video/x-flv",
    "webm" => "video/webm",
    "txt" => "text/plain",
    "md" => "text/markdown",
    "markdown" => "text/markdown",
    "json" => "application/json",
    "xml" => "application/xml",
    "html" => "text/html",
    "css" => "text/css",
    "js" => "application/javascript",
    "py" => "text/x-python",
    "java" => "text/x-java-source",
    "c" => "text/x-c",
    "cpp" => "text/x-c++",
    "h" => "text/x-c",
    "sh" => "application/x-sh",
    "bat" => "application/x-bat",
    "log" => "text/plain",
    "csv" => "text/csv",
    // You can uncomment and add more extensions and MIME types as needed
    // "doc" => "application/msword",
    // "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    // "pdf" => "application/pdf",
    // "rtf" => "application/rtf",
    // "odt" => "application/vnd.oasis.opendocument.text",
    // "xls" => "application/vnd.ms-excel",
    // "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    // "ods" => "application/vnd.oasis.opendocument.spreadsheet",
    // "ppt" => "application/vnd.ms-powerpoint",
    // "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    // "odp" => "application/vnd.oasis.opendocument.presentation",
    // "zip" => "application/zip",
    // "rar" => "application/x-rar-compressed",
    // "7z"=> "application/x-7z-compressed",
    // "tar" => "application/x-tar",
    // "gz" => "application/gzip",
    // "exe" => "application/x-msdownload",
    // "msi" => "application/x-msi",
    // "app" => "application/x-apple-diskimage",
    // "deb" => "application/vnd.debian.binary-package",
    // "rpm" => "application/x-rpm",
    // "ttf" => "font/ttf",
    // "otf" => "font/otf",
    // "woff" => "font/woff",
    // "woff2" => "font/woff2",
    // "db" => "application/x-sqlite3",
    // "sql" => "application/sql",
    // "sqlite" => "application/x-sqlite3",
    // "obj" => "model/obj",
    // "stl" => "model/stl",
    // "fbx" => "application/octet-stream",
    // "svg" => "image/svg+xml",
    // "ai" => "application/postscript",
    // "eps" => "application/postscript",
    // "epub" => "application/epub+zip",
    // "mobi" => "application/x-mobipocket-ebook",
    // "azw" => "application/vnd.amazon.ebook",
};

// Static hash map for MIME type to kind mapping
static MIME_TO_KIND: phf::Map<&'static str, ContentType> = phf_map! {
    "image/jpeg" => ContentType::Image,
    "image/png" => ContentType::Image,
    "image/gif" => ContentType::Image,
    "image/bmp" => ContentType::Image,
    "image/tiff" => ContentType::Image,
    "image/webp" => ContentType::Image,
    "audio/mpeg" => ContentType::Audio,
    "audio/wav" => ContentType::Audio,
    "audio/ogg" => ContentType::Audio,
    "audio/flac" => ContentType::Audio,
    "audio/mp4" => ContentType::Audio,
    "video/mp4" => ContentType::Video,
    "video/x-msvideo" => ContentType::Video,
    "video/quicktime" => ContentType::Video,
    "video/x-ms-wmv" => ContentType::Video,
    "video/x-matroska" => ContentType::Video,
    "video/x-flv" => ContentType::Video,
    "video/webm" => ContentType::Video,
    "text/plain" => ContentType::RawText,
    "text/markdown" => ContentType::RawText,
    "application/json" => ContentType::RawText,
    "application/xml" => ContentType::RawText,
    "text/html" => ContentType::RawText,
    "text/css" => ContentType::RawText,
    "application/javascript" => ContentType::RawText,
    "text/x-python" => ContentType::RawText,
    "text/x-java-source" => ContentType::RawText,
    "text/x-c" => ContentType::RawText,
    "text/x-c++" => ContentType::RawText,
    "application/x-sh" => ContentType::RawText,
    "application/x-bat" => ContentType::RawText,
    "text/csv" => ContentType::RawText,
    // "application/msword" => "document",
    // "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "document",
    // "application/pdf" => "document",
    // "application/rtf" => "document",
    // "application/vnd.oasis.opendocument.text" => "document",
    // "application/vnd.ms-excel" => "spreadsheet",
    // "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "spreadsheet",
    // "application/vnd.oasis.opendocument.spreadsheet" => "spreadsheet",
    // "application/vnd.ms-powerpoint" => "presentation",
    // "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "presentation",
    // "application/vnd.oasis.opendocument.presentation" => "presentation",
    // "application/zip" => "archive",
    // "application/x-rar-compressed" => "archive",
    // "application/x-7z-compressed" => "archive",
    // "application/x-tar" => "archive",
    // "application/gzip" => "archive",
    // "application/x-msdownload" => "executable",
    // "application/x-msi" => "executable",
    // "application/x-apple-diskimage" => "executable",
    // "application/vnd.debian.binary-package" => "executable",
    // "application/x-rpm" => "executable",
    // "font/ttf" => "font",
    // "font/otf" => "font",
    // "font/woff" => "font",
    // "font/woff2" => "font",
    // "application/x-sqlite3" => "database",
    // "application/sql" => "database",
    // "model/obj" => "3d",
    // "model/stl" => "3d",
    // "application/octet-stream" => "3d",
    // "image/svg+xml" => "vector",
    // "application/postscript" => "vector",
    // "application/epub+zip" => "ebook",
    // "application/x-mobipocket-ebook" => "ebook",
    // "application/vnd.amazon.ebook" => "ebook",
};


pub fn get_mime_from_extension(extension: &str) -> Option<&str> {
    EXTENSION_TO_MIME.get(extension.trim_start_matches('.')).copied()
}

pub fn get_kind_from_extension(extension: &str) -> Option<ContentType> {
    match EXTENSION_TO_MIME
        .get(extension.trim_start_matches('.'))
        .copied()
    {
        Some(mime) => get_kind_from_mime(mime),
        _ => None,
    }
}

// Function to get file kind from MIME type
pub fn get_kind_from_mime(mime_type: &str) -> Option<ContentType> {
    MIME_TO_KIND.get(mime_type).copied()
}
