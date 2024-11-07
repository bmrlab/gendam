import { ContentIndexMetadata, ContentMetadataWithType, LibrarySettings } from '@/lib/bindings'
import { ContextType, createContext, useContext } from 'react'

export type Library = {
  id: string
  dir: string
}

export type AssetObjectType = ContentMetadataWithType['contentType'] | ContentIndexMetadata['contentType']

export type AssetPreviewMetadata = {
  (assetObjectHash: string, contentType: 'Audio'): string
  (assetObjectHash: string, contentType: 'Video', timestampInSecond?: number): string
}

export type CurrentLibraryContext = {
  id: string
  dir: string
  librarySettings: LibrarySettings
  updateLibrarySettings: (partialSettingssettings: Partial<LibrarySettings>) => Promise<void>
  switchCurrentLibraryById: (libraryId: string) => Promise<void>
  getFileSrc: (assetObjectHash: string) => string
  getThumbnailSrc: (assetObjectHash: string, assetObjectType: AssetObjectType) => string
  getPreviewSrc: AssetPreviewMetadata
}

export const CurrentLibrary = createContext<CurrentLibraryContext | null>(null)

type NonNullableCurrentLibrary = NonNullable<ContextType<typeof CurrentLibrary>>

export const useCurrentLibrary = () => useContext(CurrentLibrary) as NonNullableCurrentLibrary
