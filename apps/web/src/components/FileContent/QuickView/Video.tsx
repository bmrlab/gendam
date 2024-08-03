import { Video } from '@/components/FileView/Video'
import { ExtractExplorerItem, ExtractExplorerItemWithType } from '@/Explorer/types'

export default function VideoQuickView({
  assetObject,
}: {assetObject: ExtractExplorerItemWithType<'video'>['assetObject']}) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Video hash={assetObject.hash} />
      {/* ) : (
        <div className="relative h-full w-full">
          <Image
            src={currentLibrary.getFileSrc(data.assetObject.hash)}
            alt={data.assetObject.hash}
            fill={true}
            className="h-full w-full rounded-md object-contain object-center"
            priority
          />
        </div>
      )} */}
    </div>
  )
}
