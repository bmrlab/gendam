import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { Document_Light } from '@gendam/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

export default function RawTextSearchItem({
  assetObject,
  metadata,
}: ExtractExplorerItem<'SearchResult' | 'RetrievalResult', 'rawText'>) {
  const { data: summarization } = rspc.useQuery([
    'assets.artifacts.raw_text.chunk.summarization',
    { hash: assetObject.hash, index: metadata.startIndex },
  ])
  const { data: content } = rspc.useQuery([
    'assets.artifacts.raw_text.chunk.content',
    { hash: assetObject.hash, index: metadata.startIndex },
  ])

  return (
    <div className="relative h-full w-full">
      <div className="flex h-full items-stretch justify-between">
        {summarization ? (
          <div className="flex h-full w-full items-center justify-center overflow-scroll p-2 text-sm">
            <div>{summarization}</div>
          </div>
        ) : (
          <Image src={Document_Light} alt={assetObject.hash} fill={true} className="object-cover" priority />
        )}
      </div>
      {content && (
        <div
          className={classNames(
            'absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/80 px-4 py-2 text-neutral-300',
            'invisible group-hover:visible',
            'cursor-text select-text overflow-scroll whitespace-pre-line',
          )}
        >
          <div className="text-xs">{content}</div>
        </div>
      )}
    </div>
  )
}
