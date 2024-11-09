// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "assets.artifacts.image.description", input: ImageRequestPayload, result: string } | 
        { key: "assets.artifacts.raw_text.chunk.content", input: RawTextRequestPayload, result: string } | 
        { key: "assets.artifacts.raw_text.chunk.summarization", input: RawTextRequestPayload, result: string } | 
        { key: "assets.artifacts.video.transcript", input: TranscriptRequestPayload, result: TranscriptResponse } | 
        { key: "assets.get", input: FilePathGetPayload, result: FilePathWithAssetObjectData } | 
        { key: "assets.list", input: FilePathQueryPayload, result: FilePathWithAssetObjectData[] } | 
        { key: "audio.find_by_hash", input: string, result: AudioResp[] } | 
        { key: "libraries.get_library_settings", input: never, result: LibrarySettings } | 
        { key: "libraries.list", input: never, result: LibrariesListResult[] } | 
        { key: "libraries.models.get_model", input: string, result: AIModelResult } | 
        { key: "libraries.models.list", input: never, result: ModelsListResult[] } | 
        { key: "libraries.status", input: never, result: LibraryStatusResult } | 
        { key: "p2p.state", input: never, result: any } | 
        { key: "search.all", input: SearchRequestPayload, result: SearchResultPayload[] } | 
        { key: "search.recommend", input: RecommendRequestPayload, result: SearchResultPayload[] } | 
        { key: "search.suggestions", input: never, result: string[] } | 
        { key: "tasks.get_assets_in_process", input: never, result: FilePath[] } | 
        { key: "tasks.list", input: TaskListRequestPayload, result: FileHandlerTask[] } | 
        { key: "users.get", input: never, result: Auth | null } | 
        { key: "version", input: never, result: string } | 
        { key: "video.player.video_ts", input: VideoPlayerTsRequestPayload, result: VideoPlayerTsResponse },
    mutations: 
        { key: "assets.create_asset_object", input: AssetObjectCreatePayload, result: FilePathWithAssetObjectData } | 
        { key: "assets.create_dir", input: FilePathCreatePayload, result: FilePath } | 
        { key: "assets.create_web_page_object", input: WebPageCreatePayload, result: FilePathWithAssetObjectData } | 
        { key: "assets.delete_file_path", input: FilePathDeletePayload, result: null } | 
        { key: "assets.export_video_segment", input: VideoSegmentExportPayload, result: null } | 
        { key: "assets.move_file_path", input: FilePathMovePayload, result: null } | 
        { key: "assets.process_asset", input: string, result: null } | 
        { key: "assets.process_asset_metadata", input: number, result: null } | 
        { key: "assets.receive_asset", input: AssetObjectReceivePayload, result: null } | 
        { key: "assets.rename_file_path", input: FilePathRenamePayload, result: null } | 
        { key: "audio.batch_export", input: ExportInput[], result: AudioType[] } | 
        { key: "audio.export", input: ExportInput, result: AudioType[] } | 
        { key: "libraries.create", input: string, result: null } | 
        { key: "libraries.load_library", input: string, result: LibraryLoadResult } | 
        { key: "libraries.models.download_model", input: DownloadModelPayload, result: null } | 
        { key: "libraries.models.set_model", input: SetModelPayload, result: null } | 
        { key: "libraries.unload_library", input: any | null, result: null } | 
        { key: "libraries.update_library_settings", input: LibrarySettings, result: null } | 
        { key: "p2p.accept_file_share", input: string, result: AcceptShareOutput } | 
        { key: "p2p.cancel_file_share", input: string, result: any } | 
        { key: "p2p.finish_file_share", input: string, result: string[] } | 
        { key: "p2p.reject_file_share", input: string, result: any } | 
        { key: "p2p.share", input: SharePayload, result: any } | 
        { key: "storage.upload_to_s3", input: UploadPayload, result: null } | 
        { key: "tasks.cancel", input: TaskCancelRequestPayload, result: null } | 
        { key: "tasks.regenerate", input: TaskRedoRequestPayload, result: null } | 
        { key: "users.set", input: Auth, result: Auth },
    subscriptions: 
        { key: "p2p.events", input: never, result: any } | 
        { key: "search.rag", input: RAGRequestPayload, result: RAGResult }
};

