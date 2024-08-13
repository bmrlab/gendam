import { matchRetrievalResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Document_Light } from '@gendam/assets/images'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@gendam/ui/v1/tabs'
import Image from 'next/image'
import { match } from 'ts-pattern'

export default function RawTextRetrievalItem(props: ExtractExplorerItem<'RetrievalResult', 'rawText'>) {
  const currentLibrary = useCurrentLibrary()

  return match(props)
    .with(matchRetrievalResult('rawText', 'chunkSumEmbed'), (props) => <RawTextSummarizationItem {...props} />)
    .otherwise(() => (
      <div className="flex flex-col space-y-2 rounded-md p-2">
        <div className="relative h-40 w-64">
          <Image src={Document_Light} className="object-cover" fill priority alt={props.assetObject.hash} />
        </div>
      </div>
    ))
}

function RawTextSummarizationItem({
  assetObject,
  metadata,
}: ExtractExplorerItem<'RetrievalResult', 'rawText', 'chunkSumEmbed'>) {
  const { data: summarization } = rspc.useQuery([
    'assets.artifacts.raw_text.chunk.summarization',
    {
      hash: assetObject.hash,
      index: metadata.startIndex,
    },
  ])

  const { data: content } = rspc.useQuery(
    [
      'assets.artifacts.raw_text.chunk.content',
      {
        hash: assetObject.hash,
        index: metadata.startIndex,
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
          <Image src={Document_Light} className="object-contain" fill priority alt={assetObject.hash} />
        </div>
      </div>

      <Tabs defaultValue="tldr" className="w-full flex-1">
        <TabsList>
          <TabsTrigger value="tldr">TLDR</TabsTrigger>
          <TabsTrigger value="transcript">Detail</TabsTrigger>
        </TabsList>
        <TabsContent value="tldr">{summarization ?? ''}</TabsContent>
        <TabsContent value="transcript">{content ?? ''}</TabsContent>
      </Tabs>
    </div>
  )
}
