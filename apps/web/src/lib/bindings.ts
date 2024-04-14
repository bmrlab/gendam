// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "assets.get", input: FilePathGetPayload, result: FilePath } | 
        { key: "assets.list", input: FilePathQueryPayload, result: FilePath[] } | 
        { key: "audio.find_by_hash", input: string, result: AudioResp[] } | 
        { key: "libraries.get_current_library", input: never, result: CurrentLibraryResult } | 
        { key: "libraries.get_library_settings", input: never, result: LibrarySettings } | 
        { key: "libraries.list", input: never, result: LibrariesListResult[] } | 
        { key: "users.list", input: never, result: any } | 
        { key: "version", input: never, result: string } | 
        { key: "video.search.all", input: SearchRequestPayload, result: SearchResultPayload[] } | 
        { key: "video.tasks.list", input: TaskListRequestPayload, result: VideoWithTasksPageResult },
    mutations: 
        { key: "assets.create_asset_object", input: AssetObjectCreatePayload, result: null } | 
        { key: "assets.create_dir", input: FilePathCreatePayload, result: null } | 
        { key: "assets.delete_file_path", input: FilePathDeletePayload, result: null } | 
        { key: "assets.move_file_path", input: FilePathMovePayload, result: null } | 
        { key: "assets.process_video_asset", input: number, result: null } | 
        { key: "assets.process_video_metadata", input: number, result: null } | 
        { key: "assets.rename_file_path", input: FilePathRenamePayload, result: null } | 
        { key: "audio.batch_export", input: ExportInput[], result: AudioType[] } | 
        { key: "audio.export", input: ExportInput, result: AudioType[] } | 
        { key: "libraries.create", input: string, result: null } | 
        { key: "libraries.quit_current_library", input: string, result: any } | 
        { key: "libraries.set_current_library", input: string, result: any } | 
        { key: "libraries.update_library_settings", input: LibrarySettings, result: null } | 
        { key: "video.tasks.cancel", input: TaskCancelRequestPayload, result: null } | 
        { key: "video.tasks.create", input: string, result: null } | 
        { key: "video.tasks.regenerate", input: TaskRedoRequestPayload, result: null } | 
        { key: "video.tasks.trigger_unfinished", input: string, result: null },
    subscriptions: never
};

export type FilePathCreatePayload = { materializedPath: string; name: string }

export type LibrariesListResult = { id: string; dir: string; title: string }

export type CurrentLibraryResult = { id: string; dir: string }

export type FilePathQueryPayload = { materializedPath: string; isDir?: boolean | null; includeSubDirs?: boolean | null }

export type TaskListRequestFilter = "all" | "processing" | "completed" | "failed" | "canceled" | "excludeCompleted" | { exitCode: number }

export type FilePathRenamePayload = { id: number; isDir: boolean; materializedPath: string; oldName: string; newName: string }

export type FilePathMovePayload = { active: FilePathRequestPayload; target: FilePathRequestPayload | null }

export type FilePath = { id: number; isDir: boolean; materializedPath: string; name: string; description: string | null; assetObjectId: number | null; createdAt: string; updatedAt: string }

export type LibrarySettingsLayoutEnum = "list" | "grid" | "media"

export type TaskRedoRequestPayload = { assetObjectId: number }

export type Pagination = { pageSize: number; pageIndex: number }

export type AssetObjectCreatePayload = { materializedPath: string; name: string; localFullPath: string }

export type SearchResultPayload = { name: string; materializedPath: string; assetObjectId: number; assetObjectHash: string; startTime: number; endTime: number; score: number }

export type FileHandlerTask = { id: number; assetObjectId: number; taskType: string; exitCode: number | null; exitMessage: string | null; startsAt: string | null; endsAt: string | null; createdAt: string; updatedAt: string }

export type LibrarySettingsThemeEnum = "light" | "dark"

export type FilePathGetPayload = { materializedPath: string; name: string }

export type TaskCancelRequestPayload = { assetObjectId: number }

export type AudioResp = { type: AudioType; content: string }

export type FilePathRequestPayload = { id: number; isDir: boolean; materializedPath: string; name: string }

export type LibrarySettings = { title: string; appearanceTheme: LibrarySettingsThemeEnum; explorerLayout: LibrarySettingsLayoutEnum }

export type MediaData = { id: number; width: number | null; height: number | null; duration: number | null; bitRate: number | null; hasAudio: boolean | null; assetObjectId: number; createdAt: string; updatedAt: string }

export type VideoWithTasksPageResult = { data: VideoWithTasksResult[]; pagination: Pagination; maxPage: number }

export type VideoWithTasksResult = { name: string; materializedPath: string; assetObject: AssetObject; tasks: FileHandlerTask[]; mediaData: MediaData | null }

export type TaskListRequestPayload = { pagination: Pagination; filter: TaskListRequestFilter }

export type FilePathDeletePayload = { materializedPath: string; name: string }

export type AssetObject = { id: number; hash: string; size: number; mimeType: string | null; createdAt: string; updatedAt: string }

export type ExportInput = { types: AudioType[]; hash: string; path: string; fileName?: string | null }

export type SearchRequestPayload = { text: string; recordType: string }

export type AudioType = "txt" | "srt" | "json" | "vtt" | "csv" | "ale" | "docx"