export type ModelArtifact = { url: string; checksum: string }

export type AIModelResult = { info: AIModel; status: AIModelStatus }

export type Auth = { id: string; name: string }

export type ContentIndexMetadata = ({ contentType: "Video" } & VideoIndexMetadata) | ({ contentType: "Audio" } & AudioIndexMetadata) | ({ contentType: "Image" } & ImageIndexMetadata) | ({ contentType: "RawText" } & RawTextIndexMetadata) | ({ contentType: "WebPage" } & WebPageIndexMetadata)

export type VideoIndexMetadata = { sliceType: VideoSliceType; startTimestamp: number; endTimestamp: number }

export type WebPageIndexMetadata = { chunkType: WebPageChunkType; startIndex: number; endIndex: number }

export type VideoAvgFrameRate = { numerator: number; denominator: number }

export type ImageRequestPayload = { hash: string }

export type RecommendRequestPayload = { assetObjectHash: string; timestamp: number }

export type SetModelPayload = { category: AIModelCategory; modelId: string }

export type AssetObjectWithMediaData = { id: number; hash: string; size: number; mimeType: string | null; createdAt: string; updatedAt: string; mediaData: ContentMetadata | null }

export type FilePathGetPayload = { materializedPath: string; name: string }

export type RetrievalResultPayload = { filePath: FilePathWithAssetObjectData; metadata: ContentIndexMetadata; score: number; referenceContent: string }

export type TranscriptType = "Original" | "Summarization"

export type AudioType = "txt" | "srt" | "json" | "vtt" | "csv" | "ale" | "docx"

export type RAGResult = { resultType: "Reference"; data: RetrievalResultPayload } | { resultType: "Response"; data: string } | { resultType: "Error"; data: string } | { resultType: "Done" }

export type ImageMetadata = { width: number; height: number; color: string }

export type LibrarySettingsThemeEnum = "light" | "dark"

export type S3Config = { bucket: string; endpoint: string; accessKeyId: string; secretAccessKey: string }

export type RawTextChunkType = "Content"

export type AudioIndexMetadata = { sliceType: AudioSliceType; startTimestamp: number; endTimestamp: number }

export type LibrarySettings = { title: string; appearanceTheme: LibrarySettingsThemeEnum; explorer: LibrarySettingsExplorer; models: LibraryModels; alwaysDeleteLocalFileAfterUpload: boolean; s3Config: S3Config | null }

export type FileHandlerTask = { id: number; assetObjectId: number; taskType: string; exitCode: number | null; exitMessage: string | null; startsAt: string | null; endsAt: string | null; createdAt: string; updatedAt: string }

export type FilePath = { id: number; isDir: boolean; materializedPath: string; name: string; description: string | null; assetObjectId: number | null; createdAt: string; updatedAt: string }

export type VideoPlayerTsResponse = { data: number[] }

export type AudioResp = { type: AudioType; content: string }

export type AIModelCategory = "ImageEmbedding" | "MultiModalEmbedding" | "ImageCaption" | "AudioTranscript" | "TextEmbedding" | "LLM"

export type FilePathDeletePayload = { materializedPath: string; name: string }

export type FilePathQueryPayload = { materializedPath: string; isDir?: boolean | null; includeSubDirs?: boolean | null }

export type ModelsListResult = { category: AIModelCategory; models: AIModelResult[] }

export type LibrariesListResult = { id: string; dir: string; title: string }

export type VideoMetadata = { width: number; height: number; duration: number; bitRate: number; avgFrameRate: VideoAvgFrameRate; audio: AudioMetadata | null }

export type LibraryStatusResult = { id: string | null; loaded: boolean; isBusy: boolean }

export type FilePathWithAssetObjectData = { id: number; isDir: boolean; materializedPath: string; name: string; description: string | null; assetObjectId: number | null; assetObject?: AssetObjectWithMediaData | null; createdAt: string; updatedAt: string }

