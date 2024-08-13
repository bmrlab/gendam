import { matchRetrievalResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { formatDuration } from '@/lib/utils'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@gendam/ui/v1/tabs'
import Image from 'next/image'
import { match } from 'ts-pattern'

export default function VideoRetrievalItem(props: ExtractExplorerItem<'RetrievalResult', 'video'>) {
  const currentLibrary = useCurrentLibrary()
  const { assetObject, metadata } = props

  return match(props)
    .with(matchRetrievalResult('video', 'transChunkSumEmbed'), (props) => <VideoTranscriptItem {...props} />)
    .otherwise(() => (
      <div className="relative h-full w-full">
        <div className="flex h-full items-stretch justify-between">
          <Image
            src={currentLibrary.getThumbnailSrc(assetObject.hash, 'video')}
            alt={assetObject.hash}
            fill={true}
            className="object-cover"
            priority
          />
        </div>
      </div>
    ))
}

function VideoTranscriptItem({
  assetObject,
  metadata,
}: ExtractExplorerItem<'RetrievalResult', 'video', 'transChunkSumEmbed'>) {
  const currentLibrary = useCurrentLibrary()

  const { data: summarization } = rspc.useQuery([
    'assets.artifacts.video.transcript',
    {
      hash: assetObject.hash,
      startTimestamp: metadata.startTime,
      endTimestamp: metadata.endTime,
      requestType: 'Summarization',
    },
  ])

  const { data: transcript } = rspc.useQuery(
    [
      'assets.artifacts.video.transcript',
      {
        hash: assetObject.hash,
        startTimestamp: metadata.startTime,
        endTimestamp: metadata.endTime,
        requestType: 'Original',
      },
    ],
    {
      enabled: true, // TODO only retrieve when transcript is enabled
    },
  )

  return (
    <div className="flex items-start justify-between space-x-4 rounded-md">
      <div className="flex flex-col space-y-2">
        <div className="relative h-[200px] w-[280px]">
          <Image
            src={currentLibrary.getPreviewSrc(assetObject.hash, 'video', Math.floor(metadata.startTime / 1e3))}
            className="object-cover"
            fill
            priority
            alt={assetObject.hash}
          />
        </div>

        <div className="flex flex-col items-start space-y-1 text-xs text-gray-600">
          <div className="flex items-center justify-start space-x-1">
            <span>{formatDuration(metadata.startTime / 1e3)}</span>
            <span>→</span>
            <span>{formatDuration(metadata.endTime / 1e3)}</span>
          </div>
        </div>
      </div>

      <Tabs defaultValue="tldr" className="w-full flex-1">
        <TabsList>
          <TabsTrigger value="tldr">TLDR</TabsTrigger>
          <TabsTrigger value="transcript">Transcript</TabsTrigger>
        </TabsList>
        <TabsContent value="tldr">{summarization?.content ?? ''}</TabsContent>
        <TabsContent value="transcript">{transcript?.content ?? ''}</TabsContent>
      </Tabs>
    </div>
  )
}
