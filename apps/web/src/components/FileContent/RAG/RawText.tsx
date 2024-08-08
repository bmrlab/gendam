import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { Document_Light } from '@gendam/assets/images'
import Image from 'next/image'

export default function RawTextRetrievalItem({
  assetObject,
  metadata,
}: ExtractExplorerItem<'RetrievalResult', 'rawText'>) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="bg-app-overlay flex flex-col space-y-2 rounded-md p-2">
      <div className="relative h-40 w-64">
        <Image src={Document_Light} className="object-cover" fill priority alt={assetObject.hash} />
      </div>

      <div className="flex flex-col items-start space-y-1 text-xs text-gray-600">
        <span>{assetObject.hash}</span>
        <div className="flex items-center justify-start space-x-1">
          <span>{metadata.index}</span>
        </div>
      </div>
    </div>
  )
}
