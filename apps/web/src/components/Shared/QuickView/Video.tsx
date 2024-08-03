import { Video } from '@/components/FileView/Video'
import { PickQuickViewItem } from './store'

export default function VideoQuickView({ data }: { data: PickQuickViewItem<'Video'> }) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Video hash={data.assetObject.hash} />
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
