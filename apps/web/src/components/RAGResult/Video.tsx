import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { formatDuration } from '@/lib/utils'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@gendam/ui/v1/tabs'
import classNames from 'classnames'
import Image from 'next/image'
import { match } from 'ts-pattern'
import { matchRetrievalResultWithTaskPattern, PickRetrievalResult, PickRetrievalResultWithTask } from '.'

export default function VideoRetrievalItem({ data }: { data: PickRetrievalResult<'video'> }) {
  const currentLibrary = useCurrentLibrary()

  return match(data)
    .with(matchRetrievalResultWithTaskPattern('video', 'transChunkSumEmbed'), (item) => (
      <VideoTranscriptItem data={item} />
    ))
    .otherwise(() => (
      <div className="relative h-full w-full">
        <div className="flex h-full items-stretch justify-between">
          <Image
            src={currentLibrary.getThumbnailSrc(data.filePath.assetObject.hash, 'audio')}
            alt={data.filePath.name}
            fill={true}
            className="object-cover"
            priority
          />
        </div>
        <div
          className={classNames(
            'absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300',
            'invisible group-hover:visible',
          )}
        >
          <div className="truncate text-xs">
            {data.filePath.materializedPath}
            {data.filePath.name}
          </div>
          <div className="flex items-center justify-between text-xs">
            <div>{formatDuration(data.metadata.startTime / 1000)}</div>
            <div>→</div>
            <div>{formatDuration(data.metadata.endTime / 1000 + 1)}</div>
          </div>
        </div>
      </div>
    ))
}

function VideoTranscriptItem({ data }: { data: PickRetrievalResultWithTask<'video', 'transChunkSumEmbed'> }) {
  const currentLibrary = useCurrentLibrary()

  const { data: summarization } = rspc.useQuery([
    'video.rag.transcript',
    {
      hash: data.filePath.assetObject.hash,
      startTimestamp: data.metadata.startTime,
      endTimestamp: data.metadata.endTime,
      requestType: 'Summarization',
    },
  ])

  const { data: transcript } = rspc.useQuery(
    [
      'video.rag.transcript',
      {
        hash: data.filePath.assetObject?.hash!,
        startTimestamp: data.metadata.startTime,
        endTimestamp: data.metadata.endTime,
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
            src={currentLibrary.getVideoPreviewSrc(
              data.filePath.assetObject?.hash!,
              Math.floor(data.metadata.startTime / 1e3),
            )}
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

      <Tabs defaultValue="tldr" className="w-full flex-1">
        <TabsList>
          <TabsTrigger value="tldr">TLDR</TabsTrigger>
          <TabsTrigger value="transcript">transcript</TabsTrigger>
        </TabsList>
        <TabsContent value="tldr">{summarization?.content ?? ''}</TabsContent>
        <TabsContent value="transcript">{transcript?.content ?? ''}</TabsContent>
      </Tabs>
    </div>
  )
}
