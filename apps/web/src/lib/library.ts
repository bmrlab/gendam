import { ContentMetadataWithType, ContentTaskTypeSpecta, LibrarySettings, SearchResultMetadata } from '@/lib/bindings'
import { ContextType, createContext, useContext } from 'react'

export type Library = {
  id: string
  dir: string
}

export type AssetObjectType =
  | ContentMetadataWithType['contentType']
  | SearchResultMetadata['type']
  | ContentTaskTypeSpecta['contentType']

type CurrentLibraryContext = {
  id: string
  dir: string
  librarySettings: LibrarySettings
  updateLibrarySettings: (partialSettingssettings: Partial<LibrarySettings>) => Promise<void>
  switchCurrentLibraryById: (libraryId: string) => Promise<void>
  getFileSrc: (assetObjectHash: string) => string
  getThumbnailSrc: (assetObjectHash: string, assetObjectType: AssetObjectType) => string
  getVideoPreviewSrc: (assetObjectHash: string, timestampInSecond?: number) => string
  getAudioPreviewSrc: (assetObjectHash: string) => string | undefined
}

export const CurrentLibrary = createContext<CurrentLibraryContext | null>(null)

type NonNullableCurrentLibrary = NonNullable<ContextType<typeof CurrentLibrary>>

export const useCurrentLibrary = () => useContext(CurrentLibrary) as NonNullableCurrentLibrary