export type FilePathRequestPayload = { id: number; isDir: boolean; materializedPath: string; name: string }

export type ExportInput = { types: AudioType[]; hash: string; path: string; fileName?: string | null }

export type AudioMetadata = { bitRate: number; duration: number }

export type LibrarySettingsLayoutEnum = "list" | "grid" | "media"

export type AssetObjectCreatePayload = { materializedPath: string; name: string; localFullPath: string }

export type TranscriptRequestPayload = { hash: string; startTimestamp: number; endTimestamp: number; requestType: TranscriptType }

export type ConcreteModelType = "BLIP" | "CLIP" | "Moondream" | "OrtTextEmbedding" | "Whisper" | "Yolo" | "Qwen2" | "OpenAI" | "AzureOpenAI" | "LLaVAPhi3Mini"

export type AcceptShareOutput = { fileList: string[] }

export type WebPageChunkType = "Content"

export type RawTextMetadata = { textCount: number }

export type TaskCancelRequestPayload = { assetObjectId: number; taskTypes: string[] | null }

export type AIModelStatus = { downloaded: boolean; downloadStatus: ModelDownloadStatus | null }

export type LibraryModels = { MultiModalEmbedding: string; TextEmbedding: string; ImageCaption: string; AudioTranscript: string; Llm: string }

export type DownloadModelPayload = { modelId: string }

export type VideoPlayerTsRequestPayload = { hash: string; index: number; size: number }

export type LibraryLoadResult = { id: string; dir: string }

export type FilePathRenamePayload = { id: number; isDir: boolean; materializedPath: string; oldName: string; newName: string }

export type TranscriptResponse = { content: string }

export type SearchResultPayload = { filePath: FilePathWithAssetObjectData; metadata: ContentIndexMetadata; score: number; highlight: string | null }

export type RawTextIndexMetadata = { chunkType: RawTextChunkType; startIndex: number; endIndex: number }

export type ContentMetadata = ({ contentType: "Audio" } & AudioMetadata) | ({ contentType: "Video" } & VideoMetadata) | ({ contentType: "Image" } & ImageMetadata) | ({ contentType: "RawText" } & RawTextMetadata) | ({ contentType: "WebPage" } & WebPageMetadata) | { contentType: "Unknown" }

export type AudioSliceType = "Transcript"

export type WebPageMetadata = { sourceUrl: string }

export type RawTextRequestPayload = { hash: string; index: number }

export type VideoSegmentExportPayload = { verboseFileName: string; outputDir: string; assetObjectId: number; millisecondsFrom: number; millisecondsTo: number }

export type WebPageCreatePayload = { materializedPath: string; url: string }

export type ImageIndexMetadata = { data: number }

export type FilePathCreatePayload = { materializedPath: string; name: string }

export type UploadPayload = { materializedPaths: string[]; hashes: string[] }

export type TaskListRequestFilter = { assetObjectId?: number | null; assetObjectIds?: number[] | null }

export type TaskRedoRequestPayload = { assetObjectId: number }

export type SharePayload = { fileIdList: number[]; peerId: string }

export type TaskListRequestPayload = { filter: TaskListRequestFilter }

export type ModelDownloadStatus = { totalBytes: string; downloadedBytes: string }

export type AssetObjectReceivePayload = { hash: string; materializedPath: string }

export type AIModel = { id: string; title: string; description: string; categories: AIModelCategory[]; artifacts_dir: string; artifacts: ModelArtifact[]; model_type: ConcreteModelType; params: any; dim: number | null }

export type SearchRequestPayload = { text: string }

export type FilePathMovePayload = { active: FilePathRequestPayload; target: FilePathRequestPayload | null }

export type LibrarySettingsExplorer = { layout: LibrarySettingsLayoutEnum; inspectorSize: number; inspectorShow: boolean }

export type VideoSliceType = "Visual" | "Audio"

export type RAGRequestPayload = { query: string }
