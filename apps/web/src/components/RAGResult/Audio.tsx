import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import Image from 'next/image'
import { PickRetrievalResult } from '.'

export default function AudioRetrievalItem({ data }: { data: PickRetrievalResult<'audio'> }) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="bg-app-overlay flex flex-col space-y-2 rounded-md p-2">
      <div className="relative h-40 w-64">
        <Image
          src={currentLibrary.getThumbnailSrc(data.filePath.assetObject.hash, 'audio')}
          className="object-cover"
          fill
          priority
          alt={data.filePath.name}
        />
      </div>

      <div className="flex flex-col items-start space-y-1 text-xs text-gray-600">
        <span>{data.filePath.name}</span>
        <div className="flex items-center justify-start space-x-1">
          <span>{formatDuration(data.metadata.startTime / 1e3)}</span>
          <span>→</span>
          <span>{formatDuration(data.metadata.endTime / 1e3)}</span>
        </div>
      </div>
    </div>
  )
}
