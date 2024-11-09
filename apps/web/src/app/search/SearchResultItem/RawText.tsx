import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
// import { Document_Light } from '@gendam/assets/images'
// import Image from 'next/image'
import { useMemo } from 'react'

export default function RawTextSearchItem(itemData: ExtractExplorerItem<'SearchResult', 'RawText'>) {
  const { assetObject, metadata, hitText } = itemData
  const title = useMemo(() => {
    if (itemData.type === 'SearchResult') {
      return itemData.filePaths[0]?.name ?? ''
    } else {
      return ''
    }
  }, [itemData])

  // const { data: summarization } = rspc.useQuery([
  //   'assets.artifacts.raw_text.chunk.summarization',
  //   { hash: assetObject.hash, index: metadata.startIndex },
  // ])

  const { data: content } = rspc.useQuery([
    'assets.artifacts.raw_text.chunk.content',
    { hash: assetObject.hash, index: metadata.startIndex },
  ])

  return (
    <div className="relative h-full w-full p-2">
      <div className="flex h-full flex-col items-stretch justify-start gap-1 overflow-hidden">
        <div className="relative flex items-center justify-start gap-1 py-1">
          <Icon.File className="h-4 w-4" />
          <div className="flex-1 text-xs font-bold">{title}</div>
        </div>
        <div className="text-ink/70 w-full flex-1 overflow-auto whitespace-pre-line break-words p-1 text-xs">
          {content?.trim()}
        </div>
      </div>
    </div>
    // <div className="relative h-full w-full">
    //   <div className="flex h-full items-stretch justify-between">
    //     {summarization ? (
    //       <div className="flex h-full w-full items-center justify-center overflow-scroll p-2 text-sm">
    //         <div>{summarization}</div>
    //       </div>
    //     ) : (
    //       <Image src={Document_Light} alt={assetObject.hash} fill={true} className="object-cover" priority />
    //     )}
    //   </div>
    //   {content && (
    //     <div
    //       className={classNames(
    //         'absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/80 px-4 py-2 text-neutral-300',
    //         'invisible group-hover:visible',
    //         'cursor-text select-text overflow-scroll whitespace-pre-line',
    //       )}
    //     >
    //       <div className="text-xs">{content}</div>
    //     </div>
    //   )}
    // </div>
  )
}
