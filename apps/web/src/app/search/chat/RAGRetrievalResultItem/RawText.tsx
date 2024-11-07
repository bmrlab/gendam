import { matchRetrievalResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Document_Light } from '@gendam/assets/images'
import { Tabs } from '@gendam/ui/v2/tabs'
import Image from 'next/image'
import { match } from 'ts-pattern'

export default function RawTextRetrievalItem(props: ExtractExplorerItem<'RetrievalResult', 'RawText'>) {
  const currentLibrary = useCurrentLibrary()

  return match(props)
    .with(matchRetrievalResult('RawText', { chunkType: 'Content' }), (props) => <RawTextSummarizationItem {...props} />)
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
}: ExtractExplorerItem<'RetrievalResult', 'RawText', { chunkType: 'Content' }>) {
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

      <Tabs.Root defaultValue="tldr" className="w-full flex-1">
        <Tabs.List>
          <Tabs.Trigger value="tldr">TLDR</Tabs.Trigger>
          <Tabs.Trigger value="transcript">Detail</Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="tldr">{summarization ?? ''}</Tabs.Content>
        <Tabs.Content value="transcript">{content ?? ''}</Tabs.Content>
      </Tabs.Root>
    </div>
  )
}
